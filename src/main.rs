use clap::{Args, Parser, Subcommand};
use std::process::Command;
use chrono::prelude::*;

mod hog;
mod diskstate;
mod users;
mod claims;
mod util;
mod systemd_timers;

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
    /// More detailed status
    verbose: bool,
}

// this does not actually set the default value for the cli
impl Default for StatusCommand {
    fn default() -> Self {
        Self { verbose: false}
    }
}

#[derive(Args)]
pub struct ClaimCommand {
    /// Timeout (hard): after this time the claim will be removed
    timeout: String,
    /// Optional message/note
    comment: Vec<String>,
    /// Optional timeout: will not remove the claim, but will be shown in the status
    #[arg(short, long)]
    soft_timeout: Option<String>,
    /// Claim explclusive access. Other new claims will not be allowed.
    #[arg(short, long)]
    exclusive: bool,
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
        #[command(flatten)]
        claim: ClaimCommand,
    },
    /// prematurely release a claim (removes all of your hogs and exclusive claims)
    Release {},
    /// Hog the entire host (others will hate you)
    Hog {
        /// Block ssh login for all users except the ones specified here (default: your user and
        /// root). Specify -u multiple times to add more users.
        #[arg(short, long)]
        users: Vec<String>,
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
    },

    #[command(hide(true))]
    /// Disable systemd-timers
    SystemdTimers {},
    #[command(hide(true))]
    // Internal command used to trigger updating the list of claims and hogs
    Maintenance {}
}

fn show_status_verbose(_cmd: StatusCommand, state: &diskstate::DiskState) {
    println!("{}", serde_yaml::to_string(&state).unwrap());
}

fn show_status(_cmd: StatusCommand, state: &diskstate::DiskState) {
    if state.overmounts.len() > 0 {
        println!("");
        if let Some(claim) = &state.hogger {
            println!("{}", hog::ssh_hogged_message(claim));
        }
        let active_overmounts = state.overmounts.iter().filter(|file| hog::is_overmounted(file)).collect::<Vec<&String>>().len();
        // println!("The following ssh keys are temporarily disabled:");
        // for file in &state.overmounts {
        //     if hog::is_overmounted(file) {
        //         active_overmounts += 1;
        //     }
        // }
        println!("{} keys were disabled to hog, {} keys are still disabled", state.overmounts.len(), active_overmounts);
        println!("");
    }

    println!("Active claims:");

    println!("{:<13} {:<13} {}", "Remaining", "User", "Comment");
    let now = DateTime::from(Local::now());
    for claim in &state.claims {
        // format timeout duration
        let duration = claim.timeout - now;
        let duration = util::format_timeout(duration);

        // replace duration with soft duration if applicable
        let duration = match claim.soft_timeout {
            Some(soft_timeout) => {
                let duration = soft_timeout - now;
                let duration = util::format_timeout(duration);
                format!("{} (soft)", duration)
            },
            None => duration,
        };

        let comment = match claim.exclusive {
            true => format!("(exclusive) {}", claim.comment),
            false => claim.comment.clone(),
        };

        println!("{:<13} {:<13} {}", duration, claim.user, comment);
    }
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

fn do_maintenance(mut state: &mut diskstate::DiskState) {
    let mut needs_release = false;
    diskstate::maintenance(&mut state, &mut needs_release);
    if needs_release {
        hog::do_release(&mut state);
    }
    if state.hogger.is_none() && state.overmounts.len() != 0 {
        println!("WARN: host is not hogged, yet there still seem to be unexpected overmounts. Attempting to remove.");
        hog::release_ssh(state);
    }
}

fn main() {
    let cli = Cli::parse();

    let _original_state = diskstate::load();
    let mut state = diskstate::load();

    match cli.command {
        Some(Commands::Status { status }) if !status.verbose => {
            show_status(status, &mut state);
        }
        Some(Commands::Status { status }) if status.verbose => {
            show_status_verbose(status, &mut state);
        }
        Some(Commands::Claim { claim }) => {
            do_maintenance(&mut state);
            claims::do_claim(&claim, &mut state); 
        }
        Some(Commands::Release { }) => {
            do_maintenance(&mut state);
            hog::do_release(&mut state);
        }
        Some(Commands::Hog{ users }) => {
            do_maintenance(&mut state);
            hog::do_hog(users, &mut state)
        },
        Some(Commands::Post{ message }) => {
            do_post(message)
        },
        Some(Commands::Users { }) => {
            users::do_list_users();
        },
        Some(Commands::SystemdTimers { }) =>{
            systemd_timers::start_hook();
        },
        Some(Commands::Maintenance { }) => {
            do_maintenance(&mut state);
        },
        None => {
            show_status(StatusCommand::default(), &mut state);
            println!(
                "See more options with: {} help",
                util::prog_name()
            );
        }
        _ => unimplemented!()
    };
    
    if _original_state != state {
        // println!("state changed, storing");
        diskstate::store(&state);
    }
}
