mod client;
mod config;
mod formatter;
mod commands;

use clap::{Parser, Subcommand};
use clap_complete::Shell;
use client::JiraClient;
use config::Config;
use formatter::{Formatter, OutputFormat};

#[derive(Parser)]
#[command(name = "jiri")]
#[command(about = "Minimal Jira CLI", long_about = None)]
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
}

#[derive(Subcommand)]
enum Commands {
    /// List projects visible to the authenticated user
    Projects,
    /// Run a JQL search and list issues
    Search {
        /// The JQL query string
        jql: String,
        /// Comma-separated fields to display (default: key,summary)
        #[arg(short, long)]
        fields: Option<String>,
        /// Show available fields on the first returned issue
        #[arg(long)]
        get_fields: bool,
        /// Maximum number of issues to fetch (default: 1000)
        #[arg(long, default_value = "1000")]
        limit: i64,
    },
    /// View a single issue's details
    View {
        /// The issue key (e.g. PROJ-123)
        key: String,
    },
    /// Transition an issue to a new status
    Transition {
        /// The issue key (e.g. PROJ-123)
        key: String,
        /// Target status name (omit to list available transitions)
        status: Option<String>,
    },
    /// Create a new issue
    Create {
        /// Project key (e.g. PROJ)
        #[arg(short, long)]
        project: Option<String>,
        /// Issue summary
        #[arg(short, long)]
        summary: String,
        /// Issue type (default: Task)
        #[arg(short = 't', long, default_value = "Task")]
        issue_type: String,
        /// Issue description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Add a comment to an issue
    Comment {
        /// The issue key (e.g. PROJ-123)
        key: String,
        /// Comment message
        message: String,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Completions don't need auth
    if let Commands::Completions { shell } = &cli.command {
        commands::completions::run(*shell);
        return Ok(());
    }

    let config = Config::load()?;
    let client = JiraClient::new(config);

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
        Commands::Completions { .. } => unreachable!(),
    }

    Ok(())
}
