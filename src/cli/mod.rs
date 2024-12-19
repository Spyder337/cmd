pub mod git;

use crate::core::web::{SearchParams, basic_search};
pub use crate::git::*;
use chrono::{DateTime, Local};
use clap::{Parser, Subcommand, command};
use git::GitCommands;
#[derive(Parser, Debug)]
#[command(version, about = "Quality of life commands.", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    commands: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Git repo interactions.")]
    Git {
        #[command(subcommand)]
        com: GitCommands,
    },
    /// Seach google.
    #[command(
        about = "Search google.",
        long_about = r#"Special operators:
    - "*": Wildcard
    - "()": Parenthesis/Group
    - "allintext:": All text is in the website
    - "-": Exclude operator
    - "AND|OR": Conditonal search keywords
    - '"': Search for exact phrases or a word
    - "site": Restrict a search to a specific site."#
    )]
    WebSearch {
        /// The search query that shows up in the google search bar.
        #[arg(required = true)]
        query: String,
        #[arg(long, default_value = "true", default_value_t = true)]
        open: bool,
        #[arg(long)]
        site: Option<String>,
        #[arg(long = "allintext")]
        all_in_text: Option<String>,
    },
    #[command(about = "Gets a random quote if no flags are provided.")]
    Quotes {},
}

pub fn handle_commands(cli: &Cli) -> () {
    // println!("{:?}", cli.commands);
    match &cli.commands {
        Some(command) => match command {
            Commands::Git { com } => git::handle_commands(com),
            Commands::WebSearch {
                query,
                open,
                site,
                all_in_text,
            } => {
                let mut query_string = query.clone();
                if let Some(site) = site {
                    query_string.push_str(&format!(" site:{}", site));
                }
                if let Some(all_in_text) = all_in_text {
                    query_string.push_str(&format!(" allintext:{}", all_in_text));
                }
                let search = SearchParams::new(&query_string);

                basic_search(search, open);
            }
            Commands::Quotes {} => todo!(),
        },
        None => println!("No command provided."),
    }
}

/// Returns the current DateTime object in the local timezone.
pub fn time_now() -> DateTime<Local> {
    let local = chrono::Local::now();
    local
}
