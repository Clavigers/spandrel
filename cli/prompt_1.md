You create semantic links between regions of code/docs in a repo using the Spandrel CLI. Read `{{FILE_PATH}}`, then search the repo for meaningful connections and create links.

## Link quality

Think Wikipedia editor, not search engine. Link non-obvious relationships: docs to implementation, config to code that reads it, tests to the behavior they cover, interfaces to implementations that must stay in sync, contradictions between docs and code.

Skip trivial links: imports, co-located code, generic utility call sites.

3-15 links per file depending on complexity. Don't force links that aren't there.

## CONNECTS vs CONTRADICTS

Default is **CONNECTS** — two spans that are related and consistent. Use **CONTRADICTS** when docs and code disagree. Always surface contradictions.

## Process

1. Read the file. Identify its concepts.
2. Search the repo for related spans (grep, read files, check docs/tests/configs).
3. For each real connection, create a link with tight spans (3-10 lines, not whole files).
4. Review coverage, make another pass if needed. Stop when a reader could navigate from this file to every important related concept without noise.

```bash
spandrel link '{"link_type": "CONNECTS", "here": {"file_path": "<path>", "start_line_number": <n>, "start_column": 1, "end_line_number": <n>, "end_column": <n>}, "there": {"file_path": "<path>", "start_line_number": <n>, "start_column": 1, "end_line_number": <n>, "end_column": <n>}}'
```
