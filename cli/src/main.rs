use clap::{Parser, Subcommand};
use serde::Deserialize;
use sqlx::PgPool;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

#[derive(Parser)]
#[command(name = "spandrel", about = "Create and validate source-code links")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a validated link between two source spans
    Link {
        /// JSON string describing the link (see README for schema)
        json: String,
    },
    /// Print the source content of a link's spans
    Print {
        /// UUID of the link to print, or --all for every link
        #[arg(default_value = None)]
        id: Option<String>,

        /// Print all links
        #[arg(long)]
        all: bool,
    },
    /// Show the percentage of CONTRADICTS vs CONNECTS links for the current snapshot
    Stats,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum LinkType {
    Connects,
    Contradicts,
}

impl LinkType {
    fn as_str(&self) -> &'static str {
        match self {
            LinkType::Connects => "CONNECTS",
            LinkType::Contradicts => "CONTRADICTS",
        }
    }
}

#[derive(Debug, Deserialize)]
struct SourceSpan {
    file_path: String,
    start_line_number: i32,
    #[serde(default)]
    start_column: i32,
    end_line_number: i32,
    #[serde(default)]
    end_column: i32,
    /// Optional: instead of specifying columns, provide the exact text to match
    /// within the line range. Columns will be resolved from this.
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LinkInput {
    link_type: LinkType,
    here: SourceSpan,
    there: Vec<SourceSpan>,
}

fn parse_link_json(json: &str) -> Result<LinkInput, String> {
    serde_json::from_str::<LinkInput>(json).map_err(|e| {
        format!(
            "Invalid link JSON: {e}\n\n\
             Expected schema:\n  \
             {{\n    \
               \"link_type\": \"CONNECTS\" | \"CONTRADICTS\",\n    \
               \"here\": {{ \"file_path\": string, \"start_line_number\": int, \"end_line_number\": int, ...columns }},\n    \
               \"there\": [{{ ... same as here ... }}, ...]\n  \
             }}\n\n\
             Columns can be specified two ways:\n  \
               1. \"start_column\": int, \"end_column\": int\n  \
               2. \"text\": string  (exact substring to match within the line range)"
        )
    })
}

fn count_lines(path: &Path) -> Result<usize, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    Ok(content.lines().count())
}

/// If the span has a `text` field, resolve it to exact column numbers by finding
/// the text as a substring within the specified line range of the file.
fn resolve_text_to_columns(span: &mut SourceSpan, label: &str, repo_root: &Path) -> Result<(), String> {
    let needle = match &span.text {
        Some(t) => t.clone(),
        None => return Ok(()),
    };

    let path = repo_root.join(&span.file_path);
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("{label}: failed to read {}: {e}", span.file_path))?;
    let file_lines: Vec<&str> = content.lines().collect();

    let start_idx = (span.start_line_number - 1) as usize;
    let end_idx = (span.end_line_number - 1) as usize;

    if start_idx >= file_lines.len() || end_idx >= file_lines.len() {
        return Err(format!(
            "{label}: line range L{}–L{} is out of bounds (file has {} lines)",
            span.start_line_number, span.end_line_number, file_lines.len()
        ));
    }

    // Build the text of the line range
    let region: String = file_lines[start_idx..=end_idx].join("\n");

    if let Some(byte_offset) = region.find(&needle) {
        // Convert byte offset to line + column within the region
        let before = &region[..byte_offset];
        let newlines_before = before.matches('\n').count();
        let col_start = if newlines_before == 0 {
            byte_offset + 1 // 1-indexed
        } else {
            byte_offset - before.rfind('\n').unwrap() // chars after last newline, already 1-indexed
        };

        let needle_end = byte_offset + needle.len();
        let up_to_end = &region[..needle_end];
        let newlines_in = up_to_end.matches('\n').count();
        let col_end = if newlines_in == newlines_before {
            // Same line as start
            col_start + needle.len() - 1
        } else {
            let last_nl = up_to_end.rfind('\n').unwrap();
            needle_end - last_nl - 1
        };

        span.start_line_number += newlines_before as i32;
        span.start_column = col_start as i32;
        span.end_line_number = span.start_line_number + (newlines_in - newlines_before) as i32;
        span.end_column = col_end as i32;
        span.text = None; // consumed

        Ok(())
    } else {
        // Build a diagnostic showing what the lines actually contain
        let mut diagnostic = format!(
            "{label}: text not found in L{}–L{} of {}\n\n",
            span.start_line_number, span.end_line_number, span.file_path
        );
        diagnostic.push_str("  Searched for:\n");
        diagnostic.push_str(&format!("    \"{needle}\"\n\n"));
        diagnostic.push_str("  Actual content of those lines:\n");
        for (i, line_idx) in (start_idx..=end_idx).enumerate() {
            diagnostic.push_str(&format!(
                "    L{}: \"{}\"\n",
                span.start_line_number + i as i32,
                file_lines[line_idx]
            ));
        }

        // Check for near-matches: case-insensitive or trimmed whitespace
        let needle_lower = needle.to_lowercase();
        let region_lower = region.to_lowercase();
        if region_lower.contains(&needle_lower) {
            diagnostic.push_str("\n  Hint: a case-insensitive match was found. Check capitalization.\n");
        } else {
            // Check if any individual line contains a substantial substring
            let needle_trimmed = needle.trim();
            if !needle_trimmed.is_empty() && region.contains(needle_trimmed) {
                diagnostic.push_str("\n  Hint: a match was found after trimming whitespace from your search text.\n");
            }
        }

        Err(diagnostic)
    }
}

