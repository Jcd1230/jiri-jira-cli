mod client;
mod config;
mod formatter;
mod commands;

use clap::{Parser, Subcommand};
use client::JiraClient;
use config::Config;
use formatter::{Formatter, OutputFormat};

#[derive(Parser)]
#[command(name = "jiri")]
#[command(about = "Minimal Jira CLI", long_about = None)]
struct Cli {
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config = Config::from_env()?;
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
    }

    Ok(())
}
