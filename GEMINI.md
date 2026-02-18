# Jiri (Jira CLI)

A minimal, fast, and modular Jira CLI client written in Rust.

## Overview
`jiri` allows users to interact with Jira Cloud directly from the terminal. It supports listing projects and searching issues using JQL, with flexible output formats (Table, CSV, Plain). It was rewritten from an original TypeScript implementation to improve performance and portability.

## Technology Stack
- **Language**: Rust (Edition 2021)
- **CLI Framework**: [`clap`](https://crates.io/crates/clap)
- **HTTP Client**: [`reqwest`](https://crates.io/crates/reqwest) (async, structured with `rustls`)
- **Async Runtime**: [`tokio`](https://crates.io/crates/tokio)
- **Serialization**: [`serde`](https://crates.io/crates/serde), `serde_json`
- **Table Formatting**: [`comfy-table`](https://crates.io/crates/comfy-table)

## Configuration
The tool requires the following environment variables to be set:
- `JIRA_API_USERNAME`: Your Atlassian account email.
- `JIRA_API_TOKEN`: Your Atlassian API token.
- `JIRA_SITE`: Base Jira site URL (e.g., `https://your-org.atlassian.net`).

## Usage

### Build
```bash
cargo build --release
```

### Run
The binary is located in `target/release/jiri-jira-cli`.

#### List Projects
```bash
./target/release/jiri-jira-cli projects
```

#### Search Issues
```bash
# Basic search
./target/release/jiri-jira-cli search "assignee = currentUser()"

# Custom fields and limit
./target/release/jiri-jira-cli search "project = TJP" --fields "key,summary,status,priority" --limit 20

# CSV Output
./target/release/jiri-jira-cli search "project = TJP" --csv > issues.csv
```

## Project Structure
- **`src/main.rs`**: Entry point. Defines the `clap` CLI structure and dispatches commands.
- **`src/config.rs`**: Handles loading and validation of environment variables.
- **`src/client.rs`**: Contains `JiraClient`, wrapping the Jira REST API interactions and handling authentication.
- **`src/formatter.rs`**: Logic for rendering data into Table, CSV, or Plain text formats.
- **`src/commands/`**: Separate modules for each subcommand.
  - **`projects.rs`**: Implementation of `jiri projects`.
  - **`search.rs`**: Implementation of `jiri search`, including field resolution and JQL execution.

## Key Features
- **Field Discovery**: Use `--get-fields` with the search command to see available field IDs for the returned issues, aiding in JQL construction.
- **Smart Formatting**: Automatically handles complex Jira field types (arrays, objects) to display human-readable values.
- **TLS**: Uses `rustls` to avoid dependency on system OpenSSL libraries.

## Development Workflow
This project uses [Jujutsu (jj)](https://github.com/martinvonz/jj) for version control. It is recommended to commit frequently using `jj commit` to track changes granularly.
