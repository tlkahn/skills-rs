use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, Subcommand};

use skills_ref::errors::SkillError;

#[derive(Parser)]
#[command(name = "skills-ref", version, about = "Reference library for Agent Skills")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a skill directory
    Validate {
        /// Path to the skill directory (or SKILL.md file)
        skill_path: PathBuf,
    },
    /// Read and print skill properties as JSON
    #[command(name = "read-properties")]
    ReadProperties {
        /// Path to the skill directory (or SKILL.md file)
        skill_path: PathBuf,
    },
    /// Generate <available_skills> XML for agent prompts
    #[command(name = "to-prompt")]
    ToPrompt {
        /// Paths to skill directories (or SKILL.md files)
        skill_paths: Vec<PathBuf>,
    },
}

fn is_skill_md_file(path: &Path) -> bool {
    path.is_file() && path.file_name().is_some_and(|n| n.to_string_lossy().to_lowercase() == "skill.md")
}

fn resolve_skill_path(path: &Path) -> PathBuf {
    if is_skill_md_file(path) {
        path.parent().unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { skill_path } => {
            let dir = resolve_skill_path(&skill_path);
            let errors = skills_ref::validate(&dir);
            if errors.is_empty() {
                println!("Valid skill: {}", dir.display());
            } else {
                eprintln!("Validation failed for {}:", dir.display());
                for e in &errors {
                    eprintln!("  - {}", e);
                }
                process::exit(1);
            }
        }
        Commands::ReadProperties { skill_path } => {
            let dir = resolve_skill_path(&skill_path);
            match skills_ref::read_properties(&dir) {
                Ok(props) => {
                    let json = serde_json::to_string_pretty(&props.to_json_value()).unwrap();
                    println!("{}", json);
                }
                Err(SkillError::Parse(msg)) | Err(SkillError::Validation { message: msg, .. }) => {
                    eprintln!("Error: {}", msg);
                    process::exit(1);
                }
            }
        }
        Commands::ToPrompt { skill_paths } => {
            if skill_paths.is_empty() {
                eprintln!("Error: at least one skill path is required");
                process::exit(1);
            }
            let resolved: Vec<PathBuf> = skill_paths.iter().map(|p| resolve_skill_path(p)).collect();
            let refs: Vec<&std::path::Path> = resolved.iter().map(|p| p.as_path()).collect();
            match skills_ref::to_prompt(&refs) {
                Ok(output) => println!("{}", output),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}
