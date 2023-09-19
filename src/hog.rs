use nix;

pub fn hog_ssh(users: Vec<String>) {
    nix::mount::mount(Some("/home/peter/dev/phd/hosthog/foo"), "/home/peter/dev/phd/hosthog/bar", None::<&str>, nix::mount::MsFlags::MS_BIND, None::<&str>).expect("bind mount failed");

    // let mut command = vec![String::from("pkill"), String::from("-u")];
    // command.extend(users);
    // run(&command);
}
