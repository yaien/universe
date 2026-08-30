pub use clap::{Parser, Subcommand};
use url::Url;

#[derive(Parser, Debug)]
#[command(version = "1.0")]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "Create a new organization with admin user")]
    Create {
        #[arg()]
        url: Url,

        #[arg()]
        title: String,

        #[arg()]
        email: String,
    },
    #[command(about = "Invite a user to the organization")]
    Invite {
        #[arg()]
        email: String,

        #[arg()]
        hostname: String,
    },
}
