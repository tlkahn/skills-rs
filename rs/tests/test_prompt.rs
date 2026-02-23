mod common;

use std::path::Path;

use skills_ref::prompt::to_prompt;

#[test]
fn test_empty_list() {
    let result = to_prompt(&[]).unwrap();
    assert_eq!(result, "<available_skills>\n</available_skills>");
}

#[test]
fn test_single_skill() {
    let (_tmp, dir) = common::create_skill_dir(
        "my-skill",
        "---\nname: my-skill\ndescription: A test skill\n---\nBody\n",
    );
    let result = to_prompt(&[dir.as_path()]).unwrap();
    assert!(result.contains("<available_skills>"));
    assert!(result.contains("</available_skills>"));
    assert!(result.contains("<name>\nmy-skill\n</name>"));
    assert!(result.contains("<description>\nA test skill\n</description>"));
    assert!(result.contains("<location>"));
    assert!(result.contains("SKILL.md"));
}

#[test]
fn test_multiple_skills() {
    let tmp = tempfile::TempDir::new().unwrap();

    let skill_a = tmp.path().join("skill-a");
    std::fs::create_dir_all(&skill_a).unwrap();
    std::fs::write(
        skill_a.join("SKILL.md"),
        "---\nname: skill-a\ndescription: First skill\n---\nBody\n",
    )
    .unwrap();

    let skill_b = tmp.path().join("skill-b");
    std::fs::create_dir_all(&skill_b).unwrap();
    std::fs::write(
        skill_b.join("SKILL.md"),
        "---\nname: skill-b\ndescription: Second skill\n---\nBody\n",
    )
    .unwrap();

    let dirs: Vec<&Path> = vec![skill_a.as_path(), skill_b.as_path()];
    let result = to_prompt(&dirs).unwrap();
    assert_eq!(result.matches("<skill>").count(), 2);
    assert_eq!(result.matches("</skill>").count(), 2);
    assert!(result.contains("skill-a"));
    assert!(result.contains("skill-b"));
}

#[test]
fn test_special_chars_escaped() {
    let (_tmp, dir) = common::create_skill_dir(
        "special-skill",
        "---\nname: special-skill\ndescription: \"Use <foo> & <bar> tags\"\n---\nBody\n",
    );
    let result = to_prompt(&[dir.as_path()]).unwrap();
    assert!(result.contains("&lt;foo&gt;"), "got: {}", result);
    assert!(result.contains("&amp;"), "got: {}", result);
    assert!(result.contains("&lt;bar&gt;"), "got: {}", result);
    assert!(!result.contains("<foo>"), "got: {}", result);
    assert!(!result.contains("<bar>"), "got: {}", result);
}
