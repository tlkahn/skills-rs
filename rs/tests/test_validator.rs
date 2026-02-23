mod common;

use skills_ref::validator::validate;

#[test]
fn test_valid_skill() {
    let (_tmp, dir) = common::create_skill_dir(
        "my-skill",
        "---\nname: my-skill\ndescription: A test skill\n---\n# My Skill\n",
    );
    let errors = validate(&dir);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn test_nonexistent_path() {
    let tmp = tempfile::TempDir::new().unwrap();
    let errors = validate(&tmp.path().join("nonexistent"));
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("does not exist"), "got: {}", errors[0]);
}

#[test]
fn test_not_a_directory() {
    let tmp = tempfile::TempDir::new().unwrap();
    let file_path = tmp.path().join("file.txt");
    std::fs::write(&file_path, "test").unwrap();
    let errors = validate(&file_path);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("Not a directory"), "got: {}", errors[0]);
}

#[test]
fn test_missing_skill_md() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path().join("my-skill");
    std::fs::create_dir_all(&dir).unwrap();
    let errors = validate(&dir);
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0].contains("Missing required file: SKILL.md"),
        "got: {}",
        errors[0]
    );
}

#[test]
fn test_invalid_name_uppercase() {
    let (_tmp, dir) = common::create_skill_dir(
        "MySkill",
        "---\nname: MySkill\ndescription: A test skill\n---\nBody\n",
    );
    let errors = validate(&dir);
    assert!(errors.iter().any(|e| e.contains("lowercase")), "got: {:?}", errors);
}

#[test]
fn test_name_too_long() {
    let long_name = "a".repeat(70);
    let content = format!("---\nname: {}\ndescription: A test skill\n---\nBody\n", long_name);
    let (_tmp, dir) = common::create_skill_dir(&long_name, &content);
    let errors = validate(&dir);
    assert!(
        errors.iter().any(|e| e.contains("exceeds") && e.contains("character limit")),
        "got: {:?}",
        errors
    );
}

#[test]
fn test_name_leading_hyphen() {
    let (_tmp, dir) = common::create_skill_dir(
        "-my-skill",
        "---\nname: -my-skill\ndescription: A test skill\n---\nBody\n",
    );
    let errors = validate(&dir);
    assert!(
        errors.iter().any(|e| e.contains("cannot start or end with a hyphen")),
        "got: {:?}",
        errors
    );
}

#[test]
fn test_name_consecutive_hyphens() {
    let (_tmp, dir) = common::create_skill_dir(
        "my--skill",
        "---\nname: my--skill\ndescription: A test skill\n---\nBody\n",
    );
    let errors = validate(&dir);
    assert!(
        errors.iter().any(|e| e.contains("consecutive hyphens")),
        "got: {:?}",
        errors
    );
}

#[test]
fn test_name_invalid_characters() {
    let (_tmp, dir) = common::create_skill_dir(
        "my_skill",
        "---\nname: my_skill\ndescription: A test skill\n---\nBody\n",
    );
    let errors = validate(&dir);
    assert!(
        errors.iter().any(|e| e.contains("invalid characters")),
        "got: {:?}",
        errors
    );
}

#[test]
fn test_name_directory_mismatch() {
    let (_tmp, dir) = common::create_skill_dir(
        "wrong-name",
        "---\nname: correct-name\ndescription: A test skill\n---\nBody\n",
    );
    let errors = validate(&dir);
    assert!(
        errors.iter().any(|e| e.contains("must match skill name")),
        "got: {:?}",
        errors
    );
}

#[test]
fn test_unexpected_fields() {
    let (_tmp, dir) = common::create_skill_dir(
        "my-skill",
        "---\nname: my-skill\ndescription: A test skill\nunknown_field: should not be here\n---\nBody\n",
    );
    let errors = validate(&dir);
    assert!(
        errors.iter().any(|e| e.contains("Unexpected fields")),
        "got: {:?}",
        errors
    );
}

