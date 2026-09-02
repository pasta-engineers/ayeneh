# Agent Guide

## Project Overview

`mirror-tester` is a Rust 2021 Cargo workspace that benchmarks package
registry mirrors, reports their performance, and installs Python packages
with mirror fallback. It supports PyPI and npm and exposes both a scripted CLI
and an interactive terminal UI.

The main runtime behavior is networked: a benchmark resolves and downloads a
real package archive from each configured mirror, writes it briefly to the OS
temporary directory, deletes it, and records the latency. It never installs or
executes the benchmark package.

## Repository Layout

```text
crates/
  core/       Shared library: mirrors, benchmarking, pip, reports, scheduler
  cli/        `mirror-cli` command parsing and command handlers
  tui/        `mirror-tui` Ratatui/Crossterm interface
data/         Static mirror configuration for PyPI and npm
reports/      Generated JSON reports (ignored by Git, except `.gitkeep`)
.github/      CI and release workflows
ARCHITECTURE.md
Cargo.toml
Makefile
README.md
```

Read `ARCHITECTURE.md` for the detailed data flow and benchmark semantics.

## Crate Boundaries

- `mirror-core` owns reusable domain and application logic. Its modules are
  `app`, `benchmark`, `mirror`, `pip`, `report`, and `scheduler`.
- `mirror-cli` owns `clap` parsing, command dispatch, stdout tables, and CLI
  error/exit behavior. Its command handlers are exposed from `src/lib.rs` so
  the TUI can reuse them.
- `mirror-tui` owns terminal setup, keyboard events, UI state, and rendering.
  It wraps `mirror_core::app::App` and can also dispatch CLI commands when
  arguments are supplied.

Dependency rules:

- Keep `mirror-core` independent of `ratatui` and `crossterm`.
- Keep `mirror-cli` independent of `ratatui` and `crossterm`.
- Put shared behavior in `mirror-core`, not in either frontend.
- Keep TUI-only state and rendering inside `crates/tui`.

## Important Runtime Details

### Mirror Configuration

`data/pypi.json` and `data/npm.json` each contain:

- `package`: the sample package downloaded during benchmarks
- `mirrors`: an ordered list of mirror base URLs

Mirror discovery is static; the application does not scrape or discover new
mirrors automatically. The TUI's add-mirror action appends directly to the
selected JSON configuration.

Data directory resolution uses this order:

1. `MIRROR_DATA_DIR`, when set
2. A `data/` directory found by walking upward from the executable
3. A `data/` directory found by walking upward from the current directory
4. `data/` relative to the current directory

Reports use the same lookup strategy and honor `MIRROR_REPORTS_DIR` first.

### Benchmarking

- Each mirror gets three sequential attempts.
- Each attempt has a 15-second `reqwest` client timeout.
- PyPI resolves an archive from the PEP 503 simple index.
- npm resolves `dist.tarball` from the registry's `/latest` metadata.
- Successful attempt latencies are averaged; failed attempts reduce the
  success rate but do not abort the complete run.
- Mirrors with zero successful attempts are marked timed out and sort last.
- Results are otherwise sorted by average latency ascending.

Because benchmarks make real network requests, failures can be caused by DNS,
connectivity, mirror metadata, rate limits, or the mirror's download host. Do
not treat a network-dependent test failure as a deterministic code regression
without checking the failure details.

### Reports and Generated Files

`mirror-cli report` and `mirror-cli schedule` write JSON files under
`reports/YYYY-MM-DD_HH-MM.json`. Report JSON is ignored by Git; do not force
generated reports into commits unless explicitly requested.

## Common Commands

Example commands can be found in the [Makefile](Makefile).

## Development Workflow

1. Inspect the relevant crate and existing module boundaries before changing
   code.
2. For shared behavior, implement and test it in `crates/core` first, then
   keep frontend changes limited to presentation and command wiring.
3. Format and validate Rust changes with:

   ```bash
   cargo fmt --all -- --check
   cargo check --workspace
   cargo test --workspace
   ```

## CI and Releases

- `.github/workflows/rust.yml` builds and tests on pushes and pull requests to
  `main`.
- `.github/workflows/release.yml` builds release binaries for Ubuntu and
  Windows when a GitHub release is published.
- Release artifacts are `mirror-cli` and `mirror-tui` (with `.exe` on
  Windows).

Keep workspace dependency versions in the root `Cargo.toml` and use workspace
dependencies from individual crate manifests where possible.

## Change Guidance

- Preserve the CLI's non-interactive behavior and meaningful exit codes.
- Preserve TUI terminal cleanup, including raw-mode and alternate-screen
  restoration on normal exit.
- Do not move network, filesystem, or subprocess logic into UI rendering code.
- When changing the benchmark algorithm, update `ARCHITECTURE.md` and any
  user-facing README behavior that has changed.
- When changing the JSON shape of mirror configuration or reports, update the
  sample files/documentation and consider compatibility with existing files.
