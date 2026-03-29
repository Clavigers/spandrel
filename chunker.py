from gitingest import ingest
import asyncio
from dataclasses import dataclass


def _is_separator(line: str) -> bool:
    s = line.rstrip("\r\n")
    return len(s) >= 4 and all(c == "=" for c in s)


@dataclass
class Chunk:
    content: str
    file_path: str


async def send_to_claude_agent(chunk: Chunk) -> None:
    """Stub — replace with your real Claude dispatch."""
    print(f"-----→ [{chunk.file_path}] sending chunk ({len(chunk.content)} chars):\n{chunk.content}")


async def producer(repo_url: str, queue: asyncio.Queue, chunk_size: int = 300):
    summary, tree, content = await asyncio.to_thread(ingest, repo_url)
    lines = content.splitlines(keepends=True)

    file_header = ""
    current_lines: list[str] = []
    current_chars = 0
    i = 0

    while i < len(lines):
        # Detect 3-line file separator: ====, File: path, ====
        if (
            _is_separator(lines[i])
            and i + 2 < len(lines)
            and lines[i + 1].startswith("FILE: ")
            and _is_separator(lines[i + 2])
        ):
            # Flush current chunk before switching files
            if current_lines:
                await queue.put(Chunk(content="".join(current_lines), file_path=file_header))
                current_lines = []
                current_chars = 0

            file_header = lines[i + 1].rstrip("\r\n").removeprefix("FILE: ")
            i += 3
            continue

        current_lines.append(lines[i])
        current_chars += len(lines[i])

        if current_chars >= chunk_size:
            await queue.put(Chunk(content="".join(current_lines), file_path=file_header))
            current_lines = []
            current_chars = 0

        i += 1

    # Flush any remaining lines
    if current_lines:
        await queue.put(Chunk(content="".join(current_lines), file_path=file_header))

    await queue.put(None)


# async def producer(data: str, queue: asyncio.Queue, chunk_size: int = 200) -> None:
#     for i in range(0, len(data), chunk_size):
#         chunk = data[i : i + chunk_size]
#         await queue.put(chunk)
#     await queue.put(None)  # signal done


async def consumer(queue: asyncio.Queue[Chunk | None]) -> None:
    while True:
        chunk = await queue.get()
        if chunk is None:
            break
        await send_to_claude_agent(chunk)


async def main(source: str) -> None:
    queue: asyncio.Queue[Chunk | None] = asyncio.Queue(maxsize=4)
    await asyncio.gather(producer(source, queue), consumer(queue))


if __name__ == "__main__":
    import sys
    from pathlib import Path

    # path = sys.argv[1] if len(sys.argv) > 1 else __file__
    path = 'https://github.com/gillespie-alex/C_Data_Structures/tree/master'
    # print(path)
    # source = Path(path).read_text()
    # print(source)
    asyncio.run(main(path))
