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

### Priority (highest to lowest)
1. **Local config file** (`jiri.toml`)
2. **Global config file** (`~/.config/jiri/config.toml`)
3. **Environment variables**

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
jiri search "status = 'In Progress'"                  # filters by default project if set
jiri search "status = 'In Progress'" --all-projects   # search across all projects
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

#### Setup Wizard
```bash
jiri init   # interactive onboarding: credentials, validation, completions
```

#### Edit an Issue
```bash
jiri edit PROJ-123 --summary "New title" --labels "bug,urgent"
jiri edit PROJ-123 --field "Story Points=5"
```

#### Bulk Edit Issues
```bash
jiri bulk-edit --jql "project = PROJ AND status = Done" --labels "archived"
jiri bulk-edit --issues "PROJ-1,PROJ-2" --assignee "jane@co.com" --yes
```

#### Assign / Attach / Open
```bash
jiri assign PROJ-123 "jane@example.com"
jiri attach PROJ-123 ./screenshot.png --message "See attached"
jiri open PROJ-123
```

#### Configuration & Diagnostics
```bash
jiri config show
jiri config set project PROJ
jiri doctor
```

### Output Formats
All tabular commands support global flags:
- `--csv` — comma-separated values
- `--json` — JSON array of objects (header-keyed)
- `--jsonl` — newline-delimited JSON (one object per line)
- `--markdown` — GitHub-flavored Markdown table
- `--plain` — space-padded columns, no borders
- `--no-header` — omit header row

### Shell Completions
```bash
jiri completions bash >> ~/.bashrc
jiri completions zsh >> ~/.zshrc
jiri completions fish > ~/.config/fish/completions/jiri.fish
```

## Project Structure
- **`jiri-core/`**: Reusable library crate (public API for external integrations).
  - **`src/adf.rs`**: Atlassian Document Format (ADF) parsing and manipulation.
  - **`src/client.rs`**: `AtlassianClient` for Jira and Confluence REST APIs.
  - **`src/config.rs`**: Configuration loading and layering.
  - **`src/fields.rs`**: Jira field resolution and value formatting.
  - **`src/formatter.rs`**: Multi-format output renderer.
- **`src/main.rs`**: CLI entry point and argument definitions.
- **`src/commands/`**: Subcommand implementations.

## Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Auth/permission error (401/403) |
| 4 | Resource not found (404) |
| 5 | Network/connectivity error |

## Release Process
- **Commit often**: Use `jj commit` (or `jj describe`) frequently to track progress.
- **Version Bumps**: Do NOT create separate "chore: bump version" commits. Always include the `Cargo.toml` version increment in the final commit of the feature or bugfix.
- **Cleanup**: Ensure the `master` bookmark is updated before tagging a release.
- **Release**: Use `gh release create vX.Y.Z --target master --generate-notes`.

## Key Features
- **Programmatic Patcher**: Reliable targeted edits to Confluence pages with auto-retries on version conflicts.
- **Markdown Support**: Automatically converts Markdown to ADF for Confluence edits.
- **Smart Formatting**: Human-readable tables and plain-text ADF rendering.
- **TLS**: Uses `rustls` — no system OpenSSL dependency.
