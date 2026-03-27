mod client;
mod config;
mod formatter;
mod commands;
mod adf;
mod fields;

use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use client::AtlassianClient;
use config::Config;
use formatter::{Formatter, OutputFormat};

fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Cyan.on_default())
}

/// Command-line interface for Jiri (Jira & Confluence CLI).
/// 
/// A minimal, fast, and modular CLI client for Atlassian Cloud.
#[derive(Parser)]
#[command(name = "jiri")]
#[command(version)]
#[command(about, long_about = None)]
#[command(styles = get_styles())]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output comma-separated values (no borders)
    #[arg(long, global = true)]
    csv: bool,

    /// No borders, padded columns
    #[arg(long, global = true)]
    plain: bool,

    /// Omit header row
    #[arg(long, global = true)]
    no_header: bool,

    /// Verbose output (debug logging of API requests)
    #[arg(long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List Jira projects visible to you
    #[command(visible_alias = "p")]
    Projects,

    /// Search Jira issues using JQL
    /// 
    /// Examples:
    ///   jiri search "assignee = currentUser()"
    ///   jiri search "project = TJP" --fields "key,summary,status" --limit 20
    #[command(visible_alias = "s")]
    Search {
        /// The JQL query string
        jql: String,
        /// Comma-separated fields to display (default: key,summary)
        #[arg(short, long)]
        fields: Option<String>,
        /// Show available fields on the first returned issue
        #[arg(long)]
        get_fields: bool,
        /// Maximum number of issues to fetch
        #[arg(long, default_value = "1000")]
        limit: i64,
    },

    /// View details of a specific Jira issue
    /// 
    /// Example: jiri view PROJ-123
    #[command(visible_alias = "v")]
    View {
        /// The issue key (e.g. PROJ-123)
        key: String,
    },

    /// Transition a Jira issue to a new status
    /// 
    /// If no status is provided, it lists available transitions.
    #[command(visible_alias = "t")]
    Transition {
        /// The issue key (e.g. PROJ-123)
        key: String,
        /// Target status name or ID (omit to list available)
        status: Option<String>,
    },

    /// Create a new Jira issue
    #[command(visible_alias = "c")]
    Create {
        /// Project key (e.g. PROJ). Uses config default if omitted.
        #[arg(short, long)]
        project: Option<String>,
        /// Short summary of the issue
        #[arg(short, long)]
        summary: String,
        /// Issue type name (default: Task)
        #[arg(short = 't', long, default_value = "Task")]
        issue_type: String,
        /// Detailed description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Add a comment to a Jira issue
    Comment {
        /// The issue key (e.g. PROJ-123)
        key: String,
        /// Text of the comment
        message: String,
    },

    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        subcommand: ConfigCommands,
    },

    /// Diagnostic tool to check configuration and connectivity
    Doctor,

    /// Confluence Cloud operations (Search, View, Edit)
    #[command(visible_alias = "conf")]
    Confluence {
        #[command(subcommand)]
        subcommand: ConfluenceCommands,
    },

    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show the current configuration
    Show {
        /// Show global configuration file
        #[arg(short, long)]
        global: bool,
        /// Show local (project) configuration file
        #[arg(short, long)]
        local: bool,
    },
    /// Set a configuration value
    Set {
        /// The key to set (e.g., auth.username, general.default_project)
        key: String,
        /// The value to set
        value: String,
        /// Set in global configuration file
        #[arg(short, long)]
        global: bool,
        /// Set in local (project) configuration file
        #[arg(short, long)]
        local: bool,
    },
}

#[derive(Subcommand)]
enum ConfluenceCommands {
    /// Search for Confluence pages
    /// 
    /// Examples:
    ///   jiri confluence search "Release Notes"
    ///   jiri confluence search "*dev*" --limit 50
    ///   jiri confluence search --cql "space = TPL and lastModified > now('-1w')"
    Search {
        /// Page title fragment or CQL query
        query: Option<String>,
        /// Filter by space ID or Key
        #[arg(short, long)]
        space: Option<String>,
        /// Maximum number of results to fetch
        #[arg(long, default_value = "25")]
        limit: i64,
        /// Interpret query as a raw CQL string
        #[arg(long)]
        cql: bool,
    },

    /// Create a new Confluence page
    Create {
        /// Page title
        title: String,
        /// Space ID or Key
        #[arg(short, long)]
        space: String,
        /// Parent Page ID (folder)
        #[arg(short, long)]
        parent: Option<String>,
        /// Page content (Markdown by default)
        #[arg(short, long)]
        content: Option<String>,
        /// Use raw ADF JSON for content
        #[arg(long)]
        adf: bool,
    },

