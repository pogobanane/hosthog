use crate::diskstate;
use zbus_systemd::{zbus, zvariant::OwnedObjectPath};

type ExResult<T> = Result<T, Box<dyn std::error::Error + 'static>>;

/// Layout defined by https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1.html
#[derive(Debug)]
#[allow(dead_code)]
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

async fn list_timers<'a>(
    manager: &zbus_systemd::systemd1::ManagerProxy<'a>,
    states: Vec<String>,
) -> Vec<Unit> {
    let units = manager
        .list_units_by_patterns(states, vec!["*.timer".to_string()])
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
    return units.collect();
}

pub fn disable_resource(state: &mut diskstate::DiskState) {
    println!("systemd_timers: disable timers");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let ret = rt.block_on(disable_timers(state));
    if let Err(e) = ret {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

pub fn enable_resource(state: &mut diskstate::DiskState) {
    println!("systemd_timers: enable timers");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let ret = rt.block_on(enable_timers(state));
    if let Err(e) = ret {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn disable_timers(state: &mut diskstate::DiskState) -> ExResult<()> {
    let conn = zbus::Connection::system().await.expect("Can't connect");
    let manager = zbus_systemd::systemd1::ManagerProxy::new(&conn)
        .await
        .expect("Can't get systemd manager");

    // list units
    let patterns = vec![
        "active".to_string(),
        // "inactive".to_string()
    ];
    let units = list_timers(&manager, patterns).await;

    for unit in units {
        disable_timer(state, &manager, &unit).await;
        // println!(" - {} ({})", unit.name, unit.active_state);

        // // print timer details
        // let unit_proxy  = zbus_systemd::systemd1::TimerProxy::new(&conn, unit.unit_path)
        //     .await
        //     .expect("Can't get systemd unit");
        // let calendar = unit_proxy.timers_calendar().await.expect("Cant get calendar of timer");
        // for (timer_base, crontab_spec, elaps_timestamp) in calendar {
        //     println!("   - {}", crontab_spec);
        // }

        // manager.stop_unit(unit.name, "fail".to_string()); // or maybe "replace"?
    }
    return Ok(());
}

async fn disable_timer<'a>(
    state: &mut diskstate::DiskState,
    manager: &zbus_systemd::systemd1::ManagerProxy<'a>,
    unit: &Unit,
) {
    println!("disabling {}", unit.name);
    match manager
        .stop_unit(unit.name.clone(), "fail".to_string())
        .await
    {
        // or maybe "replace"?
        Err(zbus::Error::MethodError(name, _option, _message))
            if name == "org.freedesktop.DBus.Error.AccessDenied" =>
        {
            println!(
                "WARN: insuficient permissions to start {}. Try to run this program as root.",
                unit.name
            );
        }
        Err(e) => {
            panic!("Can't start timer unit: {}", e);
        }
        Ok(_) => {
            if !state.disabled_systemd_timers.contains(&unit.name) {
                state.disabled_systemd_timers.push(unit.name.clone());
            }
        }
    };
}

async fn enable_timers(state: &mut diskstate::DiskState) -> ExResult<()> {
    let conn = zbus::Connection::system().await.expect("Can't connect");
    let manager = zbus_systemd::systemd1::ManagerProxy::new(&conn)
        .await
        .expect("Can't get systemd manager");

    let disabled_timers_copy: Vec<String> = state
        .disabled_systemd_timers
        .iter()
        .map(|t| t.clone())
        .collect();
    for timer_name in disabled_timers_copy {
        println!("enabling {}", timer_name);
        match manager
            .start_unit(timer_name.clone(), "fail".to_string())
            .await
        {
            // or maybe "replace"?
            Err(zbus::Error::MethodError(name, _option, _message))
                if name == "org.freedesktop.DBus.Error.AccessDenied" =>
            {
                println!(
                    "WARN: insuficient permissions to start {}. Try to run this program as root.",
                    timer_name
                );
            }
            Err(e) => {
                panic!("Can't start timer unit: {}", e);
            }
            Ok(_) => {
                // let foo = state.disabled_systemd_timers.iter().filter_map(|t| {
                //     if *t == timer_name {
                //         None
                //     } else {
                //         Some(t.clone())
                //     }
                // }).collect();
                // state.disabled_systemd_timers = foo;
                state.disabled_systemd_timers.retain(|t| *t != timer_name);
            }
        };
    }
    return Ok(());
}
