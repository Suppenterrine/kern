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

**Flags System**: There are two levels of flags:
- **Global flags**: Apply to entire execution (e.g., `--verbose`, `--cipher`)
- **Local flags**: Apply to specific pipeline positions (e.g., `word1 -v word2 -c py,ch`)

### Key Files

- `src/main.rs`: CLI entry point, argument parsing, output formatting
- `src/bin/kern-server.rs`: Axum-based REST API server
- `src/lib.rs`: Public API exposing core types and functions
- `src/core/flow.rs`: Pipeline engine, FlowContext, and execution logic
- `src/core/ciphers/mod.rs`: Cipher trait and registry of all cipher implementations
- `src/core/ciphers/*.rs`: Individual cipher implementations
- `src/core/utils.rs`: Character conversion utilities
- `bedeutungen.yaml`: Embedded numerological meanings (1-9, 11, 22, 33) with "Lichtseite" and "Schattenseite"

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
- `GET /reduce?input=word1,word2&debug=true&length=true&onlyTotal=false`
- `GET /lookup/:number?parts=light|shadow|both`
- `GET /lookup?numbers=1,2,3&parts=light|shadow|both`
- `GET /date?range=-3..7&debug=false`

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

### Edition 2024

This project uses Rust edition 2024 (Cargo.toml). Ensure you're using a nightly toolchain that supports this edition.
