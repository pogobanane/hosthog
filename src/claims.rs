use chrono::{DateTime, Local, Duration};
use crate::ClaimCommand;
use crate::diskstate::{DiskState, Claim};
use crate::parse_timeout;
use crate::users;
use crate::util;
use std::process::{Command, Stdio};
use std::io::ErrorKind;
use std::io::Write;

fn next_minute(timeout: DateTime<Local>) -> DateTime<Local> {
    // timeout is in same minute. `at` cant handle that because it ignores seconds.
    // Hence we always have to select the next minute.
    return timeout + Duration::seconds(61);
}

fn schedule_maintenance(timeout: DateTime<Local>) {
    let timeout = next_minute(timeout);
    let future_command = format!("{} maintenance", util::prog());
    let future = format!("{}", timeout.format("%H:%M %Y-%m-%d"));
    println!("Scheduling job {} at {}", future_command, future);
    match Command::new("at").arg(future).stdin(Stdio::piped()).spawn() {
        Ok(mut command) => {
            {
                let mut stdin = command.stdin.take().expect("Failed to open stdin");
                stdin.write_all(future_command.as_bytes()).expect("Failed to write to stdin");
            }
            let exit = command.wait().expect("Command didnt run");
            if !exit.success() {
                println!("Scheduling maintenance failed");
            }
        },
        Err(err) if err.kind() == ErrorKind::NotFound => {
            println!("Claim may not expire in time (program `at` is missing).");
        }
        Err(err) => {
            println!("Claim may not expire in time: at: {}", err);
        },
    }
}

pub fn do_claim(claim: &ClaimCommand, state: &mut DiskState) {
    // filter claims for exclusive claims by other users
    let other_exclusive_claims = state.claims.iter().any(|claim| claim.exclusive && claim.user != users::my_username());
    if other_exclusive_claims {
        panic!("Exclusive claim already exists. Release first.");
    }

    let timeout = parse_timeout(&claim.timeout);
    let soft_timeout = match &claim.soft_timeout {
        Some(soft_timeout) => Some(parse_timeout(soft_timeout)),
        None => None,
    }; 
    let claim = Claim {
        timeout,
        soft_timeout,
        exclusive: claim.exclusive,
        user: users::my_username(),
        comment: claim.comment.join(" "),
    };

    state.claims.push(claim.clone());
    
    println!("{:?}", claim);
    schedule_maintenance(timeout);
}
