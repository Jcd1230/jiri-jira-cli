# Jiri (Jira CLI)

A minimal, fast, and modular Jira CLI client written in Rust.

## Overview
`jiri` allows users to interact with Jira Cloud directly from the terminal. It supports listing projects, searching issues with JQL, viewing issue details, managing transitions, creating issues, adding comments, and generating shell completions. It was rewritten from an original TypeScript implementation to improve performance and portability.

## Technology Stack
- **Language**: Rust (Edition 2021)
- **CLI Framework**: [`clap`](https://crates.io/crates/clap)
- **HTTP Client**: [`reqwest`](https://crates.io/crates/reqwest) (async, with `rustls`)
- **Async Runtime**: [`tokio`](https://crates.io/crates/tokio)
- **Serialization**: [`serde`](https://crates.io/crates/serde), `serde_json`
- **Table Formatting**: [`comfy-table`](https://crates.io/crates/comfy-table)

## Configuration

### Config File (recommended)
Create `~/.config/jiri/config.toml`:
```toml
[auth]
username = "you@example.com"
token = "your-api-token"
site = "https://your-org.atlassian.net"

[general]
default_project = "TJP"
```

### Environment Variables (fallback)
If no config file is found, jiri reads:
- `JIRA_API_USERNAME`: Your Atlassian account email.
- `JIRA_API_TOKEN`: Your Atlassian API token.
- `JIRA_SITE`: Base Jira site URL (e.g., `https://your-org.atlassian.net`).
- `JIRA_DEFAULT_PROJECT`: Default project key (optional).

## Usage

### Build
```bash
cargo build --release
```

### Commands

#### List Projects
```bash
jiri projects
```

#### Search Issues
```bash
jiri search "assignee = currentUser()"
jiri search "project = TJP" --fields "key,summary,status" --limit 20
jiri search "project = TJP" --csv > issues.csv
jiri search "project = TJP" --get-fields
```

#### View an Issue
```bash
jiri view PROJ-123
```

#### Transition an Issue
```bash
jiri transition PROJ-123             # list available transitions
jiri transition PROJ-123 "In Progress"  # perform transition
```

#### Create an Issue
```bash
jiri create --project PROJ --summary "Fix bug" --type Bug --description "Details here"
```

#### Add a Comment
```bash
jiri comment PROJ-123 "This is my comment"
```

#### Shell Completions
```bash
jiri completions bash >> ~/.bashrc
jiri completions zsh >> ~/.zshrc
jiri completions fish > ~/.config/fish/completions/jiri.fish
```

### Releases
Automated releases are handled via GitHub Actions. To trigger a new release:
1. Update the version in `Cargo.toml`.
2. Create and push a new Git tag:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. The GitHub Action will automatically build binaries for Linux, macOS, and Windows and attach them to a new GitHub Release.

## Project Structure
- **`src/main.rs`**: Entry point. Defines the `clap` CLI structure and dispatches commands.
- **`src/config.rs`**: Loads credentials from config file or environment variables.
- **`src/client.rs`**: `JiraClient` wrapping all Jira REST API interactions.
- **`src/formatter.rs`**: Renders data as Table, CSV, or Plain text.
- **`src/commands/`**: One module per subcommand:
  - `projects.rs`, `search.rs`, `view.rs`, `transition.rs`, `create.rs`, `comment.rs`, `completions.rs`

## Key Features
- **Field Discovery**: Use `--get-fields` with search to discover available Jira field IDs.
- **Smart Formatting**: Handles complex Jira field types (arrays, objects, ADF) to display human-readable values.
- **TLS**: Uses `rustls` — no system OpenSSL dependency.
- **Shell Completions**: Generate completions for bash, zsh, fish, elvish, or PowerShell.

## Development Workflow
This project uses [Jujutsu (jj)](https://github.com/martinvonz/jj) for version control. It is recommended to commit frequently using `jj commit` to track changes granularly.
