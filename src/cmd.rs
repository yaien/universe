pub use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version = "1.0")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Serve,
    CreateOrganization {
        #[arg()]
        url: String,

        #[arg()]
        hostname: String,

        #[arg()]
        title: String,
    },
}
