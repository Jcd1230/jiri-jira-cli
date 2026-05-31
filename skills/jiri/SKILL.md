---
name: jiri
description: A CLI tool for interacting with Jira Cloud and Confluence Cloud to manage issues, projects, pages, and automated workflows.
---

# Jiri (Jira & Confluence CLI)

`jiri` is a command-line interface for Atlassian Cloud. It allows you to manage Jira issues (search, view, create, edit, transition, assign, comment, attach) and Confluence pages (search, view, create, edit with programmatic patching).

## Setup

Run the interactive setup wizard:
```bash
jiri init
```

Or configure manually via `~/.config/jiri/config.toml`, `jiri.toml` (local), or environment variables.

### Configuration (`config.toml`)
```toml
[auth]
username = "your-email@example.com"
token = "your-api-token"
site = "https://your-org.atlassian.net"

[general]
default_project = "PROJ"
```

### Environment Variables
- `JIRA_API_USERNAME`
- `JIRA_API_TOKEN`
- `JIRA_SITE`
- `JIRA_DEFAULT_PROJECT` (optional)

## Jira Commands

### Searching Issues
```bash
# Basic search (default fields: key, summary)
jiri search "assignee = currentUser() AND status = 'In Progress'"

# Custom fields and output limit
jiri search "project = PROJ" --fields "key,summary,status,priority,assignee" --limit 50

# Machine-readable output
jiri search "project = PROJ" --jsonl | jq '.KEY'
jiri search "project = PROJ" --csv --no-header

# Discover available fields
jiri search "project = PROJ" --get-fields

# Search all projects (ignores default_project filter)
jiri search "status = Done" --all-projects
```

### Viewing an Issue
```bash
jiri view PROJ-123
```

### Transitioning an Issue
```bash
# List available transitions
jiri transition PROJ-123

# Perform a transition
jiri transition PROJ-123 "Done"
```

### Creating an Issue
```bash
jiri create --project PROJ --summary "Fix the login bug" --type Bug --description "Details"
```

### Editing an Issue
```bash
jiri edit PROJ-123 --summary "Updated title" --labels "bug,urgent"
jiri edit PROJ-123 --assignee "jane@example.com"
jiri edit PROJ-123 --field "Story Points=5" --field-json "customfield_10010={\"value\":\"High\"}"
```

### Bulk Editing
```bash
jiri bulk-edit --jql "project = PROJ AND status = Done" --labels "archived" --yes
jiri bulk-edit --issues "PROJ-1,PROJ-2" --assignee "jane@example.com"
```

### Assigning, Commenting, Attaching
```bash
jiri assign PROJ-123 "jane@example.com"
jiri comment PROJ-123 "Fixed in latest commit."
jiri attach PROJ-123 ./screenshot.png --message "See attached"
```

### Opening in Browser
```bash
jiri open PROJ-123
```

## Confluence Commands

### Searching Pages
```bash
jiri confluence search "Release Notes"
jiri confluence search "Meeting" --space TEAM
jiri confluence search --cql "space = TEAM and lastModified > now('-1w')"
```

### Viewing a Page
```bash
jiri confluence view 12345678          # rendered plain text
jiri confluence view 12345678 --raw    # raw ADF JSON
```

### Creating a Page
```bash
jiri confluence create "Page Title" --space TEAM --content "# Hello\nMarkdown body"
```

### Editing a Page (Programmatic Patcher)
```bash
jiri confluence edit 12345678 --append "## New Section\nContent here"
jiri confluence edit 12345678 --prepend "# WARNING\nThis page is deprecated"
jiri confluence edit 12345678 --replace "OLD_TERM:NEW_TERM"
jiri confluence edit 12345678 --anchor "heading:Changelog" --after "## v1.2\n- Fixed bug"
jiri confluence edit 12345678 --title "New Title" --minor
```

## Output Formats

All tabular commands support these global flags:
- `--csv` — comma-separated values
- `--json` — JSON array of objects (keys from header row)
- `--jsonl` — one JSON object per line (ideal for `jq` and streaming)
- `--markdown` — GitHub-flavored Markdown table
- `--plain` — space-padded columns, no borders
- `--no-header` — omit header row

## Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Auth/permission error |
| 4 | Resource not found |
| 5 | Network error |

## Tips for Agents
- Use `jiri search "..." --get-fields` first to discover field IDs before requesting custom fields.
- Use `--jsonl` for programmatic processing — easier to parse than tables or CSV.
- Use `--csv --no-header` when you need simple line-by-line output.
- When transitioning issues, first run `jiri transition <KEY>` without a status to see available options.
- For Confluence edits, prefer `--anchor "heading:Section Name" --after "content"` for precise placement.
- Use `jiri doctor` to diagnose connectivity or auth problems.
- Exit codes are machine-friendly: check for 2 (config), 3 (auth), 4 (not found), 5 (network) in scripts.
