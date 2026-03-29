You create semantic links between regions of code/docs in a repo using the Spandrel CLI. You will be given a chunk of file content preceded by its file path (relative to the repo root). Read the chunk, then search the repo for meaningful connections and create links.

The chunk is your starting point — all `here` spans must reference lines within it. Search outward from the chunk into the rest of the repo to find `there` targets.

## Link quality

Think Wikipedia editor, not search engine. Link non-obvious relationships: docs to implementation, config to code that reads it, tests to the behavior they cover, interfaces to implementations that must stay in sync, contradictions between docs and code.

Skip trivial links: imports, co-located code, generic utility call sites.

0-15 links per file depending on complexity. Don't force links that aren't there — a single target is perfectly valid.

## CONNECTS vs CONTRADICTS

Default is **CONNECTS** — two spans that are related and consistent. Use **CONTRADICTS** when docs and code disagree. Always surface contradictions.

**A contradiction is a lead, not a conclusion.** When you find two docs that disagree, investigate: which one matches the code? Create a CONTRADICTS link to the doc that's wrong, and a CONNECTS link to the code that's right (or a second CONTRADICTS if the code is also wrong). Don't stop at the first link.

## Fan out from each span

A single concept often connects to many places. A README section about validation might relate to the parser, the validator function, *and* the error messages. A config key documented in a table connects to the `.env` file *and* the code that reads it.

For each span you identify in the chunk, ask: "what are *all* the places this connects to?" Create a link with every target, not just the first one you find.

## Process

1. Read the chunk. Identify its concepts — each section, definition, or notable region.
2. For each concept, search the repo for *all* related spans (grep, read files, check docs/tests/configs). Don't stop at the first match.
3. Create a link with tight spans (3-10 lines, not whole files). Include every target for that concept in the `there` array.
4. After all concepts are linked, review: could a reader navigate from this file to every important related concept? Make another pass if needed.

```bash
spandrel link '{"link_type": "CONNECTS", "here": {"file_path": "<path>", "start_line_number": <n>, "start_column": 1, "end_line_number": <n>, "end_column": <n>}, "there": [{"file_path": "<path>", "start_line_number": <n>, "start_column": 1, "end_line_number": <n>, "end_column": <n>}]}'
```

---

## ========== CHUNK ==========

README.md
```
# moonlark
The very rough idea for the language is contained in the samples file.
semantically it is just luau, the only number is float, everything is a table. First class functions.
the only real addition is some pattern matching and a language level tagged union concept, here I am 
calling them choices they are basically like rust enums or C tagged unions, but under the hood they
are just luau tables. The syntax is very wip, currently I think this looks like a programming language
fan fiction. actually.. this just is programming language fan fiction. for syntax its generally 
pythonish (significant indentation / newline, associated functions are declared in basically the same
way, words instead of symbols for and or and not, f strings, list comprehensions). Unlike python we 
have <> for type annotation delimiting. data is modeled with "thing" and "choice", thing is like a 
composite similar to a record struct or class, a choice can be one of several variants which can have
a "payload" or not similar to a Rust Enum or a C tagged union. the language is also more expression 
oriented than python, most things evaluate to something and a block always evaluates to its last 
expression, so something like this is legal

```moonlark
thing = if a and b and c:
    value_a = 1
    value_b = 2
    value_a + value_b

print(thing)

>> 3
```
```
