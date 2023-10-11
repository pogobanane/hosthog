use nix;
use crate::diskstate;

fn overmount(file: &str) -> Result<(), ()> {
    if !std::path::Path::new(file).is_file() {
        return Err(());
    }
    nix::mount::mount(
        Some("/dev/null"), 
        file,
        None::<&str>, 
        nix::mount::MsFlags::MS_BIND, 
        None::<&str>
    ).expect("bind mount failed");

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

pub fn hog_ssh(exclude_users: Vec<String>, state: &mut diskstate::DiskState) {
    let users = list_users().into_iter().filter(|u| !exclude_users.contains(&u.name)).collect::<Vec<User>>();
    let auth_key_files = diskstate::expand_authorized_keys_file(&state.settings, users);
    for file in auth_key_files {
        match overmount(&file) {
            Ok(_) => { state.overmounts.push(file); },
            Err(_) => { },
        }
    }
    println!("overmounts: {:?}", state.overmounts);



    // let mut command = vec![String::from("pkill"), String::from("-u")];
    // command.extend(users);
    // run(&command);
}
