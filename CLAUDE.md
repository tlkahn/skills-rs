# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**skills-ref** is a Python reference library for the Agent Skills specification. It provides tools for parsing, validating, and generating prompts from SKILL.md files. Demonstration-only, not for production use.

All source code lives under `py/`.

## Commands

All commands run from the `py/` directory:

```bash
# Install dependencies
uv sync

# Run all tests
uv run pytest

# Run a single test file or specific test
uv run pytest tests/test_parser.py
uv run pytest tests/test_validator.py::test_name_must_be_lowercase

# Format code
uv run ruff format .

# Lint (with autofix)
uv run ruff check --fix .
```

## Architecture

The library follows a pipeline: **parse → validate → generate prompt**.

- `parser.py` — Finds and parses SKILL.md files (YAML frontmatter + markdown body). Entry points: `find_skill_md()`, `read_properties()`
- `validator.py` — Validates skill properties (name constraints, field limits, directory name match, i18n with NFKC normalization). Entry point: `validate()`
- `prompt.py` — Generates `<available_skills>` XML blocks for agent system prompts. Entry point: `to_prompt()`
- `models.py` — `SkillProperties` dataclass (name, description, license, compatibility, allowed_tools, metadata)
- `errors.py` — Exception hierarchy: `SkillError` → `ParseError`, `ValidationError`
- `cli.py` — Click CLI with subcommands: `validate`, `read-properties`, `to-prompt`

Public API is exported from `__init__.py`. The CLI entry point is `skills_ref.cli:main`, installed as the `skills-ref` command.