    /// View content of a Confluence page
    /// 
    /// Renders Atlassian Document Format (ADF) as plain text.
    View {
        /// The page ID
        id: String,
        /// Output raw ADF JSON instead of rendered text
        #[arg(long)]
        raw: bool,
    },

    /// Programmatically edit a Confluence page
    /// 
    /// Performs a Fetch-Modify-PUT cycle to ensure targeted edits
    /// are safe and handle version conflicts automatically.
    /// 
    /// Input content defaults to Markdown unless --adf is specified.
    Edit {
        /// The page ID
        id: String,

        /// Replace the entire document content
        #[arg(long, group = "action")]
        full: Option<String>,

        /// Append content to the end of the document
        #[arg(long, group = "action")]
        append: Option<String>,

        /// Prepend content to the beginning of the document
        #[arg(long, group = "action")]
        prepend: Option<String>,

        /// Global recursive find and replace text (format: "OLD:NEW")
        #[arg(long, group = "action")]
        replace: Option<String>,

        /// Identify a target node for relative edits (format: "type:query")
        /// Types: heading, panel, list, id
        #[arg(long)]
        anchor: Option<String>,

        /// Insert content before the anchor
        #[arg(long, group = "action", requires = "anchor")]
        before: Option<String>,

        /// Insert content after the anchor
        #[arg(long, group = "action", requires = "anchor")]
        after: Option<String>,

        /// Replace the anchor node with new content
        #[arg(long, group = "action", requires = "anchor")]
        replace_node: Option<String>,

        /// Update the page title
        #[arg(long)]
        title: Option<String>,

        /// Treat input as raw ADF JSON instead of Markdown
        #[arg(long)]
        adf: bool,

        /// Mark as minor edit to suppress notifications
        #[arg(long)]
        minor: bool,
    },
}

#[tokio::main]
/// Entry point for the jiri CLI.
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.verbose {
        std::env::set_var("JIRI_VERBOSE", "1");
    }

    // Completions don't need auth
    if let Commands::Completions { shell } = &cli.command {
        commands::completions::run(*shell);
        return Ok(());
    }

    let config = Config::load()?;
    let client = AtlassianClient::new(config);

    let format = if cli.csv {
        OutputFormat::CSV
    } else if cli.plain {
        OutputFormat::Plain
    } else {
        OutputFormat::Table
    };

    let formatter = Formatter::new(format, cli.no_header);

    match cli.command {
        Commands::Projects => {
            commands::projects::run(&client, &formatter).await?;
        }
        Commands::Search {
            jql,
            fields,
            get_fields,
            limit,
        } => {
            commands::search::run(&client, &formatter, jql, fields, get_fields, limit).await?;
        }
        Commands::View { key } => {
            commands::view::run(&client, key).await?;
        }
        Commands::Transition { key, status } => {
            commands::transition::run(&client, key, status).await?;
        }
        Commands::Create {
            project,
            summary,
            issue_type,
            description,
        } => {
            let project_key = project
                .or_else(|| client.config().default_project.clone())
                .ok_or("Project key is required. Use --project or set default_project in config.")?;
            commands::create::run(&client, project_key, summary, issue_type, description).await?;
        }
        Commands::Comment { key, message } => {
            commands::comment::run(&client, key, message).await?;
        }
        Commands::Config { subcommand } => {
            match subcommand {
                ConfigCommands::Show { global, local } => {
                    commands::config::run_show(global, local).await?;
                }
                ConfigCommands::Set { key, value, global, local } => {
                    commands::config::run_set(key, value, global, local).await?;
                }
            }
        }
        Commands::Doctor => {
            commands::doctor::run(&client).await?;
        }
        Commands::Confluence { subcommand } => {
            match subcommand {
                ConfluenceCommands::Search {
                    query,
                    space,
                    limit,
                    cql,
                } => {
                    commands::confluence::run_search(&client, &formatter, query, space, limit, cql)
                        .await?;
                }
                ConfluenceCommands::Create {
                    title,
                    space,
                    parent,
                    content,
                    adf,
                } => {
                    commands::confluence::run_create(&client, title, space, parent, content, adf)
                        .await?;
                }
                ConfluenceCommands::View { id, raw } => {
                    commands::confluence::run_view(&client, id, raw).await?;
                }
                ConfluenceCommands::Edit {
                    id,
                    full,
                    append,
                    prepend,
                    replace,
                    anchor,
                    before,
                    after,
                    replace_node,
                    title,
                    adf,
                    minor,
                } => {
                    commands::confluence::run_edit(
                        &client, id, full, append, prepend, replace, anchor, before, after, replace_node, title, adf, minor,
                    )
                    .await?;
                }
            }
        }
        Commands::Completions { .. } => unreachable!(),
    }

    Ok(())
}
