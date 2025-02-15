use crate::diskstate;
use zbus_systemd::{zbus, zvariant::OwnedObjectPath};

const DISABLE_TIMERS: bool = true;
const DISABLE_UNITS: &[&str] = &["xrdp.service"];

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

async fn list_units<'a>(
    manager: &zbus_systemd::systemd1::ManagerProxy<'a>,
    states: Vec<String>,
    match_globs: Vec<String>,
) -> Vec<Unit> {
    let units = manager
        .list_units_by_patterns(states, match_globs)
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
    println!("systemd_units: disable systemd services");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let ret = rt.block_on(disable_units(state));
    if let Err(e) = ret {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

pub fn enable_resource(state: &mut diskstate::DiskState) {
    println!("systemd_units: enable systemd services");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let ret = rt.block_on(enable_units(state));
    if let Err(e) = ret {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn disable_units(state: &mut diskstate::DiskState) -> ExResult<()> {
    let conn = zbus::Connection::system().await.expect("Can't connect");
    let manager = zbus_systemd::systemd1::ManagerProxy::new(&conn)
        .await
        .expect("Can't get systemd manager");

    // list units
    let states = vec![
        "active".to_string(),
        // "inactive".to_string()
    ];
    let mut units = vec![];
    if DISABLE_TIMERS {
        let names = vec!["*.timer".to_string()];
        units.append(&mut list_units(&manager, states.clone(), names.clone()).await);
    }
    for unit in DISABLE_UNITS {
        let names = vec![unit.to_string()];
        units.append(&mut list_units(&manager, states.clone(), names).await);
    }

    for unit in units {
        disable_unit(state, &manager, &unit).await;
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

async fn disable_unit<'a>(
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
            panic!("Can't start systemd unit: {}", e);
        }
        Ok(_) => {
            if !state.disabled_systemd_units.contains(&unit.name) {
                state.disabled_systemd_units.push(unit.name.clone());
            }
        }
    };
}

async fn enable_units(state: &mut diskstate::DiskState) -> ExResult<()> {
    let conn = zbus::Connection::system().await.expect("Can't connect");
    let manager = zbus_systemd::systemd1::ManagerProxy::new(&conn)
        .await
        .expect("Can't get systemd manager");

    let disabled_timers_copy: Vec<String> = state
        .disabled_systemd_units
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
                panic!("Can't start systemd unit: {}", e);
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
                state.disabled_systemd_units.retain(|t| *t != timer_name);
            }
        };
    }
    return Ok(());
}
