# Agent Guide

## Project Overview

`ayeneh`: Rust 2021 Cargo workspace. Benchmarks package registry mirrors, installs packages with mirror fallback.

## Repository Layout

```text
crates/
  core/       Shared library
  cli/        CLI command parsing and command handlers
  tui/        TUI Ratatui/Crossterm interface
data/         Static mirror configuration
.github/      CI workflows
ARCHITECTURE.md
Cargo.toml
Makefile
README.md
```

Read `ARCHITECTURE.md` for data flow + benchmark semantics.

## Crate Boundaries

- `ayeneh-core`: reusable domain + app logic. Modules: `app`, `benchmark`, `config`, `mirror`, `pip`, `report`, `scheduler`.
- `ayeneh-cli`: `clap` parsing, command dispatch, stdout tables, CLI error/exit behavior. Handlers exposed from `src/lib.rs` for TUI reuse.
- `ayeneh-tui`: terminal setup, keyboard events, UI state, rendering. Wraps `ayeneh_core::app::App`; can dispatch CLI commands when args supplied.

Dependency rules:

- Keep `ayeneh-core` independent of `ratatui` + `crossterm`.
- Keep `ayeneh-cli` independent of `ratatui` + `crossterm`.
- Shared behavior in `ayeneh-core`, not frontends.
- TUI-only state + rendering in `crates/tui`.

## Important Runtime Details

### Mirror Configuration

Read how configuration is loaded from [README](./README.md#Configuration)


## Common Commands

Example commands in the [Makefile](Makefile).

## Development Workflow

1. Inspect relevant crate + module boundaries before changes.
2. Shared behavior: implement + test in `crates/core` first; keep frontend changes to presentation + command wiring.
3. Format + validate Rust changes:

   ```bash
   cargo fmt --all -- --check
   cargo check --workspace
   cargo test --workspace
   ```

## Change Guidance

When changing the benchmark algorithm, update `ARCHITECTURE.md` and any user-facing README behavior that has changed.

## graphify

If the project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships, follow this rule:

When the user types `/graphify`, use the installed graphify skill or instructions before doing anything else.

Rules:
- For codebase questions, first run `graphify query "<question>"` when graphify-out/graph.json exists. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- Dirty graphify-out/ files are expected after hooks or incremental updates; dirty graph files are not a reason to skip graphify. Only skip graphify if the task is about stale or incorrect graph output, or the user explicitly says not to use it.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).
