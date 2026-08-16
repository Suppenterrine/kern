# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

KERN is a Rust-based numerology system that performs "resonant reduction" (Quersummen/digital root calculation) on text, numbers, and dates. It decodes symbolic patterns using various cipher systems (Ordinal, Pythagorean, Chaldean, etc.) and projects numerological meanings from a YAML knowledge base.

The project consists of:
- **CLI tool** (`kern`): Terminal interface for numerological calculations
- **Web server** (`kern-server`): REST API exposing the same functionality
- **Core library**: Shared logic used by both binaries

## Build & Development Commands

### Building
```bash
# Build CLI (default binary)
cargo build --release

# Build the web server
cargo build --release --bin kern-server

# Build both binaries
cargo build --release --bins
```

### Running
```bash
# Run CLI directly
cargo run -- [ARGS]

# Examples:
cargo run -- Wickfeld
cargo run -- --lookup -d 0..7
cargo run -- --cipher all "test"
cargo run -- word1 -v word2 -c py,ch word3 -l

# Run web server
cargo run --bin kern-server

# Or run the compiled binary
./target/release/kern [ARGS]
./target/release/kern-server
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test file
cargo test bedeutungen_tests
cargo test kern_core

# Run tests with output
cargo test -- --nocapture
```

### Docker
```bash
# Build Docker image
docker build -t kern-server .

# Run container
docker run --rm -p 3000:3000 --name kern-server kern-server

# Or use Docker Hub image
docker pull 24biteggplant/kern-server:latest
docker run -d -p 3000:3000 --name kern-server 24biteggplant/kern-server:latest
```

## Architecture

### Core Concepts

**Pipeline System**: The architecture uses a pipeline/flow engine where inputs flow through multiple steps, each producing results that are stored in a flow context "memory". This enables complex operations like aggregating totals and looking up all values from previous steps.

**Cipher Trait**: All cipher implementations follow the `Cipher` trait (src/core/ciphers/mod.rs) which defines:
- `name()`: Returns the cipher's canonical name
- `char_to_value(ch: char) -> u32`: Maps characters to numeric values

Available ciphers include: Ordinal, Reverse Ordinal, Pythagorean, Reverse Pythagorean, Chaldean, Agrippa, Primes, Fibonacci, Squares, Cubes, and Septenary.

**Operation Types**: The flow engine supports different operations (defined in src/lib.rs:Operation):
- `Reduce`: Standard numerological reduction
- `DateReduce`: Reduction of date values
- `AggregateTotal`: Sum all previous results and reduce
- `Lookup`: Collect all values and their sources for bedeutungen.yaml lookup
- `Custom(String)`: Placeholder for extensions

**Flags**: All flags are global — they apply to the entire execution.

Position matters, inconsistently: flags are only recognised **before** the
first input. After it they are silently reduced as if they were words
(`kern hello --verbose` reduces the string "--verbose"). The sole exceptions
are `-t/--total` and `-l/--lookup`, which `parse_pipeline_tokens` picks out by
hand and which therefore work anywhere.

Earlier versions of this file described "local flags" bound to pipeline
positions (`word1 -v word2 -c py,ch`). **That does not exist** — `-v` is
reduced as a word, and `-c` is not defined as a short form at all. Measured
behaviour and a one-line fix are written up in
`docs/proposals/cli-argument-order.md`.

### Key Files

- `src/main.rs`: CLI entry point, argument parsing, output formatting
- `src/bin/kern-server.rs`: Axum-based REST API server
- `src/lib.rs`: Public API exposing core types and functions
- `src/core/flow.rs`: Pipeline engine, FlowContext, and execution logic
- `src/core/ciphers/mod.rs`: Cipher trait and registry of all cipher implementations
- `src/core/ciphers/*.rs`: Individual cipher implementations
- `src/core/utils.rs`: Character conversion utilities
- `bedeutungen.yaml`: Embedded numerological meanings (1-9, 11, 22, 33) with "Lichtseite" and "Schattenseite" — German base file, also holds the `rtap_*` prompts
- `bedeutungen.en.yaml`: English meanings plus the English `rtap_*` prompts
- `bedeutungen.fr.yaml`: French meanings (numeric keys only — no prompts)
- `spektra_prompt.txt` / `spektra_prompt.en.txt`: German and English SPEKTRA templates