fn validate_span(span: &SourceSpan, label: &str, repo_root: &Path) -> Result<(), String> {
    let path = repo_root.join(&span.file_path);
    if !path.exists() {
        return Err(format!(
            "{label}: file does not exist: {} (resolved to {})",
            span.file_path,
            path.display()
        ));
    }

    if span.start_line_number < 1 {
        return Err(format!(
            "{label}: start_line_number must be >= 1, got {}",
            span.start_line_number
        ));
    }
    if span.start_column < 1 {
        return Err(format!(
            "{label}: start_column must be >= 1, got {}",
            span.start_column
        ));
    }
    if span.end_line_number < span.start_line_number {
        return Err(format!(
            "{label}: end_line_number ({}) must be >= start_line_number ({})",
            span.end_line_number, span.start_line_number
        ));
    }
    if span.end_column < 1 {
        return Err(format!(
            "{label}: end_column must be >= 1, got {}",
            span.end_column
        ));
    }

    let total_lines = count_lines(&path)?;
    if span.end_line_number as usize > total_lines {
        return Err(format!(
            "{label}: end_line_number ({}) exceeds total lines in file ({total_lines}): {}",
            span.end_line_number, span.file_path
        ));
    }

    Ok(())
}

fn git_repo_root() -> Result<PathBuf, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| format!("Failed to run git: {e}"))?;
    if !output.status.success() {
        return Err("Not inside a git repository".to_string());
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}

fn normalize_to_repo_root(span: &mut SourceSpan, repo_root: &Path) -> Result<(), String> {
    let input_path = Path::new(&span.file_path);
    let absolute = if input_path.is_absolute() {
        input_path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Failed to get cwd: {e}"))?
            .join(input_path)
    };
    let canonical = absolute
        .canonicalize()
        .map_err(|_| format!("File does not exist: {}", span.file_path))?;
    let repo_canonical = repo_root
        .canonicalize()
        .map_err(|e| format!("Failed to resolve repo root: {e}"))?;
    let relative = canonical
        .strip_prefix(&repo_canonical)
        .map_err(|_| {
            format!(
                "File {} is outside the repo root {}",
                canonical.display(),
                repo_canonical.display()
            )
        })?;
    span.file_path = relative.to_string_lossy().to_string();
    Ok(())
}

fn ensure_columns_present(span: &SourceSpan, label: &str) -> Result<(), String> {
    if span.text.is_none() && (span.start_column == 0 || span.end_column == 0) {
        return Err(format!(
            "{label}: must provide either \"text\" or both \"start_column\" and \"end_column\""
        ));
    }
    Ok(())
}

fn validate_link(link: &mut LinkInput, repo_root: &Path) -> Result<(), String> {
    normalize_to_repo_root(&mut link.here, repo_root)?;
    ensure_columns_present(&link.here, "here")?;
    resolve_text_to_columns(&mut link.here, "here", repo_root)?;
    validate_span(&link.here, "here", repo_root)?;
    if link.there.is_empty() {
        return Err("there: must contain at least one target span".to_string());
    }
    for (i, target) in link.there.iter_mut().enumerate() {
        let label = format!("there[{i}]");
        normalize_to_repo_root(target, repo_root)?;
        ensure_columns_present(target, &label)?;
        resolve_text_to_columns(target, &label, repo_root)?;
        validate_span(target, &label, repo_root)?;
    }
    Ok(())
}

