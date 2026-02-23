use std::path::Path;

use crate::errors::SkillError;
use crate::parser::{find_skill_md, read_properties};

/// Escape XML special characters.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Generate the `<available_skills>` XML block for inclusion in agent prompts.
pub fn to_prompt(skill_dirs: &[&Path]) -> Result<String, SkillError> {
    if skill_dirs.is_empty() {
        return Ok("<available_skills>\n</available_skills>".into());
    }

    let mut lines = vec!["<available_skills>".to_string()];

    for &skill_dir in skill_dirs {
        let skill_dir = skill_dir
            .canonicalize()
            .map_err(|e| SkillError::parse(format!("Cannot resolve path: {}", e)))?;

        let props = read_properties(&skill_dir)?;

        lines.push("<skill>".into());
        lines.push("<name>".into());
        lines.push(escape_xml(&props.name));
        lines.push("</name>".into());
        lines.push("<description>".into());
        lines.push(escape_xml(&props.description));
        lines.push("</description>".into());

        let skill_md_path = find_skill_md(&skill_dir).unwrap();
        lines.push("<location>".into());
        lines.push(skill_md_path.to_string_lossy().into_owned());
        lines.push("</location>".into());

        lines.push("</skill>".into());
    }

    lines.push("</available_skills>".into());

    Ok(lines.join("\n"))
}
