use clap::{Args, Parser, Subcommand};
use std::process::Command;

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
        Self { list: true }
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
    /// post a message to all logged in users
    ///
    /// The message will arrive at:
    /// - login shells (wall)
    /// - all tmux sessions (tmux display-popup)
    Post {
        /// message to post
        message: Vec<String>
    },
    /// List all logged in users
    ///
    /// Checks:
    /// - login shells/ssh sessions (users)
    /// - tmux sessions of all users
    /// - xrdp sessions
    /// - vscode?
    Users {
    }
}

fn show_status(cmd: StatusCommand) {
    println!("Showing status.");
    println!("status list: {}", cmd.list);
}

fn prog() -> Option<String> {
    std::env::current_exe()
        .ok()?
        .file_name()?
        .to_str()?
        .to_owned()
        .into()
}

/// successfully runs a command or crashes
fn run(command: &Vec<String>) {
    if command.len() < 1 {
        panic!("can not execute emptystring");
    }
    let mut args = command.into_iter();
    let bin = args.next().expect("can not execute emptystring");
    let out = Command::new(bin)
        .args(args)
        .output()
        .expect("failed to run binary");
    if !out.status.success() {
        panic!("{} exited with code {}", bin, out.status.code().expect("no exit code found"));
    }
    println!("{:?}", out.stdout);
    println!("{:?}", out.stderr);
}

fn do_post(mut message: Vec<String>) {
    println!("post message:");
    message.as_slice().into_iter().for_each(|i| print!("{} ", i));
    println!("");
    message.insert(0, String::from("wall"));
    run(&message);
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Status { status }) => {
            show_status(status);
        }
        Some(Commands::Claim { status }) => {
            println!("claim list: {}", status.list);
        }
        Some(Commands::Release { status }) => {
            println!("release list: {}", status.list);
        }
        Some(Commands::Post{ message }) => do_post(message),
        None => {
            println!("print some global settings like link to calendar, spreadsheet or database");
            show_status(StatusCommand::default());
            println!(
                "See more options with: {} help",
                prog().expect("Your binary name looks very unexpected.")
            );
        }
        _ => unimplemented!()
    };
}
