# AGENTS.md of QSO Tools

## Overview
TBW

## Project Layout

- `src/main.rs`: entry point, dispatches subcommands
- `src/cli.rs`: top-level clap definitions (`Cli`, `Command`) and shared arguments (`QsoQueryArgs`)
- `assets/`: example files, Lua scripts and Lua type definitions

TBW

### Adding a Tool
TBW

## Commands

- Build: `cargo build`
- Lint: `cargo clippy --all-targets`
- Format: `cargo fmt`
- Test: `cargo test`

## Conventions

- **Use English**

### Rust

- Use `modname.rs` with `modname/` for modules that have submodules; _do not_ use `modname/mod.rs`
- Run `cargo fmt` and keep `cargo clippy` free of warnings
- Keep Wavelog API types and requests inside `src/wavelog/`

### Text Files

- Use LF line endings and UTF-8
- Follow `.editorconfig`

### Doc Comments

- Use third-person singular, present tense, active voice
- End with period except for listings
- Use capital letter at the beginning of the sentence

### Commit Message, Issue and PR

- Use imperative mood
- _Do not_ use conventional commit message

### Branch

- Use lowercase
- Hyphenate to separate words
- Use categories below:
    - feature/
    - fix/
    - refactor/
    - docs/
    - test/

### Documentations for Humans

- Use English or Japanese

### File Arrangement

- Example files should be placed under `assets/`
- Create subdirectories if necessary
