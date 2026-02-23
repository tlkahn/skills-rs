mod common;

use skills_ref::errors::SkillError;
use skills_ref::parser::{find_skill_md, parse_frontmatter, read_properties};

// ── parse_frontmatter tests ──

#[test]
fn test_valid_frontmatter() {
    let content = "---\nname: my-skill\ndescription: A test skill\n---\n# My Skill\n\nInstructions here.\n";
    let (metadata, body) = parse_frontmatter(content).unwrap();
    assert_eq!(
        metadata["name"],
        serde_yaml::Value::String("my-skill".into())
    );
    assert_eq!(
        metadata["description"],
        serde_yaml::Value::String("A test skill".into())
    );
    assert!(body.contains("# My Skill"));
}

#[test]
fn test_missing_frontmatter() {
    let content = "# No frontmatter here";
    let err = parse_frontmatter(content).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("must start with YAML frontmatter"),
        "got: {}",
        msg
    );
}

#[test]
fn test_unclosed_frontmatter() {
    let content = "---\nname: my-skill\ndescription: A test skill\n";
    let err = parse_frontmatter(content).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("not properly closed"), "got: {}", msg);
}

#[test]
fn test_invalid_yaml() {
    let content = "---\nname: [invalid\ndescription: broken\n---\nBody here\n";
    let err = parse_frontmatter(content).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Invalid YAML"), "got: {}", msg);
}

#[test]
fn test_non_dict_frontmatter() {
    let content = "---\n- just\n- a\n- list\n---\nBody\n";
    let err = parse_frontmatter(content).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("must be a YAML mapping"), "got: {}", msg);
}

// ── find_skill_md tests ──

#[test]
fn test_find_prefers_uppercase() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path().join("my-skill");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("SKILL.md"), "uppercase").unwrap();
    std::fs::write(dir.join("skill.md"), "lowercase").unwrap();

    let result = find_skill_md(&dir);
    assert!(result.is_some());
    assert_eq!(result.unwrap().file_name().unwrap(), "SKILL.md");
}

#[test]
fn test_find_accepts_lowercase() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path().join("my-skill");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("skill.md"), "lowercase").unwrap();

    let result = find_skill_md(&dir);
    assert!(result.is_some());
    let name = result
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_lowercase();
    assert_eq!(name, "skill.md");
}

#[test]
fn test_find_returns_none() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path().join("my-skill");
    std::fs::create_dir_all(&dir).unwrap();

    let result = find_skill_md(&dir);
    assert!(result.is_none());
}

// ── read_properties tests ──

#[test]
fn test_read_valid_skill() {
    let (_tmp, dir) = common::create_skill_dir(
        "my-skill",
        "---\nname: my-skill\ndescription: A test skill\nlicense: MIT\n---\n# My Skill\n",
    );
    let props = read_properties(&dir).unwrap();
    assert_eq!(props.name, "my-skill");
    assert_eq!(props.description, "A test skill");
    assert_eq!(props.license.as_deref(), Some("MIT"));
}

#[test]
fn test_read_with_metadata() {
    let (_tmp, dir) = common::create_skill_dir(
        "my-skill",
        "---\nname: my-skill\ndescription: A test skill\nmetadata:\n  author: Test Author\n  version: 1.0\n---\nBody\n",
    );
    let props = read_properties(&dir).unwrap();
    assert_eq!(props.metadata.get("author").unwrap(), "Test Author");
    assert_eq!(props.metadata.get("version").unwrap(), "1.0");
}

#[test]
fn test_missing_skill_md() {
    let tmp = tempfile::TempDir::new().unwrap();
    let err = read_properties(tmp.path()).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("SKILL.md not found"), "got: {}", msg);
}

#[test]
fn test_missing_name() {
    let (_tmp, dir) = common::create_skill_dir(
        "my-skill",
        "---\ndescription: A test skill\n---\nBody\n",
    );
    let err = read_properties(&dir).unwrap_err();
    match &err {
        SkillError::Validation { message, .. } => {
            assert!(
                message.contains("Missing required field") && message.contains("name"),
                "got: {}",
                message
            );
        }
        other => panic!("expected Validation, got: {:?}", other),
    }
}

#[test]
fn test_missing_description() {
    let (_tmp, dir) = common::create_skill_dir(
        "my-skill",
        "---\nname: my-skill\n---\nBody\n",
    );
    let err = read_properties(&dir).unwrap_err();
    match &err {
        SkillError::Validation { message, .. } => {
            assert!(
                message.contains("Missing required field") && message.contains("description"),
                "got: {}",
                message
            );
        }
        other => panic!("expected Validation, got: {:?}", other),
    }
}

#[test]
fn test_read_lowercase_skill_md() {
    let (_tmp, dir) = common::create_skill_dir_with_filename(
        "my-skill",
        "skill.md",
        "---\nname: my-skill\ndescription: A test skill\n---\n# My Skill\n",
    );
    let props = read_properties(&dir).unwrap();
    assert_eq!(props.name, "my-skill");
    assert_eq!(props.description, "A test skill");
}

#[test]
fn test_read_with_allowed_tools() {
    let (_tmp, dir) = common::create_skill_dir(
        "my-skill",
        "---\nname: my-skill\ndescription: A test skill\nallowed-tools: Bash(jq:*) Bash(git:*)\n---\nBody\n",
    );
    let props = read_properties(&dir).unwrap();
    assert_eq!(props.allowed_tools.as_deref(), Some("Bash(jq:*) Bash(git:*)"));
    let d = props.to_json_value();
    assert_eq!(d["allowed-tools"], "Bash(jq:*) Bash(git:*)");
}
