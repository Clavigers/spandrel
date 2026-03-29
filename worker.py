"""Hatchet worker: picks up chunk tasks and runs the spandrel linking agent."""

import os
import subprocess
from pathlib import Path
from urllib.parse import urlparse

from dotenv import load_dotenv
from pydantic import BaseModel
from hatchet_sdk import Hatchet, Context
from claude_agent_sdk import ClaudeSDKClient, ClaudeAgentOptions

load_dotenv()

hatchet = Hatchet()

PROMPT_FILE = Path(__file__).parent / "cli" / "prompt_1.md"
REPOS_DIR = Path(__file__).parent / "repos"


def load_system_prompt() -> str:
    with open(PROMPT_FILE, "r", encoding="utf-8") as f:
        return f.read().strip()


SYSTEM_PROMPT = load_system_prompt()


def clone_or_reuse(repo_url: str) -> Path:
    """Clone the repo if not already cached, return the local path."""
    parsed = urlparse(repo_url)
    # e.g. /gillespie-alex/C_Data_Structures/tree/master -> gillespie-alex/C_Data_Structures
    parts = parsed.path.strip("/").split("/")
    if len(parts) >= 2:
        repo_name = f"{parts[0]}_{parts[1]}"
    else:
        repo_name = parts[0] or "unknown"

    # Reconstruct a cloneable URL (strip /tree/branch etc.)
    clone_url = f"https://{parsed.netloc}/{parts[0]}/{parts[1]}.git" if len(parts) >= 2 else repo_url

    clone_path = REPOS_DIR / repo_name
    if clone_path.exists():
        return clone_path

    REPOS_DIR.mkdir(parents=True, exist_ok=True)
    print(f"Cloning {clone_url} -> {clone_path}")
    subprocess.run(
        ["git", "clone", "--depth", "1", clone_url, str(clone_path)],
        check=True,
    )
    return clone_path


class ChunkPayload(BaseModel):
    repo_url: str
    file_path: str
    content: str


link_chunk_workflow = hatchet.workflow(name="LinkChunk", input_validator=ChunkPayload)


@link_chunk_workflow.task()
async def link_chunk(input: ChunkPayload, ctx: Context) -> dict:
    repo_path = clone_or_reuse(input.repo_url)
    user_message = f"`{input.file_path}`\n\n```\n{input.content}\n```"

    options = ClaudeAgentOptions(
        permission_mode="bypassPermissions",
        system_prompt=SYSTEM_PROMPT,
        allowed_tools=["Bash", "Read", "Grep", "Glob"],
        model="opus",
        thinking={"type": "enabled", "budget_tokens": 10000},
        cwd=repo_path,
        env={
            "DATABASE_URL": os.environ.get("DATABASE_URL", ""),
            "PATH": os.environ.get("PATH", ""),
        },
    )

    links_created = 0

    async with ClaudeSDKClient(options=options) as client:
        await client.query(prompt=user_message)

        async for msg in client.receive_response():
            if type(msg).__name__ == "AssistantMessage":
                for block in msg.content:
                    if hasattr(block, "text"):
                        print(block.text, end="", flush=True)
                        if "Link created:" in block.text:
                            links_created += block.text.count("Link created:")

    print(f"\n[done] {input.file_path} — {links_created} links")
    return {"file_path": input.file_path, "links_created": links_created}


def main():
    worker = hatchet.worker(name="spandrel-linker")
    worker.register_workflow(link_chunk_workflow)
    print("Spandrel worker listening for chunks...")
    worker.start()


if __name__ == "__main__":
    main()
