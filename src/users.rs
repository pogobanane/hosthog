use netstat::*;

pub fn do_list_users() {
    println!("User sessions:");
    let who = std::process::Command::new("who")
        .output()
        .expect("failed to run who");
    println!("{}", String::from_utf8_lossy(&who.stdout));
    println!("");

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
            // ProtocolSocketInfo::Tcp(tcp_si) => println!(
            //     "TCP {}:{} -> {}:{} {:?} - {}",
            //     tcp_si.local_addr,
            //     tcp_si.local_port,
            //     tcp_si.remote_addr,
            //     tcp_si.remote_port,
            //     si.associated_pids,
            //     tcp_si.state
            // ),
            // ProtocolSocketInfo::Udp(udp_si) => println!(
            //     "UDP {}:{} -> *:* {:?}",
            //     udp_si.local_addr, udp_si.local_port, si.associated_pids
            // ),
            _ => (),
        }
    }

    // println!("SSH sessions B:");
    // let procs = pgrep("sshd");
    // for (pid, cmdline) in procs {
    //     println!("{}: {}", pid, cmdline);
    // }
}

fn pgrep(pattern: &str) -> Vec<(u32, String)> {
    let mut procs = vec![];
    for pid in list_all_pids() {
        let cmdline = std::fs::read_to_string(format!("/proc/{}/cmdline", pid)).unwrap();
        if cmdline.contains(pattern) {
            procs.push((pid, cmdline));
        }
    }
    return procs;
}

fn list_all_pids() -> Vec<u32> {
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

pub fn my_username() -> String {
    let me = unsafe {
        let cstr = libc::getlogin();
        if cstr.is_null() {
            panic!("no login name found");
        }
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(cstr as *const u8, libc::strlen(cstr)))
    }.to_string();
    return me;
}
