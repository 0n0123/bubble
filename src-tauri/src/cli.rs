use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Args {
    #[clap(default_value = "./bubble.toml")]
    pub config: PathBuf,
}

impl Args {
    pub fn load() -> Args {
        Args::parse()
    }
}