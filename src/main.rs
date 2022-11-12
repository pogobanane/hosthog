use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Args)]
struct StatusCommand {
    #[arg(short, long)]
    /// help string
    list: bool,
}

// this does not actually set the default value for the cli
impl Default for StatusCommand {
    fn default() -> Self {
        Self {
            list: true,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// show current claims
    Status {
        #[command(flatten)]
        status: StatusCommand,
    },
    /// claim a resource
    Claim {
        #[command(flatten)]
        status: StatusCommand,
    },
    /// prematurely release a claim
    Release {
        #[command(flatten)]
        status: StatusCommand,
    },
}

fn show_status(cmd: &StatusCommand) {
    println!("Showing status.");
    println!("status list: {}", cmd.list);
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Status{ status }) => {
            show_status(status);
        },
        Some(Commands::Claim{ status }) => {
            println!("claim list: {}", status.list);
        },
        Some(Commands::Release{ status }) => {
            println!("release list: {}", status.list);
        },
        None => {
            println!("print some global settings like link to calendar, spreadsheet or database");
            show_status(&StatusCommand::default());
        }
    };
}

