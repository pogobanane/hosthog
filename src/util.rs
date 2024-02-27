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
