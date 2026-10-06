use clap::Parser;

#[derive(Parser)]
pub(super) struct Cli {
    /// A path to the manager configration JSON file.
    #[arg(long)]
    pub config: String,

    /// Whether to show additional information during execution.
    #[arg(long)]
    pub verbose: bool,
}
