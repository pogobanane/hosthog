use nix;
use crate::diskstate;
use crate::users;

fn overmount(file: &str) -> Result<(), Option<nix::errno::Errno>> {
    if !std::path::Path::new(file).is_file() {
        return Err(None);
    }
    match nix::mount::mount(
        Some("/dev/null"), 
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

fn hog_ssh(exclude_users: Vec<String>, state: &mut diskstate::DiskState) {
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
        match overmount(&file) {
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
    hog_ssh(users, state);
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
