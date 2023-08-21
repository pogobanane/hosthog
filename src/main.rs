use clap::{Args, Parser, Subcommand};
use std::process::Command;
use chrono::prelude::*;

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
    /// Claim a resource. Fails if already claimed exclusively.
    Claim {
        /// Timeout (hard): after this time the claim will be removed
        timeout: String,
        /// Optional timeout: will not remove the claim, but will be shown in the status
        #[arg(short, long)]
        soft_timeout: Option<String>,
        /// Claim explclusive access. Other new claims will not be allowed.
        #[arg(short, long)]
        exclusive: bool,
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

use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Debug)]
struct Claim {
    timeout: String,
    soft_timeout: Option<String>,
    exclusive: bool,
}

fn show_status(cmd: StatusCommand) {
    println!("Showing status.");
    println!("status list: {}", cmd.list);
    let claim = Claim { timeout: String::from("1h"), soft_timeout: None, exclusive: false };
    let json = serde_json::to_string(&claim).unwrap();
    println!("{}", json);
    let claim: Claim = serde_json::from_str(&json).unwrap();
    println!("{:?}", claim);
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

fn parse_timeout(timeout: &str) -> DateTime<Local> {

    let now = DateTime::from(Local::now());

    // try to parse as duration
    match duration_str::parse(timeout) {
        Ok(parsed) => {
            return now + chrono::Duration::from_std(parsed).unwrap();
        },
        Err(e) => {
            println!("error parsing timeout as duration: {}. Trying again as absolute datetime.", e);
            // try to parse as datetime
            match dateparser::parse(timeout) {
                Ok(parsed) => {
                    return DateTime::from(parsed);
                },
                Err(e) => println!("error parsing timeout as date: {}", e)
            };
        }
    }

    panic!("proper error handling");
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Status { status }) => {
            show_status(status);
        }
        Some(Commands::Claim { timeout, .. }) => {
            println!("claim until {}", parse_timeout(&timeout));
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
