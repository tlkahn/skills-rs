use std::path::PathBuf;
use tempfile::TempDir;

/// Create a skill directory with a SKILL.md file inside a temp dir.
/// Returns (TempDir guard, path to the skill directory).
pub fn create_skill_dir(dir_name: &str, skill_md_content: &str) -> (TempDir, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let skill_dir = tmp.path().join(dir_name);
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), skill_md_content).unwrap();
    (tmp, skill_dir)
}

/// Create a skill directory with a custom filename (e.g., "skill.md" lowercase).
#[allow(dead_code)]
pub fn create_skill_dir_with_filename(
    dir_name: &str,
    filename: &str,
    content: &str,
) -> (TempDir, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let skill_dir = tmp.path().join(dir_name);
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join(filename), content).unwrap();
    (tmp, skill_dir)
}
