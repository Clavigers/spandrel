"""Custom MCP tool that wraps the spandrel CLI."""

import json
import subprocess
from claude_agent_sdk import tool, create_sdk_mcp_server

SPAN_SCHEMA = {
    "type": "object",
    "properties": {
        "file_path": {"type": "string", "description": "Path to the file (relative to repo root or cwd)"},
        "start_line_number": {"type": "integer", "description": "1-indexed start line"},
        "end_line_number": {"type": "integer", "description": "1-indexed end line"},
        "text": {"type": "string", "description": "Exact substring to match within the line range (preferred over columns)"},
        "start_column": {"type": "integer", "description": "1-indexed start column (use if not providing text)"},
        "end_column": {"type": "integer", "description": "1-indexed end column (use if not providing text)"},
    },
    "required": ["file_path", "start_line_number", "end_line_number"],
}


@tool(
    "spandrel_link",
    "Create a semantic link between two regions of source code using the spandrel CLI. "
    "Connects a 'here' span to one or more 'there' target spans. Use CONNECTS for related, "
    "consistent spans. Use CONTRADICTS when docs and code disagree.",
    {
        "type": "object",
        "properties": {
            "link_type": {
                "type": "string",
                "enum": ["CONNECTS", "CONTRADICTS"],
                "description": "CONNECTS for related spans, CONTRADICTS for disagreements",
            },
            "here": SPAN_SCHEMA,
            "there": {
                "type": "array",
                "items": SPAN_SCHEMA,
                "minItems": 1,
                "description": "One or more target spans that the 'here' span links to",
            },
        },
        "required": ["link_type", "here", "there"],
    },
)
async def spandrel_link(args: dict) -> dict:
    link_json = json.dumps(args)
    result = subprocess.run(
        ["spandrel", "link", link_json],
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        return {
            "content": [{"type": "text", "text": result.stderr.strip() or result.stdout.strip()}],
            "is_error": True,
        }

    return {
        "content": [{"type": "text", "text": result.stdout.strip()}],
    }


spandrel_server = create_sdk_mcp_server(
    name="spandrel",
    version="1.0.0",
    tools=[spandrel_link],
)
