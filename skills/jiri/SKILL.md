---
name: jiri
description: A CLI tool for interacting with Jira Cloud to manage issues, projects, and comments.
---

# Jiri (Jira CLI)

`jiri` is a command-line interface for Jira Cloud. It allows you to list projects, search for issues using JQL, view issue details, transition issues between states, create new issues, and add comments.

## Setup

The tool expects a configuration file at `~/.config/jiri/config.toml` or `jiri.toml` in the current directory, or environment variables.

### Configuration (`jiri.toml`)
```toml
[auth]
username = "your-email@example.com"
token = "your-api-token"
site = "https://your-org.atlassian.net"
```

### Environment Variables
- `JIRA_API_USERNAME`
- `JIRA_API_TOKEN`
- `JIRA_SITE`

## Usage

### Listing Projects
List all projects visible to the user.
```bash
jiri projects
```

### Searching Issues
Search for issues using Jira Query Language (JQL).
```bash
# Basic search (default fields: key, summary)
jiri search "assignee = currentUser() AND status = 'In Progress'"

# Custom fields
jiri search "project = PROJ" --fields "key,summary,status,priority,assignee"

# Comma-separated output (useful for parsing)
jiri search "project = PROJ" --csv --no-header

# Discover available fields for a query (prints field IDs and names)
jiri search "project = PROJ" --get-fields
```

### Viewing an Issue
View details of a specific issue, including description and recent comments.
```bash
jiri view PROJ-123
```

### Transitioning an Issue
Move an issue to a different status (e.g., "To Do" -> "In Progress").

```bash
# List available transitions
jiri transition PROJ-123

# Perform a transition (fuzzy match)
jiri transition PROJ-123 "Done"
```

### Creating an Issue
Create a new issue.
```bash
jiri create --project PROJ --summary "Fix the login bug" --type Bug --description "Login fails with 500 error."
```

### Adding a Comment
Add a comment to an issue.
```bash
jiri comment PROJ-123 "I have fixed this in the latest commit."
```

## Tips for Agents
- Use `jiri search "..." --get-fields` first if you need to know which fields are available or what their IDs are before constructing a complex JQL query or requesting specific fields.
- Use `--csv` format when you need to process many issues programmatically, as it is easier to parse than the default table output.
- When transitioning issues, first run `jiri transition <KEY>` to see the exact names of available transitions (e.g., "In Progress" vs "In Review").
