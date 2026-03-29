from gitingest import ingest
import asyncio



async def send_to_claude_agent(chunk: str) -> None:
    """Stub — replace with your real Claude dispatch."""
    print(f"-----→ sending chunk ({len(chunk)} chars):\n{chunk}")


async def producer(repo_url: str, queue: asyncio.Queue, chunk_size: int = 200):
    summary, tree, content = await asyncio.to_thread(ingest, repo_url)
    lines = content.splitlines(keepends=True)
    for i in range(0, len(lines), chunk_size):
        chunk = "".join(lines[i : i + chunk_size])
        await queue.put(chunk)
    await queue.put(None)

# async def producer(data: str, queue: asyncio.Queue, chunk_size: int = 200) -> None:
#     for i in range(0, len(data), chunk_size):
#         chunk = data[i : i + chunk_size]
#         await queue.put(chunk)
#     await queue.put(None)  # signal done


async def consumer(queue: asyncio.Queue) -> None:
    while True:
        chunk = await queue.get()
        if chunk is None:
            break
        await send_to_claude_agent(chunk)


async def main(source: str) -> None:
    queue: asyncio.Queue[str | None] = asyncio.Queue(maxsize=4)
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
