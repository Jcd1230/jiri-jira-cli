# Jiri (Jira & Confluence CLI)

A minimal, fast, and modular Atlassian CLI client written in Rust.

## Overview
`jiri` allows users to interact with Jira Cloud and Confluence Cloud directly from the terminal. It supports listing projects, searching issues, managing issue transitions, and programmatically editing Confluence pages with targeted patches.

## Technology Stack
- **Language**: Rust (Edition 2021)
- **CLI Framework**: [`clap`](https://crates.io/crates/clap)
- **HTTP Client**: [`reqwest`](https://crates.io/crates/reqwest) (async, with `rustls`)
- **Async Runtime**: [`tokio`](https://crates.io/crates/tokio)
- **Serialization**: [`serde`](https://crates.io/crates/serde), `serde_json`
- **Table Formatting**: [`comfy-table`](https://crates.io/crates/comfy-table)
- **Markdown Parsing**: [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark)

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

### Jira Commands

#### List Projects
```bash
jiri projects
```

#### Search Issues
```bash
jiri search "assignee = currentUser()"
jiri search "project = TJP" --fields "key,summary,status" --limit 20
jiri search "project = TJP" --csv > issues.csv
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

### Confluence Commands (v2 API)

#### Search Pages
```bash
jiri confluence search "Release Notes"
jiri confluence search "Meeting" --space 12345
```

#### View a Page
```bash
jiri confluence view 12345678
jiri confluence view 12345678 --raw  # show raw ADF JSON
```

#### Edit a Page (Programmatic Patcher)
`jiri` implements a robust Fetch-Modify-PUT cycle for targeted edits. It automatically handles ADF tree manipulation and version conflict retries.

```bash
# Append a new section (Markdown supported)
jiri confluence edit 12345678 --append "## New Section\nDone via CLI!"

# Prepend a header
jiri confluence edit 12345678 --prepend "# IMPORTANT\nUpdated on $(date)"

# Search and replace text
jiri confluence edit 12345678 --replace "OLD_TERM:NEW_TERM"

# Rename page and mark as minor edit (silence notifications)
jiri confluence edit 12345678 --title "New Title" --minor
```

### Shell Completions
```bash
jiri completions bash >> ~/.bashrc
jiri completions zsh >> ~/.zshrc
jiri completions fish > ~/.config/fish/completions/jiri.fish
```

## Project Structure
- **`src/main.rs`**: Entry point and CLI definition.
- **`src/client.rs`**: `AtlassianClient` for Jira and Confluence REST APIs.
- **`src/adf.rs`**: Atlassian Document Format (ADF) parsing and manipulation.
- **`src/commands/`**: Subcommand implementations.

## Key Features
- **Programmatic Patcher**: Reliable targeted edits to Confluence pages with auto-retries on version conflicts.
- **Markdown Support**: Automatically converts Markdown to ADF for Confluence edits.
- **Smart Formatting**: Human-readable tables and plain-text ADF rendering.
- **TLS**: Uses `rustls` — no system OpenSSL dependency.
