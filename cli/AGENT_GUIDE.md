# Spandrel Agent Guide

This document explains what Spandrel is and how you (an AI agent) should use it.

## What Spandrel does

Spandrel maintains a database of semantic links between two regions of source code in a repository. Each link connects a **here** span to a **there** span — two specific locations in the codebase that are meaningfully related to each other.

Think of it as a wikipedia-style cross-reference system for code. You are building a knowledge graph over a codebase, one link at a time.

## Why this exists

Semantic vector embeddings are bad at associating documentation with code. The text in a design doc and the text in the implementation it describes look completely different when embedded — different vocabulary, different structure, different style. Embeddings miss these connections.

Spandrel solves this by letting you explicitly declare: "this region of text is related to that region of code" (or vice versa). You are the bridge between documentation and implementation.

## Link types

Every link has a type:

### CONNECTS

This is the common case. Use this when two spans are genuinely related and consistent with each other. Examples:

- A section of an architecture doc describes a system, and you find the implementation of that system.
- A README explains how to configure something, and you find the code that reads that configuration.
- A comment block describes an algorithm, and the function below implements it.
- An API doc describes an endpoint's behavior, and the handler implements that behavior.

Most links you create should be CONNECTS.

### CONTRADICTS

Use this when two spans are related but **disagree** with each other. The documentation says one thing, but the code does another. Examples:

- The design doc says "we use WebSockets to push updates to clients" but the implementation uses HTTP polling.
- The README says a feature is enabled by default, but the code defaults it to off.
- A comment says "this function returns null on failure" but it actually throws an exception.
- An API doc describes a field as required, but the code treats it as optional.

CONTRADICTS links are valuable — they surface drift between documentation and implementation that needs to be fixed. Do not avoid creating them. If you find a contradiction, link it.

## How to use the CLI

### Creating a link

```bash
spandrel link '<json>'
```

The JSON schema:

```json
{
  "link_type": "CONNECTS" or "CONTRADICTS",
  "here": {
    "file_path": "path/to/file",
    "start_line_number": 1,
    "start_column": 1,
    "end_line_number": 10,
    "end_column": 40
  },
  "there": {
    "file_path": "path/to/other/file",
    "start_line_number": 5,
    "start_column": 1,
    "end_line_number": 20,
    "end_column": 80
  }
}
```

File paths are relative to the repo root or your current working directory — the CLI normalizes them. The CLI will reject the link if either file doesn't exist or if a span exceeds the file's line count.

You do not provide the repo or commit. The CLI detects the current repository (via `gh`) and the current HEAD commit (via `git`) automatically. Every link is scoped to a specific repo and commit.

### Viewing a link

```bash
spandrel pretty_print <uuid>
```

This outputs markdown showing the actual content of both spans. If you are in the same repo at the same commit, it reads from local files. Otherwise, it fetches the content from GitHub's API.

## Guidelines for creating good links

1. **Be precise with spans.** Link the specific region that matters, not an entire file. A 5-line span that pinpoints the relevant code is better than a 200-line span that includes the whole module.

2. **Prefer documentation-to-code links.** The primary value of Spandrel is connecting docs to implementation. Code-to-code links are fine when meaningful, but doc-to-code is the core use case.

3. **Always create CONTRADICTS links when you find them.** Do not silently skip contradictions. The whole point is to surface these.

4. **One relationship per link.** If a doc section relates to three different code locations, create three links, not one link with a huge span.

5. **Both directions are valid.** "here" and "there" have no inherent ordering. A link from docs to code and a link from code to docs are equally valid.
