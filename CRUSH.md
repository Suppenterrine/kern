# KERN Project Developer Guide

## Build Commands
```bash
# Build project
cargo build

# Build with optimizations
cargo build --release

# Run the application
cargo run -- [arguments]

# Run with specific features
cargo run --release -- [arguments]
```

## Test Commands
```bash
# Run all tests
cargo test

# Run a specific test file
cargo test --test <test_name>

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release
```

## Lint/Format Commands
```bash
# Check code formatting
cargo fmt -- --check

# Auto-format code
cargo fmt

# Run clippy linter
cargo clippy

# Clippy with errors only
cargo clippy -- -D warnings
```

## Code Style Guidelines

### Imports
- Group imports in order: std, external crates, local modules
- Use explicit imports rather than glob imports
- Place imports at the top of the file

### Formatting
- Use `cargo fmt` for consistent formatting
- Max line width: 100 characters
- Use 4 spaces for indentation (no tabs)

### Types
- Use explicit types for public functions
- Prefer `u32` for numeric calculations
- Use `&str` over `String` when possible for function parameters

### Naming Conventions
- Use snake_case for functions and variables
- Use PascalCase for structs and enums
- Use SCREAMING_SNAKE_CASE for constants

### Error Handling
- Use `Result<T, E>` for functions that can fail
- Prefer `expect()` with descriptive messages over `unwrap()`
- Handle errors gracefully and provide meaningful error messages

### Module Organization
- Keep related functionality in the same module
- Use `pub mod` for publicly accessible modules
- Separate core logic from command-line interface

# ideas
Core Additions

Cipher abstraction

Trait for consistent interface (name, short, calculate, calculate_from_numbers)

Simple + advanced cipher implementations (Ordinal, Fibonacci, Chaldean, etc.)

Reduction helpers

Pure functions for letter → number mapping, digit sums, master number checks

Centralized, no duplication in ciphers

Flow Engine

FlowSpec AST

Supports Cipher, Parallel(Vec), Pipe(Vec), Matrix(Vec<Vec>)

Recursive evaluation with depth tracking

Evaluator

Parallel execution with Rayon (for parallel and matrix stages)

Depth counter (level 1 = base cipher, deeper levels for pipes/matrices)

Lineage metadata (trace of how a value was produced)

CLI Layer

Basic usage

--cipher, --all, --select (interactive selection via dialoguer)

Flow mode

--flow "(ord,pri)|fib" → custom pipelines via DSL

Optional: JSON flow specification for API compatibility

SPEKTRA mode

--spektra flag → inject results into LLM prompt template

--api-key handling (never stored, client only)

Output

Matrix rendering with tabled/comfy-table

Verbose/trace mode showing depth & lineage

Server Layer

Endpoints

/reduce → basic cipher results

/flow → evaluate FlowSpec JSON

/spektra → evaluate FlowSpec + generate SPEKTRA prompt via LLM

Stateless: API key passed per request, never stored

SPEKTRA Layer

Prompt builder

Fill LLM template with cipher results

Return structured meta-output (meta-term, meta-description, metaphor, axes)

Clean separation: no cipher logic, just transformation of results → prompt

Metadata & Depth

EvalResult struct

value: u32

cipher: String

depth: usize (evaluation level)

lineage: Vec<String> (evaluation path)

Enables rich tracing, recursive reporting, and story-like outputs

UX & Extensibility

DSL for flows (with pest/nom/chumsky parser)

JSON API for flows (machine-friendly, matches DSL semantics)

Future-proof stages (matrix ops, custom reductions, determinant, etc.)

Unified Engine: CLI & Server use the same FlowSpec evaluator