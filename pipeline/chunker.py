"""Chunker: ingests a repo via gitingest and pushes chunks to Hatchet."""

import sys
from dataclasses import dataclass

from pathlib import Path
from gitingest import ingest
from dotenv import load_dotenv

from worker import link_chunk_workflow, ChunkPayload

load_dotenv()


CHUNK_SIZE = 2000
SKIP_SUFFIXES = [".lock", ".gitignore", ".python-version"]


def _is_separator(line: str) -> bool:
    s = line.rstrip("\r\n")
    return len(s) >= 4 and all(c == "=" for c in s)


@dataclass
class Chunk:
    content: str
    file_path: str


def should_skip_file(file_path: str) -> bool:
    path = Path(file_path)
    return path.suffix == ".lock" or path.name in {".gitignore", ".python-version"}


def chunk_repo(repo_url: str, chunk_size: int = CHUNK_SIZE) -> list[Chunk]:
    """Ingest a repo and split it into chunks."""
    _summary, _tree, content = ingest(repo_url)
    lines = content.splitlines(keepends=True)

    chunks: list[Chunk] = []
    file_header = ""
    current_lines: list[str] = []
    current_chars = 0
    i = 0

    while i < len(lines):
        if (
            _is_separator(lines[i])
            and i + 2 < len(lines)
            and lines[i + 1].startswith("FILE: ")
            and _is_separator(lines[i + 2])
        ):
            if current_lines and not should_skip_file(file_header):
                chunks.append(
                    Chunk(content="".join(current_lines), file_path=file_header)
                )

            current_lines = []
            current_chars = 0
            file_header = lines[i + 1].rstrip("\r\n").removeprefix("FILE: ")
            i += 3
            continue

        current_lines.append(lines[i])
        current_chars += len(lines[i])

        if current_chars >= chunk_size:
            if not should_skip_file(file_header):
                chunks.append(
                    Chunk(content="".join(current_lines), file_path=file_header)
                )
            current_lines = []
            current_chars = 0

        i += 1

    if current_lines and not should_skip_file(file_header):
        chunks.append(Chunk(content="".join(current_lines), file_path=file_header))

    return chunks


def main(repo_url: str):
    chunks = chunk_repo(repo_url)
    print(f"Pushing {len(chunks)} chunks to Hatchet...")

    for i, chunk in enumerate(chunks):
        link_chunk_workflow.run_no_wait(
            input=ChunkPayload(
                repo_url=repo_url, file_path=chunk.file_path, content=chunk.content
            )
        )
        print(
            f"\n===== CHUNK [{i + 1}/{len(chunks)}] {chunk.file_path} ({len(chunk.content)} chars) ====="
        )
        print(chunk.content)
        print(f"===== END CHUNK [{i + 1}/{len(chunks)}] =====")

    print("Done.")


if __name__ == "__main__":
    repo_url = (
        sys.argv[1] if len(sys.argv) > 1 else "https://github.com/sadosystems/spandrel"
    )
    main(repo_url)
