use clap::{Parser, Subcommand, ValueEnum};

#[rustfmt::skip]
#[derive(Parser)]
#[clap(
    name = "server",
    about = "Tombola indexer and GraphQL server",
    author = "Tombola Team",
    version = "0.1.0"
)]
pub struct Cli {
    #[clap(
        long,
        required = false,
        default_value = "config.yaml",
        help = "Path to the configuration file"
    )]
    pub config: String,
    #[clap(
        long, 
        value_enum,
        default_value_t = LogLevel::Info, 
        help = "Log level"
    )]
    pub log_level: LogLevel,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[clap(name = "indexer", about = "Start chain the indexer(s)")]
    Indexer {
        #[clap(
            long,
            help = "List of chains to process, if not provided all chains will be processed"
        )]
        chains: Option<Vec<String>>,
        #[clap(long, help = "Start the indexer with tasks", default_value = "false")]
        with_tasks: bool,
    },
    #[clap(name = "graphql", about = "Start the GraphQL server")]
    GraphQL,
}

/// Log levels which allow to specify the verbosity of the logs output.
#[derive(Debug, Clone, ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

impl ToString for Commands {
    fn to_string(&self) -> String {
        match self {
            Commands::Indexer { .. } => "indexer".to_string(),
            Commands::GraphQL => "graphql".to_string(),
        }
    }
}