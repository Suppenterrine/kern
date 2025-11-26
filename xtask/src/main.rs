use regex::Regex;
use std::{env, error::Error, fs, path::PathBuf};

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    // Erwartet: bump version <major|minor|patch>
    if args.len() != 3 || args[0] != "bump" || args[1] != "version" {
        eprintln!("Usage: cargo xtask bump version [major|minor|patch]");
        std::process::exit(1);
    }

    let kind = args[2].as_str();

    let project_root = find_project_root()?;

    let cargo_toml_path = project_root.join("Cargo.toml");
    let mut cargo_toml = fs::read_to_string(&cargo_toml_path)?;
    let (old_version, new_version) = bump_version_in_cargo_toml(&mut cargo_toml, kind)?;
    fs::write(&cargo_toml_path, &cargo_toml)?;

    let readme_path = project_root.join("README.md");
    if readme_path.exists() {
        let mut readme = fs::read_to_string(&readme_path)?;
        bump_version_in_readme(&mut readme, &old_version, &new_version)?;
        fs::write(&readme_path, &readme)?;
    } else {
        eprintln!("warning: README.md not found, only Cargo.toml was updated");
    }

    println!("version bumped ({kind}): {old_version} -> {new_version}");

    Ok(())
}

fn find_project_root() -> Result<PathBuf, Box<dyn Error>> {
    let mut dir = env::current_dir()?;
    loop {
        if dir.join("Cargo.toml").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err("could not find Cargo.toml in any parent directory".into());
        }
    }
}

fn bump_version_in_cargo_toml(
    content: &mut String,
    kind: &str,
) -> Result<(String, String), Box<dyn Error>> {
    // Sucht nach: version = "x.y.z" in der [package]-Sektion
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

    let new_content = re.replace(content, format!(r#"version = "{new_version}""#));
    *content = new_content.to_string();

    Ok((old_version, new_version))
}

fn bump_version_in_readme(
    content: &mut String,
    old: &str,
    new: &str,
) -> Result<(), Box<dyn Error>> {
    // Erwartetes Format im README (mit oder ohne Markdown-Fettdruck):
    // STATUS: Stabil | VERSION: 0.3.1
    // **STATUS:** Stabil | **VERSION:** 0.3.1
    
    // Versuche zuerst das Markdown-Format
    let old_line_md = format!("**STATUS:** Stabil | **VERSION:** {old}");
    let new_line_md = format!("**STATUS:** Stabil | **VERSION:** {new}");
    
    if content.contains(&old_line_md) {
        *content = content.replace(&old_line_md, &new_line_md);
        return Ok(());
    }
    
    // Fallback auf einfaches Format
    let old_line = format!("STATUS: Stabil | VERSION: {old}");
    let new_line = format!("STATUS: Stabil | VERSION: {new}");
    
    if content.contains(&old_line) {
        *content = content.replace(&old_line, &new_line);
        return Ok(());
    }

    // Wenn keine Version gefunden wird, klar meckern
    Err(format!(
        "did not find version line in README.md. Expected one of:\n\
        '**STATUS:** Stabil | **VERSION:** {old}'\n\
        or\n\
        'STATUS: Stabil | VERSION: {old}'"
    )
    .into())
}
