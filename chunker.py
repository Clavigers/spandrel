"""
Code chunker — splits a code string into smaller chunks by line count.
"""


def chunk_code(code: str, chunk_size: int) -> list[str]:
    """Split *code* into chunks of at most *chunk_size* lines.

    Args:
        code:        The source code string to split.
        chunk_size:  Maximum number of lines per chunk.

    Returns:
        A list of non-empty string chunks.
    """
    if chunk_size <= 0:
        raise ValueError("chunk_size must be a positive integer")

    lines = code.splitlines(keepends=True)
    return [
        "".join(lines[i : i + chunk_size])
        for i in range(0, len(lines), chunk_size)
    ]


if __name__ == "__main__":
    import sys
    import argparse

    parser = argparse.ArgumentParser(description="Split code into chunks.")
    parser.add_argument("file", nargs="?", help="Source file (default: stdin)")
    parser.add_argument("-n", "--chunk-size", type=int, default=40, help="Chunk size (default: 40)")
    parser.add_argument("-o", "--output", help="Output file (default: stdout)")
    args = parser.parse_args()

    if args.file:
        with open(args.file) as f:
            source = f.read()
    else:
        source = sys.stdin.read()

    chunks = chunk_code(source, args.chunk_size)
    output = ""
    for i, chunk in enumerate(chunks, 1):
        output += f"Chunk {i}:\n{chunk}\n"

    if args.output:
        with open(args.output, "w") as f:
            f.write(output)
        print(f"Written to {args.output}")
    else:
        print(output, end="")
