# Ayeneh

A terminal tool that installs packages from mirrors, benchmarks, and reports results, all with both simple TUI and CLI.

## Features

- Benchmarks mirrors and ranks them by dowload speed
- Install packages from mirrors. Switch to a different mirror if fails.
- Interactive keyboard-only TUI built with `ratatui`

## Installation

Pickup desired binary from the [releases](https://github.com/erfan-rfmhr/mirror-tester/releases)
Or build from source:

- Make sure you have Rust (stable) installed via [rustup](https://rustup.rs)

```bash
git clone <https://github.com/erfan-rfmhr/mirror-tester>
cd mirror-tester
make
# or
cargo build
```

## Usage

The project ships two binaries:

* `mirror-cli` — non-interactive CLI, suitable for CI pipelines, Docker
  images, and shell scripting.
* `mirror-tui` — interactive Ratatui application.

Get help:

```bash
mirror-cli --help
mirror-cli pip --help
mirror-cli npm --help
```

### Environment Variables

Runtime behavior can be tuned without rebuilding:

| Variable                          | Meaning                                | Default                   |
|-----------------------------------|----------------------------------------|---------------------------|
| `MIRROR_DATA_DIR`                 | Directory holding `pypi.json`/`npm.json` | `./data` |
| `MIRROR_REPORTS_DIR`              | Directory for generated reports        | auto-detected `reports/`  |
| `MIRROR_TIMEOUT_SECS`             | Per-attempt HTTP timeout (seconds)     | `15`                      |
| `MIRROR_ATTEMPTS`                 | Benchmark attempts per mirror          | `3`                       |
| `MIRROR_SCHEDULE_INTERVAL_SECS`   | Delay between scheduler cycles (seconds) | `3600`                    |

The numeric variables fall back to their default if unset or given a
non-numeric, zero, or negative value.

Launch the TUI:

```bash
mirror-tui
```

Install python packages from mirrors:

```bash
mirror-cli pip install <package>
```
or install from requirements file:

```bash
mirror-cli pip install -r requirements.txt
```

Install npm packages from mirrors:

```bash
mirror-cli npm install <package>
```

Run a one-off benchmark from the command line:

```bash
mirror-cli run pypi
mirror-cli run npm
```

Generate a JSON report for both package managers:

```bash
mirror-cli report
```

Run the hourly scheduler (keeps benchmarking forever):

```bash
mirror-cli schedule
```

## Example TUI

```
+------------------------------------------------+
| Mirror Benchmark                                |
+------------------------------------------------+

Package Manager:

> PyPI
  npm

--------------------------------------------
Results

Mirror                     Avg(ms)   Success
pypi.org                   120       100%
mirror1                    180       100%
mirror2                    timeout   0%
--------------------------------------------

Fastest Mirror:
https://pypi.org/simple/

[q] Quit   [Enter] Run Benchmark   [up/down] Switch
```

## Example Report

`reports/2026-07-12_14-30.json`:

```json
{
  "package_manager": "pypi",
  "generated_at": "2026-07-12T14:30:00+00:00",
  "results": [
    { "mirror": "https://pypi.org/simple/", "latency": 120, "success": 100.0 },
    { "mirror": "https://mirror.example1/simple/", "latency": 180, "success": 100.0 }
  ],
  "best": "https://pypi.org/simple/"
}
```

## Mirror Lists

`pypi.json` and `npm.json` each define the sample `package` to download during
benchmarking and the list of `mirrors` to test it against. By default, the
program reads them from `./data`:

```json
{
    "package": "requests",
    "mirrors": [
        "https://pypi.org/simple/",
        "https://pypi.devneeds.ir/simple/",
        "https://package-mirror.liara.ir/repository/pypi/"
    ]
}
```

Set `MIRROR_DATA_DIR` when the registry files are stored elsewhere:

```bash
MIRROR_DATA_DIR=/opt/mirror/registries mirror-cli run pypi
```

Edit these JSON files to add or remove mirrors, or to change the package used
for benchmarking. Mirror lists are configured statically — no network
scraping is performed to discover them.

### Docker

A Python-based image is provided that ships the prebuilt `mirror-cli` binary
alongside the mirror registry config, so it can be used directly as a base for
installing packages or benchmarking. Build it from the repository root (a
release binary must already exist):

```bash
cargo build --release -p mirror-cli
docker build -f docker/Dockerfile.python -t mirror-cli .
```

Run it, or extend it in your own Dockerfile to install packages from mirrors:

```bash
docker run --rm mirror-cli mirror-cli pip install requests
```

```dockerfile
FROM mirror-cli
RUN mirror-cli pip install -r requirements.txt
```

A Node.js-based image is provided for npm. Build it the same way (a release
binary must already exist):

```bash
docker build -f docker/Dockerfile.npm -t mirror-cli-npm .
```

Run it, or extend it in your own Dockerfile to install packages from npm
mirrors:

```bash
docker run --rm mirror-cli-npm mirror-cli npm install lodash
```

```dockerfile
FROM mirror-cli-npm
RUN mirror-cli npm install lodash express
```