fn gh_repo_name() -> Result<String, String> {
    let output = Command::new("gh")
        .args(["repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"])
        .output()
        .map_err(|e| format!("Failed to run gh: {e}. Is the GitHub CLI installed?"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh repo view failed: {stderr}"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_commit_hash() -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| format!("Failed to run git: {e}"))?;
    if !output.status.success() {
        return Err("Could not determine HEAD commit (is there at least one commit?)".to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn get_or_create_snapshot(
    pool: &PgPool,
    repo_path: &str,
    commit_hash: &str,
) -> Result<uuid::Uuid, String> {
    let row: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM snapshots WHERE repo_path = $1 AND commit_hash = $2")
            .bind(repo_path)
            .bind(commit_hash)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("Database error: {e}"))?;

    if let Some((id,)) = row {
        return Ok(id);
    }

    let id = uuid::Uuid::new_v4();
    sqlx::query("INSERT INTO snapshots (id, repo_path, commit_hash) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(repo_path)
        .bind(commit_hash)
        .execute(pool)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

    Ok(id)
}

async fn insert_link(
    pool: &PgPool,
    snapshot_id: uuid::Uuid,
    link: &LinkInput,
) -> Result<uuid::Uuid, String> {
    let id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO links (
            id, snapshot_id, link_type,
            file_path, start_line, start_column, end_line, end_column
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(id)
    .bind(snapshot_id)
    .bind(link.link_type.as_str())
    .bind(&link.here.file_path)
    .bind(link.here.start_line_number)
    .bind(link.here.start_column)
    .bind(link.here.end_line_number)
    .bind(link.here.end_column)
    .execute(pool)
    .await
    .map_err(|e| format!("Database error: {e}"))?;

    for target in &link.there {
        let target_id = uuid::Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO link_targets (
                id, link_id,
                file_path, start_line, start_column, end_line, end_column
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(target_id)
        .bind(id)
        .bind(&target.file_path)
        .bind(target.start_line_number)
        .bind(target.start_column)
        .bind(target.end_line_number)
        .bind(target.end_column)
        .execute(pool)
        .await
        .map_err(|e| format!("Database error: {e}"))?;
    }

    Ok(id)
}

#[derive(sqlx::FromRow)]
struct LinkHeaderRow {
    id: uuid::Uuid,
    link_type: String,
    file_path: String,
    start_line: i32,
    start_column: i32,
    end_line: i32,
    end_column: i32,
    repo_path: String,
    commit_hash: String,
}

#[derive(sqlx::FromRow)]
struct TargetRow {
    file_path: String,
    start_line: i32,
    start_column: i32,
    end_line: i32,
    end_column: i32,
}

struct LinkRow {
    link_type: String,
    file_path: String,
    start_line: i32,
    start_column: i32,
    end_line: i32,
    end_column: i32,
    targets: Vec<TargetRow>,
    repo_path: String,
    commit_hash: String,
}

async fn fetch_targets(pool: &PgPool, link_id: uuid::Uuid) -> Result<Vec<TargetRow>, String> {
    sqlx::query_as::<_, TargetRow>(
        r#"
        SELECT file_path, start_line, start_column, end_line, end_column
        FROM link_targets
        WHERE link_id = $1
        ORDER BY file_path, start_line
        "#,
    )
    .bind(link_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Database error: {e}"))
}

async fn fetch_link(pool: &PgPool, id: uuid::Uuid) -> Result<LinkRow, String> {
    let header = sqlx::query_as::<_, LinkHeaderRow>(
        r#"
        SELECT l.id, l.link_type,
               l.file_path, l.start_line, l.start_column, l.end_line, l.end_column,
               s.repo_path, s.commit_hash
        FROM links l
        JOIN snapshots s ON l.snapshot_id = s.id
        WHERE l.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Database error: {e}"))?
    .ok_or_else(|| format!("Link not found: {id}"))?;

    let targets = fetch_targets(pool, header.id).await?;

    Ok(LinkRow {
        link_type: header.link_type,
        file_path: header.file_path,
        start_line: header.start_line,
        start_column: header.start_column,
        end_line: header.end_line,
        end_column: header.end_column,
        targets,
        repo_path: header.repo_path,
        commit_hash: header.commit_hash,
    })
}

async fn fetch_all_links(pool: &PgPool) -> Result<Vec<LinkRow>, String> {
    let headers = sqlx::query_as::<_, LinkHeaderRow>(
        r#"
        SELECT l.id, l.link_type,
               l.file_path, l.start_line, l.start_column, l.end_line, l.end_column,
               s.repo_path, s.commit_hash
        FROM links l
        JOIN snapshots s ON l.snapshot_id = s.id
        ORDER BY s.repo_path, s.commit_hash, l.file_path, l.start_line
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Database error: {e}"))?;

    let mut links = Vec::with_capacity(headers.len());
    for header in headers {
        let targets = fetch_targets(pool, header.id).await?;
        links.push(LinkRow {
            link_type: header.link_type,
            file_path: header.file_path,
            start_line: header.start_line,
            start_column: header.start_column,
            end_line: header.end_line,
            end_column: header.end_column,
            targets,
            repo_path: header.repo_path,
            commit_hash: header.commit_hash,
        });
    }

    Ok(links)
}

fn is_local_match(repo_path: &str, commit_hash: &str) -> bool {
    let local_repo = gh_repo_name().ok();
    let local_commit = git_commit_hash().ok();
    local_repo.as_deref() == Some(repo_path) && local_commit.as_deref() == Some(commit_hash)
}

fn read_file_local(repo_root: &Path, file_path: &str) -> Result<String, String> {
    fs::read_to_string(repo_root.join(file_path))
        .map_err(|e| format!("Failed to read {file_path}: {e}"))
}

fn read_file_gh(repo_path: &str, commit_hash: &str, file_path: &str) -> Result<String, String> {
    let output = Command::new("gh")
        .args([
            "api",
            &format!("repos/{repo_path}/contents/{file_path}?ref={commit_hash}"),
            "--jq", ".content",
        ])
        .output()
        .map_err(|e| format!("Failed to run gh: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh api failed for {file_path}: {stderr}"));
    }
    let b64 = String::from_utf8_lossy(&output.stdout).trim().replace('\n', "");
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&b64)
        .map_err(|e| format!("Failed to decode base64 for {file_path}: {e}"))?;
    String::from_utf8(bytes).map_err(|e| format!("File {file_path} is not valid UTF-8: {e}"))
}

fn extract_span(content: &str, start_line: i32, start_col: i32, end_line: i32, end_col: i32) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start_idx = (start_line - 1) as usize;
    let end_idx = (end_line - 1) as usize;

    if start_idx >= lines.len() {
        return String::from("<span out of bounds>");
    }

    let mut result = String::new();
    for (i, line_idx) in (start_idx..=end_idx.min(lines.len() - 1)).enumerate() {
        let line = lines[line_idx];
        let slice = if start_idx == end_idx {
            let sc = (start_col - 1) as usize;
            let ec = end_col as usize;
            &line[sc.min(line.len())..ec.min(line.len())]
        } else if line_idx == start_idx {
            let sc = (start_col - 1) as usize;
            &line[sc.min(line.len())..]
        } else if line_idx == end_idx {
            let ec = end_col as usize;
            &line[..ec.min(line.len())]
        } else {
            line
        };
        if i > 0 {
            result.push('\n');
        }
        result.push_str(slice);
    }
    result
}

fn print_link(link: &LinkRow, local: bool, repo_root: Option<&Path>) -> Result<(), String> {
    let read_file = |file_path: &str| -> Result<String, String> {
        if local {
            read_file_local(repo_root.unwrap(), file_path)
        } else {
            read_file_gh(&link.repo_path, &link.commit_hash, file_path)
        }
    };

    let here_content = read_file(&link.file_path)?;
    let here_span = extract_span(
        &here_content,
        link.start_line, link.start_column,
        link.end_line, link.end_column,
    );

    let source = if local { "local" } else { "GitHub API" };
    let icon = if link.link_type == "CONNECTS" { "✅" } else { "❌" };
    println!("# {icon} Link\n");
    println!("> `{}@{:.8}` via {source}\n", link.repo_path, link.commit_hash);
    println!("## here — L{}:{} → L{}:{}\n",
        link.start_line, link.start_column,
        link.end_line, link.end_column,
    );
    println!("`{}`\n", link.file_path);
    println!("```\n{here_span}\n```\n");

    for (i, target) in link.targets.iter().enumerate() {
        let there_content = read_file(&target.file_path)?;
        let there_span = extract_span(
            &there_content,
            target.start_line, target.start_column,
            target.end_line, target.end_column,
        );
        println!("## there[{i}] — L{}:{} → L{}:{}\n",
            target.start_line, target.start_column,
            target.end_line, target.end_column,
        );
        println!("`{}`\n", target.file_path);
        println!("```\n{there_span}\n```\n");
    }

    Ok(())
}

fn fail(msg: &str) -> ! {
    eprintln!("Error: {msg}");
    process::exit(1);
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Link { json } => {
            let mut link = match parse_link_json(&json) {
                Ok(l) => l,
                Err(e) => fail(&e),
            };

            let repo_root = match git_repo_root() {
                Ok(r) => r,
                Err(e) => fail(&e),
            };

            if let Err(e) = validate_link(&mut link, &repo_root) {
                fail(&e);
            }

            let repo_path = match gh_repo_name() {
                Ok(p) => p,
                Err(e) => fail(&e),
            };
            let commit_hash = match git_commit_hash() {
                Ok(h) => h,
                Err(e) => fail(&e),
            };

            let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                fail("DATABASE_URL environment variable is not set");
            });

            let pool = PgPool::connect(&database_url)
                .await
                .unwrap_or_else(|e| fail(&format!("Failed to connect to database: {e}")));

            let snapshot_id = match get_or_create_snapshot(&pool, &repo_path, &commit_hash).await {
                Ok(id) => id,
                Err(e) => fail(&e),
            };

            let target_count = link.there.len();
            match insert_link(&pool, snapshot_id, &link).await {
                Ok(id) => println!("Link created: {id} ({target_count} target{}, snapshot: {repo_path}@{commit_hash:.8})",
                    if target_count == 1 { "" } else { "s" }),
                Err(e) => fail(&e),
            }
        }
        Commands::Stats => {
            let repo_path = match gh_repo_name() {
                Ok(p) => p,
                Err(e) => fail(&e),
            };
            let commit_hash = match git_commit_hash() {
                Ok(h) => h,
                Err(e) => fail(&e),
            };

            let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                fail("DATABASE_URL environment variable is not set");
            });

            let pool = PgPool::connect(&database_url)
                .await
                .unwrap_or_else(|e| fail(&format!("Failed to connect to database: {e}")));

            #[derive(sqlx::FromRow)]
            struct TypeCount {
                link_type: String,
                count: i64,
            }

            let rows = sqlx::query_as::<_, TypeCount>(
                r#"
                SELECT l.link_type, COUNT(*) as count
                FROM links l
                JOIN snapshots s ON l.snapshot_id = s.id
                WHERE s.repo_path = $1 AND s.commit_hash = $2
                GROUP BY l.link_type
                "#,
            )
            .bind(&repo_path)
            .bind(&commit_hash)
            .fetch_all(&pool)
            .await
            .unwrap_or_else(|e| fail(&format!("Database error: {e}")));

            let mut connects: i64 = 0;
            let mut contradicts: i64 = 0;
            for row in &rows {
                match row.link_type.as_str() {
                    "CONNECTS" => connects = row.count,
                    "CONTRADICTS" => contradicts = row.count,
                    _ => {}
                }
            }

            let total = connects + contradicts;
            if total == 0 {
                println!("No links found for snapshot {repo_path}@{:.8}", commit_hash);
                return;
            }

            let pct_contradicts = (contradicts as f64 / total as f64) * 100.0;
            let pct_connects = (connects as f64 / total as f64) * 100.0;

            println!("Snapshot: {repo_path}@{:.8}\n", commit_hash);
            println!("  CONNECTS:    {connects:>4}  ({pct_connects:.1}%)");
            println!("  CONTRADICTS: {contradicts:>4}  ({pct_contradicts:.1}%)");
            println!("  Total:       {total:>4}");
        }
        Commands::Print { id, all } => {
            let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                fail("DATABASE_URL environment variable is not set");
            });

            let pool = PgPool::connect(&database_url)
                .await
                .unwrap_or_else(|e| fail(&format!("Failed to connect to database: {e}")));

            let links: Vec<LinkRow> = if all {
                match fetch_all_links(&pool).await {
                    Ok(l) => l,
                    Err(e) => fail(&e),
                }
            } else {
                let id_str = id.unwrap_or_else(|| fail("Provide a link UUID or use --all"));
                let id = uuid::Uuid::parse_str(&id_str)
                    .unwrap_or_else(|e| fail(&format!("Invalid UUID: {e}")));
                match fetch_link(&pool, id).await {
                    Ok(l) => vec![l],
                    Err(e) => fail(&e),
                }
            };

            if links.is_empty() {
                println!("No links found.");
                return;
            }

            for (i, link) in links.iter().enumerate() {
                if i > 0 {
                    println!("\n---\n");
                }
                let local = is_local_match(&link.repo_path, &link.commit_hash);
                let repo_root = if local { git_repo_root().ok() } else { None };
                if let Err(e) = print_link(link, local, repo_root.as_deref()) {
                    fail(&e);
                }
            }
        }
    }
}
