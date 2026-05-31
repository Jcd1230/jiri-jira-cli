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

## Quick Start
```bash
jiri init              # interactive setup wizard
jiri doctor            # verify configuration and connectivity
```

## Usage

### Build
```bash
cargo build --release
```

### Output Formats
All tabular commands support these global flags:
```bash
jiri search "..." --csv          # comma-separated values
jiri search "..." --json         # JSON array of objects (header-keyed)
jiri search "..." --jsonl        # one JSON object per line (for streaming/jq)
jiri search "..." --markdown     # GitHub-flavored Markdown table
jiri search "..." --plain        # space-padded columns, no borders
jiri search "..." --no-header    # omit header row
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

### Other Commands

#### Edit an Issue
```bash
jiri edit PROJ-123 --summary "New title" --labels "bug,urgent"
jiri edit PROJ-123 --field "Story Points=5" --field-json "customfield_10010={\"value\":\"High\"}"
```

#### Bulk Edit Issues
```bash
jiri bulk-edit --jql "project = PROJ AND status = Done" --labels "archived"
jiri bulk-edit --issues "PROJ-1,PROJ-2,PROJ-3" --assignee "jane@co.com" --yes
```

#### Assign an Issue
```bash
jiri assign PROJ-123 "jane@example.com"
```

#### Attach a File
```bash
jiri attach PROJ-123 ./screenshot.png --message "See attached screenshot"
```

#### Open in Browser
```bash
jiri open PROJ-123
```

#### Manage Configuration
```bash
jiri config show              # show effective config and source
jiri config set project PROJ  # set default project
```

### Shell Completions
```bash
jiri completions bash >> ~/.bashrc
jiri completions zsh >> ~/.zshrc
jiri completions fish > ~/.config/fish/completions/jiri.fish
```

## Project Structure
- **`jiri-core/`**: Reusable library crate (can be used without the CLI).
  - **`src/adf.rs`**: Atlassian Document Format (ADF) parsing and manipulation.
  - **`src/client.rs`**: `AtlassianClient` for Jira and Confluence REST APIs.
  - **`src/config.rs`**: Configuration loading and layering.
  - **`src/fields.rs`**: Jira field resolution and value formatting.
  - **`src/formatter.rs`**: Multi-format output renderer (table, CSV, JSON, JSONL, Markdown, plain).
- **`src/main.rs`**: CLI entry point and argument definitions.
- **`src/commands/`**: Subcommand implementations.

## Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error (missing config) |
| 3 | Authentication/permission error (401/403) |
| 4 | Resource not found (404) |
| 5 | Network/connectivity error |

## Key Features
- **Programmatic Patcher**: Reliable targeted edits to Confluence pages with auto-retries on version conflicts.
- **Markdown Support**: Automatically converts Markdown to ADF for Confluence edits.
- **Smart Formatting**: Human-readable tables and plain-text ADF rendering.
- **TLS**: Uses `rustls` — no system OpenSSL dependency.
