use zbus_systemd::{zbus, zvariant::OwnedObjectPath};

type ExResult<T> = Result<T, Box<dyn std::error::Error + 'static>>;

/// Layout defined by https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1.html
#[derive(Debug)]
struct Unit {
    name: String,
    description: String,
    loaded_state: String,
    active_state: String,
    sub_state: String,
    followup_unit: String,
    unit_path: OwnedObjectPath,
    job_id: u32,
    job_type: String,
    job_path: OwnedObjectPath,
}

async fn list_units() {}

pub fn start_hook() {
    println!("systemd_timers hook start");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let ret = rt.block_on(run());
    if let Err(e) = ret {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> ExResult<()> {
    let conn = zbus::Connection::system().await.expect("Can't connect");
    let manager = zbus_systemd::systemd1::ManagerProxy::new(&conn)
        .await
        .expect("Can't get systemd manager");
    let target = manager
        .get_default_target()
        .await
        .expect("Can't get default target");
    println!("Default target: '{}'", target);

    // list units
    let units = manager
        .list_units()
        .await
        .expect("Can't list systemd units");
    let patterns = vec![
        "active".to_string(),
        "inactive".to_string()
    ];
    let units = manager
        .list_units_by_patterns(patterns, vec!["*.timer".to_string()])
        .await
        .expect("Can't list systemd units");
    // convert unit tuple to struct
    let units = units.into_iter().map(
        |(
            name,
            description,
            loaded_state,
            active_state,
            sub_state,
            followup_unit,
            unit_path,
            job_id,
            job_type,
            job_path,
        )| {
            Unit {
                name,
                description,
                loaded_state,
                active_state,
                sub_state,
                followup_unit,
                unit_path,
                job_id,
                job_type,
                job_path,
            }
        },
    );
    for unit in units {
        let unit_proxy  = zbus_systemd::systemd1::TimerProxy::new(&conn, unit.unit_path)
            .await
            .expect("Can't get systemd unit");
        let calendar = unit_proxy.timers_calendar().await.expect("Cant get calendar of timer");

        println!(" - {} ({})", unit.name, unit.active_state);
        for (timer_base, crontab_spec, elaps_timestamp) in calendar {
            println!("   - {}", crontab_spec);
        }
        match manager.start_unit(unit.name.clone(), "fail".to_string()).await { // or maybe "replace"?
            Err(zbus::Error::MethodError(name, option, message)) if name == "org.freedesktop.DBus.Error.AccessDenied" => {
                println!("WARN: insuficient permissions to start {}. Try to run this program as root.", unit.name);
            },
            Err(e) => {
                panic!("Can't start timer unit: {}", e);
            },
            Ok(_) => {}
        };
        // manager.stop_unit(unit.name, "fail".to_string()); // or maybe "replace"?
    }
    return Ok(());
}
