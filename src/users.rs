use netstat::*;

pub fn do_list_users() {
    println!("User sessions:");
    let who = std::process::Command::new("who")
        .output()
        .expect("failed to run who");
    println!("{}", String::from_utf8_lossy(&who.stdout));
    println!("");

    if !is_root() {
        println!("Skipping ssh sessions (you are not root)");
    } else {
        println!("SSH sessions:");
        let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
        let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
        let sockets_info = get_sockets_info(af_flags, proto_flags).unwrap();
        for si in sockets_info {
            match si.protocol_socket_info {
                ProtocolSocketInfo::Tcp(TcpSocketInfo {
                    state: TcpState::Established,
                    local_port: 22,
                    ..
                }) => {
                    let mut cmdlines = vec![];
                    for pid in &si.associated_pids {
                        let cmdline = std::fs::read_to_string(format!("/proc/{}/cmdline", pid)).unwrap();
                        // trim trailing null bytes
                        let cmdline = cmdline.trim_matches(char::from(0)).to_string();
                        cmdlines.push(cmdline);
                    }
                    let cmdline = cmdlines.join(", ");
                    println!("{}", cmdline);
                    // println!("{:?}", si.associated_pids);
                },
                _ => (),
            }
        }
    }

    // println!("SSH sessions B:");
    // let procs = pgrep("sshd");
    // for (pid, cmdline) in procs {
    //     println!("{}: {}", pid, cmdline);
    // }
}

fn _pgrep(pattern: &str) -> Vec<(u32, String)> {
    let mut procs = vec![];
    for pid in _list_all_pids() {
        let cmdline = std::fs::read_to_string(format!("/proc/{}/cmdline", pid)).unwrap();
        if cmdline.contains(pattern) {
            procs.push((pid, cmdline));
        }
    }
    return procs;
}

fn _list_all_pids() -> Vec<u32> {
    let mut pids = vec![];
    for entry in std::fs::read_dir("/proc").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            let pid = match path.file_name().unwrap().to_str().unwrap().parse::<u32>() {
                Ok(pid) => pid,
                Err(_) => continue,
            };
            pids.push(pid);
        }
    }
    return pids;
}

/// true if we have root permissions
pub fn is_root() -> bool {
    return unsafe { libc::geteuid() == 0 };
}

/// get actual login username (instead of root when in sudo)
pub fn my_username() -> Option<String> {
    let me = unsafe {
        let cstr = libc::getlogin();
        if cstr.is_null() {
            println!("WARN: no login name found");
            return None;
        }
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(cstr as *const u8, libc::strlen(cstr)))
    }.to_string();
    return Some(me);
}
