use clap::{Args, Parser, Subcommand};
use std::process::Command;
use chrono::prelude::*;

mod hog;
mod diskstate;
mod users;
mod claims;

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
    /// prematurely release a claim (removes all of your claims)
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
    }
}

fn show_status_verbose(_cmd: StatusCommand, state: &diskstate::DiskState) {
    println!("{}", serde_yaml::to_string(&state).unwrap());
}

fn show_status(_cmd: StatusCommand, state: &diskstate::DiskState) {
    if state.overmounts.len() > 0 {
        println!("");
        println!("System is currently hogged!");
        println!("The following ssh keys are temporarily disabled:");
        let active_overmounts = state.overmounts.iter().filter(|file| hog::is_overmounted(file)).collect::<Vec<&String>>().len();
        // for file in &state.overmounts {
        //     if hog::is_overmounted(file) {
        //         active_overmounts += 1;
        //     }
        // }
        println!("{} keys were disabled to hog, {} keys are still disabled", state.overmounts.len(), active_overmounts);
        println!("");
    }

    println!("Active claims:");

    println!("{:<13} {:<13} {}", "Remaning", "User", "Comment");
    let now = DateTime::from(Local::now());
    for claim in &state.claims {
        // format timeout duration
        let duration = claim.timeout - now;
        let duration = format_timeout(duration);

        // replace duration with soft duration if applicable
        let duration = match claim.soft_timeout {
            Some(soft_timeout) => {
                let duration = soft_timeout - now;
                let duration = format_timeout(duration);
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

fn do_hog(mut users: Vec<String>, state: &mut diskstate::DiskState) {
    let me = users::my_username();
    if !state.claims.iter().any(|claim| claim.user == me && claim.exclusive) {
        panic!("Hogging not allowed. Claim exclusive access first.");
    }
    println!("hog users:");
    if users.len() == 0 {
        users.push(String::from("root"));
        users.push(me);
    }
    users.as_slice().into_iter().for_each(|i| print!("{} ", i));
    println!("");
    hog::hog_ssh(users, state);
    // let mut command = vec![String::from("pkill"), String::from("-u")];
    // command.extend(users);
    // run(&command);
}

fn do_release(state: &mut diskstate::DiskState) {
    hog::release_ssh(state);
    // delete claims of current user
    let user = users::my_username();
    state.claims.retain(|claim| claim.user != user);
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

fn format_timeout(duration: chrono::Duration) -> String {
    // determine order of magnitude
    let is_seconds = duration.num_seconds() < 60;
    let is_minutes = duration.num_minutes() < 60;
    let is_hours = duration.num_hours() < 24;
    let is_days = duration.num_days() < 7;
    let is_weeks = duration.num_weeks() < 4;

    // format accurdingly
    if is_seconds {
        return format!("{}s", duration.num_seconds());
    }
    if is_minutes {
        return format!("{}m", duration.num_minutes());
    }
    if is_hours {
        return format!("{}h", duration.num_hours());
    }
    if is_days {
        return format!("{}d", duration.num_days());
    }
    if is_weeks {
        return format!("{}w", duration.num_weeks());
    }

    unreachable!();
}

fn main() {
    let cli = Cli::parse();

    let _original_state = diskstate::load();
    let mut state = diskstate::load();
    diskstate::maintenance(&mut state);

    match cli.command {
        Some(Commands::Status { status }) if !status.verbose => {
            show_status(status, &mut state);
        }
        Some(Commands::Status { status }) if status.verbose => {
            show_status_verbose(status, &mut state);
        }
        Some(Commands::Claim { claim }) => {
            println!("claim");
            claims::do_claim(&claim, &mut state); 
            // println!("claim until {}", parse_timeout(&timeout));
        }
        Some(Commands::Release { }) => {
            do_release(&mut state);
        }
        Some(Commands::Hog{ users }) => do_hog(users, &mut state),
        Some(Commands::Post{ message }) => do_post(message),
        Some(Commands::Users { }) => {
            users::do_list_users();
        },
        None => {
            show_status(StatusCommand::default(), &mut state);
            println!(
                "See more options with: {} help",
                prog().expect("Your binary name looks very unexpected.")
            );
        }
        _ => unimplemented!()
    };
    
    if _original_state != state {
        // println!("state changed, storing");
        diskstate::store(&state);
    }
}