### Tooling and versioning

**Never edit a version number by hand.** `Cargo.toml` is the single source of
truth; every other occurrence is derived and written by the tool.

```bash
cargo xtask check              # all consistency checks, writes nothing
cargo set-version 2.1.0        # bump the source of truth (cargo-edit)
cargo xtask sync-version       # write it into README + OpenAPI spec
cargo xtask check-error-codes  # ErrorCode::API vs. the OpenAPI spec
cargo xtask check-release      # release note present? tag still free?
```

Releasing is `gh workflow run release.yml`. The workflow reads the version from
`Cargo.toml`, derives the tag, takes the body from
`docs/release-notes/<version>.md`, creates a **draft**, builds, verifies the
artifacts arrived, and only then publishes. A failed build leaves a draft
rather than a published release with missing artifacts. Never create a release
by hand — the workflow no longer triggers on one.

`cargo set-version` alone is not enough — it only knows `Cargo.toml`. Add new
derived locations to the `DERIVED` table in `xtask/src/main.rs` rather than
updating them by hand.

`cargo xtask check` runs in CI (`rust.yml`) on every PR to master, and gates
`release.yml`: if it or `check-tag` fails, no binaries and no Docker image are
published.

Full detail: `docs/reference/tooling.md`.

### Documentation layout

| Where | What |
|-------|------|
| `README.md` | Usage |
| `docs/PRINCIPLES.md` | Binding design rules |
| `docs/reference/` | How each module is built, and why |
| `docs/TODO.md` | Open work |

**Docs travel with the code** (PRINCIPLES §7): changing a module means updating
its reference in the same change. `docs/reference/README.md` lists which modules
are still undocumented — extend it rather than letting the reference look
complete when it is not.

### Project principles

`docs/PRINCIPLES.md` holds the binding design rules: no silent fallbacks, single
source of truth, CLI/server parity, exhaustive matches over catch-all arms,
docs travel with the code. **Read it before** changing error handling, fallback
behaviour, anything version related, or adding a capability to only one binary.

### Data Flow

1. **Input parsing**: CLI args or API requests → parsed inputs + pipeline steps
2. **Pipeline construction**: Steps are created with operations and flags
3. **Execution**: Pipeline.run() iterates through steps, executes operations, stores results in FlowContext.memory
4. **Result aggregation**: Results are collected in ResultSet and returned
5. **Output formatting**: CLI prints tables/verbose traces, API returns JSON

### Master Numbers

Special treatment for "Masterzahlen": 11, 22, 33 are NOT reduced further. This is checked at multiple points in the reduction logic (src/lib.rs:217-221, :238).

## REST API Endpoints

The server (port 3000) exposes:
- `GET /` — compact service descriptor (health-check shaped; **not** the help)
- `GET /help` — full endpoint listing with examples
- `GET /reduce?input=word1,word2&debug=true&length=true&onlyTotal=false`
- `GET /lookup/:number?parts=light|shadow|both&lang=en|de|fr`
- `GET /lookup?numbers=1,2,3&parts=light|shadow|both&lang=en|de|fr`
- `GET /date?range=-3..7&debug=false&lang=en|de|fr`
- `GET /spektra?word=Love&lang=en|de`
- `GET /rtap?part=1|2|both&lang=en|de`

The OpenAPI contract lives in `api/kern.definition.yaml` and must be updated
alongside any endpoint change.

## Important Development Notes

### Adding New Ciphers

1. Create new file in `src/core/ciphers/` implementing the `Cipher` trait
2. Add module declaration in `src/core/ciphers/mod.rs`
3. Export the cipher type with `pub use`
4. Add a `CipherDescriptor` entry to the `CIPHERS` array with name, short code, description, and factory function

