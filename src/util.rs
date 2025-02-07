use std::fs;
use std::io;
use std::path::Path;
use chrono;
use chrono::{DateTime, Local};

pub const STATE_PATH: &str = "/var/lib/hosthog";

pub fn prog() -> String {
    std::env::current_exe()
        .ok()
        .expect("Cant look up your binary name.")
        .to_str()
        .expect("Your binary name looks very unexpected")
        .to_owned()
}

pub fn prog_name() -> String {
    std::env::current_exe()
        .ok()
        .expect("Cant look up your binary name.")
        .file_name()
        .expect("Your binary does not seem to have a name.")
        .to_str()
        .expect("Your binary name looks very unexpected.")
        .to_owned()
}

pub fn remove_dir_contents<P: AsRef<Path>>(path: P) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        fs::remove_file(entry?.path())?;
    }
    Ok(())
}

pub fn format_timeout_abs(timeout: DateTime<Local>) -> String {
    let now = DateTime::from(Local::now());
    let duration = timeout - now;
    format_timeout(duration)
}

pub fn format_timeout(duration: chrono::Duration) -> String {
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

pub fn get_username(uid: u32) -> String {
    let passwd = unsafe { libc::getpwuid(uid) };
    if passwd.is_null() {
        return "<unknown>".to_string();
    }
    let passwd = unsafe { &*passwd };
    let name = unsafe { std::ffi::CStr::from_ptr(passwd.pw_name) };
    return name.to_str().unwrap().to_string();
}

