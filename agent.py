"""Spandrel linking agent using Claude Agent SDK."""

import asyncio
import os
import sys
from pathlib import Path
from dotenv import load_dotenv
from claude_agent_sdk import ClaudeSDKClient, ClaudeAgentOptions

from spandrel_tool import spandrel_server

load_dotenv()

PROMPT_FILE = Path(__file__).parent / "cli" / "prompt_1.md"


def load_prompt() -> str:
    """Load the system prompt from prompt_1.md."""
    with open(PROMPT_FILE, "r", encoding="utf-8") as f:
        return f.read().strip()


async def run(user_input: str):
    """Run the spandrel agent on a single input."""
    if not os.environ.get("ANTHROPIC_API_KEY"):
        print("\nError: ANTHROPIC_API_KEY not found.")
        print("Set it in a .env file or export it in your shell.")
        return

    system_prompt = load_prompt()

    options = ClaudeAgentOptions(
        permission_mode="bypassPermissions",
        system_prompt=system_prompt,
        mcp_servers={"spandrel": spandrel_server},
        allowed_tools=["mcp__spandrel__spandrel_link", "Read", "Grep", "Glob"],
        model="sonnet",
    )

    async with ClaudeSDKClient(options=options) as client:
        await client.query(prompt=user_input)

        async for msg in client.receive_response():
            if type(msg).__name__ == "AssistantMessage":
                for block in msg.content:
                    if hasattr(block, "text"):
                        print(block.text, end="", flush=True)

        print()


async def chat():
    """Interactive chat loop."""
    if not os.environ.get("ANTHROPIC_API_KEY"):
        print("\nError: ANTHROPIC_API_KEY not found.")
        print("Set it in a .env file or export it in your shell.")
        return

    system_prompt = load_prompt()

    options = ClaudeAgentOptions(
        permission_mode="bypassPermissions",
        system_prompt=system_prompt,
        mcp_servers={"spandrel": spandrel_server},
        allowed_tools=["mcp__spandrel__spandrel_link", "Read", "Grep", "Glob"],
        model="sonnet",
    )

    print("\n" + "=" * 50)
    print("  Spandrel Linking Agent")
    print("=" * 50)
    print("\nPaste a file chunk and I'll find semantic links.")
    print("Type 'exit' to quit.\n")

    async with ClaudeSDKClient(options=options) as client:
        while True:
            try:
                user_input = input("\nYou: ").strip()
            except (EOFError, KeyboardInterrupt):
                break

            if not user_input or user_input.lower() in ("exit", "quit", "q"):
                break

            await client.query(prompt=user_input)

            print("\nAgent: ", end="")
            async for msg in client.receive_response():
                if type(msg).__name__ == "AssistantMessage":
                    for block in msg.content:
                        if hasattr(block, "text"):
                            print(block.text, end="", flush=True)
            print()

    print("\nGoodbye!")


if __name__ == "__main__":
    if len(sys.argv) > 1:
        # Single-shot mode: pass the argument as the prompt
        asyncio.run(run(" ".join(sys.argv[1:])))
    else:
        # Interactive mode
        asyncio.run(chat())