### Debugging

Set environment variable to dump ResultSet as JSON:
```powershell
# PowerShell
$env:KERN_DUMP_RESULTSET=1

# Then run
cargo run -- your args
```

### Bedeutungen YAML Structure

The `bedeutungen.yaml` file is embedded at compile time and loaded into a `HashMap<u32, Bedeutung>`. Each entry has:
- `bedeutung`: Main meaning text
- `lichtseite`: Positive aspects
- `schattenseite`: Shadow/negative aspects

### Localization

**English is the default** (`Lang::default() == Lang::En`) — the API is an
international interface. German requires an explicit `lang=de`.

Coverage differs per content type:

| Content | Languages | Anything else |
|---------|-----------|---------------|
| Meanings (`/lookup`, `/date`) | en, de, fr | 400 `unsupported_language` |
| SPEKTRA + RTAP prompts | en, de | 400 `language_not_available` |
| Error messages | English only | n/a — `lang` selects content, not protocol |

**There are no silent fallbacks anywhere** — see docs/PRINCIPLES.md §1. A
language that is valid but unavailable for a resource is rejected, never
substituted. Do not "helpfully" add a fallback; it is a deliberate design rule.

- `Lang` (src/lib.rs) is the language type. `Lang::parse` matches the primary
  subtag case-insensitively, so `en-US` → `En`.
- `load_bedeutungen_lang(lang)` loads one language, `load_all_bedeutungen()` all
  of them (the server keeps every language in `AppState` instead of re-parsing
  YAML per request).
- **`load_bedeutungen()` and `lookup()` are pinned to German**, deliberately not
  to `Lang::default()`. They serve German-only surfaces; wiring them to the
  default would silently put English meanings into a German prompt.
- `Lang::PROMPT_LANGS` / `Lang::has_prompts()` encode "prompts exist in de and
  en only". `prompt_assets()` (spektra.rs) and `rtap_source()` (lib.rs) match
  every `Lang` variant exhaustively and return `None` for languages without
  prompts, so a new language forces an explicit decision at compile time.
- An unknown code is **rejected**: HTTP 400 `unsupported_language` on the
  server, exit code 1 on the CLI.
- Responses carry a `lang` field naming the language used, which is always the
  language that was requested.

### Errors

Every error from **both** binaries carries a stable `code` plus human-readable
`error` prose. Clients branch on `code`; the prose may be reworded.

- Codes are the `ErrorCode` enum in `src/lib.rs`, shared by CLI and server. It
  is an enum, not free strings, so an undeclared code cannot be emitted.
- `ErrorCode::API` is the subset the HTTP API can return and must match
  `api/kern.definition.yaml` exactly — enforced by `cargo xtask check-error-codes`.
- `ErrorCode::ALL` additionally holds CLI-only codes (`invalid_arguments`).
- Server: construct via `bad_request` / `server_error`. CLI: via `output_error`.
- Error text is always English regardless of `lang`.

Full detail: `docs/reference/error-codes.md`.

### Adding a language

1. Create `bedeutungen.<code>.yaml` with the same numeric keys as `bedeutungen.yaml`
2. Add the variant to `Lang` — `code()`, `ALL`, `missing_meaning()` and
   `prompt_lang()` are exhaustive matches, so the compiler points at every spot
3. Add the `include_str!` arm in `bedeutungen_source()`
4. Decide whether the prompts get translated too — the exhaustive matches in
   `prompt_assets()` and `rtap_source()` will not compile until you do. `None`
   means the prompt endpoints reject that language. Otherwise add
   `spektra_prompt.<code>.txt`, the `rtap_*` keys, a `SpektraLabels` const, and
   the variant in `Lang::PROMPT_LANGS`
5. Run `cargo test` — the suite checks completeness, key parity across
   languages, that translations are not copies of the German source, and that
   each SPEKTRA template matches its placeholder labels

### Edition 2024

This project uses Rust edition 2024 (Cargo.toml). Ensure you're using a nightly toolchain that supports this edition.
