//! Project automation.
//!
//! `Cargo.toml` is the single source of truth for the version. Every other file
//! that repeats it is a *derived* copy and is written by `sync-version` — never
//! by hand. Bump with `cargo set-version <VERSION>`, then run
//! `cargo xtask sync-version`.

use regex::Regex;
use std::{env, error::Error, fs, path::{Path, PathBuf}};

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

const USAGE: &str = "\
Usage:
  cargo xtask sync-version [--check]   Write the Cargo.toml version into all derived files
  cargo xtask bump version <major|minor|patch>

`sync-version --check` writes nothing and exits non-zero if any file has drifted,
which makes it usable as a CI gate.";

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let root = find_project_root()?;

    match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        ["sync-version"] => sync_version(&root, false),
        ["sync-version", "--check"] => sync_version(&root, true),
        ["bump", "version", kind] => bump(&root, kind),
        _ => {
            eprintln!("{USAGE}");
            std::process::exit(1);
        }
    }
}

/// A file that repeats the version, and how to find it there.
struct DerivedVersion {
    /// Path relative to the project root
    path: &'static str,
    /// Must capture the version in group 1 and nothing else
    pattern: &'static str,
    /// Replacement with `{version}` standing in for the new value
    template: &'static str,
    /// Human-readable description for error messages
    what: &'static str,
}

const DERIVED: &[DerivedVersion] = &[
    DerivedVersion {
        path: "api/kern.definition.yaml",
        // Only the `info:` block's version, not the OpenAPI dialect version.
        pattern: r#"(?m)^(\s*)version:\s*(\d+\.\d+\.\d+)\s*$"#,
        // ${1} preserves the YAML indentation; dropping it would break the file.
        template: "${1}version: {version}",
        what: "OpenAPI info.version",
    },
    DerivedVersion {
        path: "README.md",
        pattern: r#"\*\*VERSION:\*\*\s*(\d+\.\d+\.\d+)"#,
        template: "**VERSION:** {version}",
        what: "README version badge",
    },
];

/// Writes the Cargo.toml version into every derived file, or reports drift.
///
/// Every target is resolved before anything is written, so a missing or
/// unmatched file aborts without leaving the repo half-updated.
fn sync_version(root: &Path, check_only: bool) -> Result<(), Box<dyn Error>> {
    let version = cargo_version(root)?;
    println!("source of truth: Cargo.toml = {version}");

    // Phase 1: resolve everything, collect failures rather than dying at the
    // first one, so the user sees every problem at once.
    let mut planned: Vec<(PathBuf, String, &DerivedVersion, String)> = Vec::new();
    let mut problems: Vec<String> = Vec::new();

    for target in DERIVED {
        let path = root.join(target.path);
        if !path.exists() {
            problems.push(format!("{}: file not found ({})", target.what, target.path));
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let re = Regex::new(target.pattern)?;

        let Some(caps) = re.captures(&content) else {
            problems.push(format!(
                "{}: no version found in {} (pattern: {})",
                target.what, target.path, target.pattern
            ));
            continue;
        };

        let found = caps
            .iter()
            .skip(1)
            .flatten()
            .map(|m| m.as_str())
            .find(|s| s.contains('.'))
            .unwrap_or_default()
            .to_string();

        let replacement = target.template.replace("{version}", &version);
        let updated = re
            .replace(&content, replacement.as_str())
            .to_string();

        planned.push((path, updated, target, found));
    }

    if !problems.is_empty() {
        return Err(format!(
            "cannot sync, nothing was written:\n  - {}",
            problems.join("\n  - ")
        )
        .into());
    }

    // Phase 2: report or write.
    let mut drifted = Vec::new();
    for (path, updated, target, found) in &planned {
        if found == &version {
            println!("  ok      {} ({found})", target.what);
            continue;
        }
        drifted.push(format!("{} was {found}, expected {version}", target.what));

        if check_only {
            println!("  DRIFT   {} ({found} != {version})", target.what);
        } else {
            fs::write(path, updated)?;
            println!("  updated {} ({found} -> {version})", target.what);
        }
    }

    if check_only && !drifted.is_empty() {
        return Err(format!(
            "{} file(s) out of sync:\n  - {}\nrun `cargo xtask sync-version`",
            drifted.len(),
            drifted.join("\n  - ")
        )
        .into());
    }

    if drifted.is_empty() {
        println!("all derived versions already in sync");
    }

    Ok(())
}

/// Bumps Cargo.toml, then syncs everything derived from it.
fn bump(root: &Path, kind: &str) -> Result<(), Box<dyn Error>> {
    let cargo_toml_path = root.join("Cargo.toml");
    let mut cargo_toml = fs::read_to_string(&cargo_toml_path)?;
    let (old, new) = bump_version_in_cargo_toml(&mut cargo_toml, kind)?;
    fs::write(&cargo_toml_path, &cargo_toml)?;
    println!("version bumped ({kind}): {old} -> {new}");

    sync_version(root, false)
}

fn cargo_version(root: &Path) -> Result<String, Box<dyn Error>> {
    let content = fs::read_to_string(root.join("Cargo.toml"))?;
    let re = Regex::new(r#"(?m)^version\s*=\s*"(\d+\.\d+\.\d+)""#)?;
    Ok(re
        .captures(&content)
        .ok_or("no 'version = \"x.y.z\"' line found in Cargo.toml")?[1]
        .to_string())
}

fn find_project_root() -> Result<PathBuf, Box<dyn Error>> {
    let mut dir = env::current_dir()?;
    loop {
        // The workspace root, not a member crate.
        if dir.join("Cargo.toml").exists() && dir.join("api").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err("could not find the project root (a directory with Cargo.toml and api/)".into());
        }
    }
}

fn bump_version_in_cargo_toml(
    content: &mut String,
    kind: &str,
) -> Result<(String, String), Box<dyn Error>> {
    let re = Regex::new(r#"(?m)^version\s*=\s*"(\d+)\.(\d+)\.(\d+)""#)?;
    let caps = re
        .captures(content)
        .ok_or("no 'version = \"x.y.z\"' line found in Cargo.toml")?;

    let major: u64 = caps[1].parse()?;
    let minor: u64 = caps[2].parse()?;
    let patch: u64 = caps[3].parse()?;

    let (new_major, new_minor, new_patch) = match kind {
        "major" => (major + 1, 0, 0),
        "minor" => (major, minor + 1, 0),
        "patch" => (major, minor, patch + 1),
        _ => return Err("kind must be one of: major, minor, patch".into()),
    };

    let old_version = format!("{major}.{minor}.{patch}");
    let new_version = format!("{new_major}.{new_minor}.{new_patch}");

    *content = re
        .replace(content, format!(r#"version = "{new_version}""#))
        .to_string();

    Ok((old_version, new_version))
}
