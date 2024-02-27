use nix;
use crate::diskstate;
use crate::users;
use once_cell::sync::Lazy;
use crate::util;
use std::fs;

static OVERLAY_PATH: Lazy<String> = Lazy::new(|| format!("{}/overlay", util::STATE_PATH));

pub fn ssh_hogged_message(claim: &diskstate::Claim) -> String {
    let duration = util::format_timeout_abs(claim.timeout);
    vec![
        format!("This system has been hogged by {}.", claim.user),
        format!("Comment: {}", claim.comment),
        format!("This claim will time out in {}.", duration),
    ].join("\n")
}

fn ssh_hogged_command(message: &str) -> String {
    message.lines().map(|line| 
        format!("echo {}", line)
    ).collect::<Vec<String>>().join("; ")
}

fn escape(input: &str) -> String {
    input.chars().map(|c| match c {
        '/' => String::from("_"),
        '_' => String::from("__"),
        _ => format!("{}", c),
    }).collect()
}

fn overmount(file: &str, hogged_message: &str) -> Result<(), Option<nix::errno::Errno>> {
    if !std::path::Path::new(file).is_file() {
        return Err(None);
    }

    let authorized_keys: String = fs::read_to_string(file).expect("foo");
    let overlay_keys = authorized_keys.lines().map(|line| 
        if !line.is_empty() {
            format!("restrict,command=\"{}\" {}", ssh_hogged_command(hogged_message), line)
        } else {
            String::from(line)
        }
    ).collect::<Vec<String>>().join("\n");
    let overlay_file = format!("{}/{}", OVERLAY_PATH.as_str(), escape(file));
    fs::create_dir_all(OVERLAY_PATH.as_str()).expect("foo2");
    fs::write(overlay_file.as_str(), overlay_keys).expect("foo1");

    match nix::mount::mount(
        // Some("/dev/null"), 
        Some(overlay_file.as_str()), 
        file,
        None::<&str>, 
        nix::mount::MsFlags::MS_BIND, 
        None::<&str>
    ) {
        Ok(_) => {},
        Err(err) => { return Err(Some(err)); },
    }

    return Ok(());
}

#[derive(Debug)]
pub struct User {
    pub name: String,
    pub home: String,
}

fn list_users() -> Vec<User> {
    let mut users = vec![];

    loop {
        // safe because we null check before accessing it
        let passwd = unsafe {
            let passwd = libc::getpwent();
            if passwd.is_null() { break };
            *passwd
        };
        if passwd.pw_dir.is_null() { continue };
        // safe because we null check before accessing it
        let home = unsafe { std::ffi::CStr::from_ptr(passwd.pw_dir).to_string_lossy().into_owned() };
        // safe because we null check before accessing it
        if passwd.pw_name.is_null() { continue };
        let name = unsafe { std::ffi::CStr::from_ptr(passwd.pw_name).to_string_lossy().into_owned() };

        users.push(User { name, home });
    }

    // safe because i dont know what might be unsafe about it
    unsafe { libc::endpwent() };

    return users;
}

fn hog_ssh(exclude_users: Vec<String>, state: &mut diskstate::DiskState, message: String) {
    let users = list_users().into_iter().filter(|u| !exclude_users.contains(&u.name)).collect::<Vec<User>>();
    let all_auth_key_files: Vec<String> = diskstate::expand_authorized_keys_file(&state.settings, users);
    let all_files_len = all_auth_key_files.len();
    let auth_key_files: Vec<String> = 
        all_auth_key_files.into_iter()
        // filter out files that we recorded as overmounted
        .filter(|f| !state.overmounts.contains(f))
        // filter out files that have an unknown overmount
        .filter(|f| !is_overmounted(f)).collect();
    let mut failed = 0;
    for file in &auth_key_files {
        match overmount(&file, message.as_str()) {
            Ok(_) => { state.overmounts.push(file.clone()); },
            Err(None) => { }, // ignore files that dont exist
            Err(Some(_errno)) => { failed += 1; },
        }
    }
    // println!("overmounts: {:?}", state.overmounts);
    println!("{} users locked out of ssh ({} skipped, {} failed)", auth_key_files.len(), all_files_len - auth_key_files.len(), failed);

    // let mut command = vec![String::from("pkill"), String::from("-u")];
    // command.extend(users);
    // run(&command);
}

pub fn do_hog(mut users: Vec<String>, state: &mut diskstate::DiskState) {
    let me = users::my_username();
    let claim = match state.claims.iter().find(|claim| claim.user == me && claim.exclusive) {
        Some(claim) => claim.clone(),
        None => panic!("Hogging not allowed. Claim exclusive access first."),
    };

    // Sanity check:
    if let Some(hogger) = &state.hogger {
        if hogger.user != me {
            panic!("Hogging not allowed. The system is already hogged other user {}.", hogger.user);
        }
    }

    println!("hog users:");
    if users.len() == 0 {
        users.push(String::from("root"));
        users.push(me);
    }
    users.as_slice().into_iter().for_each(|i| print!("{} ", i));
    println!("");
    let message = ssh_hogged_message(&claim);
    hog_ssh(users, state, message);
    state.hogger = Some(claim);
    // let mut command = vec![String::from("pkill"), String::from("-u")];
    // command.extend(users);
    // run(&command);
}

fn release_ssh(state: &mut diskstate::DiskState) {
    let mut overmounts: Vec<String> = vec![];
    for file in &state.overmounts {
        let path = std::path::Path::new(file);
        match nix::mount::umount(path) {
            Err(err) => {
                println!("failed to release {}: {:?}", file, err);
                overmounts.push(file.clone());
            },
            Ok(_) => {
                println!("released {}", file);
            },
        }
    }
    if let Err(err) = util::remove_dir_contents(OVERLAY_PATH.as_str()) {
        println!("WARN: could not remove overlayed files: {}", err);
    }
    state.overmounts = overmounts;
}

pub fn do_release(state: &mut diskstate::DiskState) {
    // always unhog (even when we think its not hogged) to converge towards intended state
    release_ssh(state);
    // delete exclusive claim of user used to issue this hogging
    if let Some(hogger) = &state.hogger {
        state.claims.retain(|claim| claim != hogger);
        state.hogger = None;
    }
    
    // remove "me"s exclusive claims
    let me = users::my_username();
    state.claims.retain(|claim| !(claim.user == me && claim.exclusive));
}

pub fn is_overmounted(file: &str) -> bool {
    let mounts = std::fs::read_to_string("/proc/mounts").expect("Cant read /proc/mounts");
    return mounts.contains(file);
}