#[test]
fn test_valid_with_all_fields() {
    let (_tmp, dir) = common::create_skill_dir(
        "my-skill",
        "---\nname: my-skill\ndescription: A test skill\nlicense: MIT\nmetadata:\n  author: Test\n---\nBody\n",
    );
    let errors = validate(&dir);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn test_allowed_tools_accepted() {
    let (_tmp, dir) = common::create_skill_dir(
        "my-skill",
        "---\nname: my-skill\ndescription: A test skill\nallowed-tools: Bash(jq:*) Bash(git:*)\n---\nBody\n",
    );
    let errors = validate(&dir);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn test_i18n_chinese_name() {
    let (_tmp, dir) = common::create_skill_dir(
        "技能",
        "---\nname: 技能\ndescription: A skill with Chinese name\n---\nBody\n",
    );
    let errors = validate(&dir);
    assert_eq!(errors, Vec::<String>::new(), "got: {:?}", errors);
}

#[test]
fn test_i18n_russian_with_hyphens() {
    let (_tmp, dir) = common::create_skill_dir(
        "мой-навык",
        "---\nname: мой-навык\ndescription: A skill with Russian name\n---\nBody\n",
    );
    let errors = validate(&dir);
    assert_eq!(errors, Vec::<String>::new(), "got: {:?}", errors);
}

#[test]
fn test_i18n_russian_lowercase() {
    let (_tmp, dir) = common::create_skill_dir(
        "навык",
        "---\nname: навык\ndescription: A skill with Russian lowercase name\n---\nBody\n",
    );
    let errors = validate(&dir);
    assert_eq!(errors, Vec::<String>::new(), "got: {:?}", errors);
}

#[test]
fn test_i18n_russian_uppercase() {
    let (_tmp, dir) = common::create_skill_dir(
        "НАВЫК",
        "---\nname: НАВЫК\ndescription: A skill with Russian uppercase name\n---\nBody\n",
    );
    let errors = validate(&dir);
    assert!(
        errors.iter().any(|e| e.contains("lowercase")),
        "got: {:?}",
        errors
    );
}

#[test]
fn test_description_too_long() {
    let long_desc = "x".repeat(1100);
    let content = format!(
        "---\nname: my-skill\ndescription: {}\n---\nBody\n",
        long_desc
    );
    let (_tmp, dir) = common::create_skill_dir("my-skill", &content);
    let errors = validate(&dir);
    assert!(
        errors.iter().any(|e| e.contains("exceeds") && e.contains("1024")),
        "got: {:?}",
        errors
    );
}

#[test]
fn test_valid_compatibility() {
    let (_tmp, dir) = common::create_skill_dir(
        "my-skill",
        "---\nname: my-skill\ndescription: A test skill\ncompatibility: Requires Python 3.11+\n---\nBody\n",
    );
    let errors = validate(&dir);
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn test_compatibility_too_long() {
    let long_compat = "x".repeat(550);
    let content = format!(
        "---\nname: my-skill\ndescription: A test skill\ncompatibility: {}\n---\nBody\n",
        long_compat
    );
    let (_tmp, dir) = common::create_skill_dir("my-skill", &content);
    let errors = validate(&dir);
    assert!(
        errors.iter().any(|e| e.contains("exceeds") && e.contains("500")),
        "got: {:?}",
        errors
    );
}

#[test]
fn test_nfkc_normalization() {
    // Decomposed form: 'cafe' + combining acute accent (U+0301)
    let decomposed_name = "cafe\u{0301}";
    // Composed form for directory name
    let composed_name = "caf\u{00e9}";

    let tmp = tempfile::TempDir::new().unwrap();
    let skill_dir = tmp.path().join(composed_name);
    std::fs::create_dir_all(&skill_dir).unwrap();
    let content = format!(
        "---\nname: {}\ndescription: A test skill\n---\nBody\n",
        decomposed_name
    );
    std::fs::write(skill_dir.join("SKILL.md"), &content).unwrap();

    let errors = validate(&skill_dir);
    assert_eq!(errors, Vec::<String>::new(), "got: {:?}", errors);
}
