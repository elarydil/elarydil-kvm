use elarydil_kvm::{engine::Engine, model::*, platform, service::Service, store::Store};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
fn config(mode: &str, k: usize) -> Config {
    let mut c = Config::default();
    c.safety.startup_grace_ms = 1000;
    c.usb_devices = (0..3)
        .map(|i| Device {
            id: format!("d{i}"),
            alias: format!("USB {i}"),
            enabled: true,
            selectors: vec![Selector {
                id: format!("s{i}"),
                strategy: "vid-pid".into(),
                vid: 1,
                pid: i,
                serial: None,
                path: None,
                allow_ambiguous: false,
            }],
        })
        .collect();
    c.monitors.push(SavedMonitor {
        id: "m".into(),
        alias: "Desk".into(),
        fingerprint: "edid".into(),
        connector: None,
        input_options: vec![Input {
            value: 18,
            vcp_code: 96,
            label: "HDMI 2".into(),
            verified: false,
        }],
        hidden_inputs: vec![],
    });
    c.profiles.push(Profile {
        id: "p".into(),
        name: "Desk".into(),
        enabled: true,
        rules: vec![Rule {
            commands: CommandHooks::default(),
            id: "r".into(),
            name: "Away".into(),
            enabled: true,
            trigger: "disconnect".into(),
            device_ids: vec!["d0".into(), "d1".into(), "d2".into()],
            mode: mode.into(),
            minimum_count: k,
            coincidence_window_ms: Some(1500),
            debounce_ms: 400,
            rearm_ms: 500,
            cooldown_ms: 5000,
            dry_run: false,
            actions: vec![Action {
                id: "a".into(),
                monitor_id: "m".into(),
                vcp_code: 96,
                input_value: 18,
            }],
        }],
    });
    c.validate().unwrap();
    c
}
fn state(s: [bool; 3]) -> HashMap<String, bool> {
    s.into_iter()
        .enumerate()
        .map(|(i, v)| (format!("d{i}"), v))
        .collect()
}
fn arm(e: &mut Engine, c: &Config, t: u64) {
    assert!(e.tick(c, &state([true; 3]), t).is_empty());
    assert!(e.tick(c, &state([true; 3]), t + 500).is_empty());
}
#[test]
fn staggered_two_cycles_exactly_once() {
    let c = config("all", 1);
    let mut e = Engine::default();
    let mut writes = 0;
    for cycle in 0..2 {
        let t = cycle * 10000;
        arm(&mut e, &c, t);
        assert!(e.tick(&c, &state([false, true, true]), t + 1000).is_empty());
        assert!(e
            .tick(&c, &state([false, false, true]), t + 1100)
            .is_empty());
        assert!(e.tick(&c, &state([false; 3]), t + 1200).is_empty());
        assert!(e.tick(&c, &state([false; 3]), t + 1599).is_empty());
        writes += e.tick(&c, &state([false; 3]), t + 1600).len();
        for n in 0..20 {
            assert!(e.tick(&c, &state([false; 3]), t + 1700 + n * 50).is_empty())
        }
    }
    assert_eq!(writes, 2)
}
#[test]
fn startup_absent_never_fires() {
    let c = config("all", 1);
    let mut e = Engine::default();
    for t in (0..20000).step_by(50) {
        assert!(e.tick(&c, &state([false; 3]), t).is_empty())
    }
}
#[test]
fn two_of_three_with_one_remaining() {
    let c = config("at-least", 2);
    let mut e = Engine::default();
    arm(&mut e, &c, 0);
    e.tick(&c, &state([false, true, true]), 1000);
    e.tick(&c, &state([false, false, true]), 1100);
    assert_eq!(e.tick(&c, &state([false, false, true]), 1500).len(), 1)
}
#[test]
fn coincidence_boundary() {
    for (gap, expected) in [(1500, 1), (1501, 0)] {
        let c = config("all", 1);
        let mut e = Engine::default();
        arm(&mut e, &c, 0);
        e.tick(&c, &state([false, true, true]), 1000);
        e.tick(&c, &state([false, false, false]), 1000 + gap);
        assert_eq!(e.tick(&c, &state([false; 3]), 1400 + gap).len(), expected)
    }
}
#[test]
fn bounce_does_not_fire() {
    let c = config("all", 1);
    let mut e = Engine::default();
    arm(&mut e, &c, 0);
    e.tick(&c, &state([false; 3]), 1000);
    e.tick(&c, &state([true; 3]), 1200);
    assert!(e.tick(&c, &state([true; 3]), 2000).is_empty())
}
#[test]
fn any_requires_complete_return_before_rearm() {
    let c = config("any", 1);
    let mut e = Engine::default();
    arm(&mut e, &c, 0);
    e.tick(&c, &state([false, true, true]), 1000);
    assert_eq!(e.tick(&c, &state([false, true, true]), 1400).len(), 1);
    e.tick(&c, &state([false, true, true]), 7000);
    e.tick(&c, &state([false, true, true]), 7600);
    e.tick(&c, &state([false, false, true]), 8000);
    assert!(e.tick(&c, &state([false, false, true]), 8500).is_empty())
}
#[test]
fn resume_drops_stale_edges() {
    let c = config("all", 1);
    let mut e = Engine::default();
    arm(&mut e, &c, 0);
    e.reset(1000, 4000);
    for t in [1000, 5000, 6000, 10000] {
        assert!(e.tick(&c, &state([false; 3]), t).is_empty())
    }
}
#[test]
fn connect_rule() {
    let mut c = config("all", 1);
    c.profiles[0].rules[0].trigger = "connect".into();
    let mut e = Engine::default();
    e.tick(&c, &state([false; 3]), 0);
    e.tick(&c, &state([false; 3]), 500);
    e.tick(&c, &state([true; 3]), 1000);
    assert_eq!(e.tick(&c, &state([true; 3]), 1400).len(), 1)
}
#[test]
fn disabled_device_blocks_evaluation() {
    let c = config("all", 1);
    let mut e = Engine::default();
    arm(&mut e, &c, 0);
    let mut p = state([false; 3]);
    p.remove("d0");
    for t in [1000, 2000, 3000] {
        assert!(e.tick(&c, &p, t).is_empty())
    }
}
#[test]
fn cooldown_consumes_transition() {
    let c = config("all", 1);
    let mut e = Engine::default();
    arm(&mut e, &c, 0);
    e.tick(&c, &state([false; 3]), 1000);
    assert_eq!(e.tick(&c, &state([false; 3]), 1400).len(), 1);
    arm(&mut e, &c, 2000);
    e.tick(&c, &state([false; 3]), 3000);
    assert!(e.tick(&c, &state([false; 3]), 3400).is_empty());
    assert!(e.tick(&c, &state([false; 3]), 10000).is_empty())
}
#[test]
fn no_window_allows_stagger() {
    let mut c = config("all", 1);
    c.profiles[0].rules[0].coincidence_window_ms = None;
    let mut e = Engine::default();
    arm(&mut e, &c, 0);
    e.tick(&c, &state([false, true, true]), 1000);
    e.tick(&c, &state([false; 3]), 5000);
    assert_eq!(e.tick(&c, &state([false; 3]), 5400).len(), 1)
}
#[test]
fn number_formats() {
    assert_eq!(number("18").unwrap(), 18);
    assert_eq!(number("0x12").unwrap(), 18);
    for bad in ["256", "-1", "0x100", "HDMI 2", ""] {
        assert!(number(bad).is_err())
    }
}
#[test]
fn config_rejects_unsafe_and_dangling() {
    let mut c = config("all", 1);
    c.safety.global_monitor_cooldown_ms = 120001;
    assert!(c.validate().is_err());
    c.safety.global_monitor_cooldown_ms = 5000;
    c.profiles[0].rules[0].device_ids.push("missing".into());
    assert!(c.validate().is_err())
}
#[test]
fn config_rejects_unknown_keys() {
    let mut v = json!(Config::default());
    v["unsafe"] = json!(true);
    assert!(serde_json::from_value::<Config>(v).is_err())
}
#[test]
fn threshold_validation() {
    let mut c = config("at-least", 2);
    c.profiles[0].rules[0].minimum_count = 4;
    assert!(c.validate().is_err())
}
fn usb(id: &str, pid: u16) -> UsbDevice {
    UsbDevice {
        model: Some("Test device".into()),
        id: id.into(),
        physical_id: "physical-receiver".into(),
        name: "Receiver".into(),
        manufacturer: "Test".into(),
        vid: 1,
        pid,
        serial: None,
        path: "bus:1".into(),
        parent: None,
        category: "hid".into(),
        connected: true,
        last_seen: "now".into(),
    }
}
#[test]
fn physical_interfaces_count_once() {
    let c = config("at-least", 2);
    assert!(presence(&c, &[usb("a", 0), usb("b", 1)]).is_err())
}
#[test]
fn duplicate_selector_blocks() {
    let c = config("all", 1);
    let a = usb("a", 0);
    let mut b = a.clone();
    b.physical_id = "other".into();
    assert!(presence(&c, &[a, b]).is_err())
}
#[test]
fn serial_matching_survives_new_path() {
    let mut u = usb("a", 0);
    u.serial = Some("stable".into());
    let s = Selector {
        id: "s".into(),
        strategy: "serial".into(),
        vid: 1,
        pid: 0,
        serial: Some("stable".into()),
        path: Some("old".into()),
        allow_ambiguous: false,
    };
    assert!(s.matches(&u));
    u.serial = Some("other".into());
    assert!(!s.matches(&u))
}
#[test]
fn instance_matching_checks_path() {
    let u = usb("a", 0);
    let mut s = Selector {
        id: "s".into(),
        strategy: "instance".into(),
        vid: 1,
        pid: 0,
        serial: None,
        path: Some(u.path.clone()),
        allow_ambiguous: false,
    };
    assert!(s.matches(&u));
    s.path = Some("other".into());
    assert!(!s.matches(&u))
}
fn monitor() -> Monitor {
    Monitor {
        id: "m".into(),
        name: "Desk".into(),
        fingerprint: "edid".into(),
        connector: "DP-1".into(),
        backend: "virtual".into(),
        path: "virtual".into(),
        edid_available: true,
        permissions: "granted".into(),
        limitations: vec!["simulation".into()],
    }
}
#[test]
fn monitor_ties_and_replacement_block() {
    let c = config("all", 1);
    let m = &c.monitors[0];
    assert!(resolve_monitor(m, &[]).is_err());
    assert!(resolve_monitor(m, &[monitor(), monitor()]).is_err());
    let mut other = monitor();
    other.fingerprint = "replacement".into();
    assert!(resolve_monitor(m, &[other]).is_err())
}
#[test]
fn parse_capabilities_preserves_vendor_values() {
    assert_eq!(
        platform::capability_inputs("(prot(monitor) vcp(10 12 60(0F 10 11 12 E1) D6(01 04)))"),
        vec![15, 16, 17, 18, 225]
    );
    assert!(platform::capability_inputs("garbage 60(11 12)").is_empty())
}
#[test]
fn atomic_and_backup() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(Some(temp.path().into())).unwrap();
    let mut initial = Config::default();
    initial.app.theme = "system".into();
    store.replace(initial).unwrap();
    store
        .mutate(|c| {
            c.app.theme = "dark".into();
            Ok(())
        })
        .unwrap();
    assert_eq!(store.load().unwrap().app.theme, "dark");
    let old: Config =
        serde_json::from_slice(&std::fs::read(temp.path().join("config.json.bak")).unwrap())
            .unwrap();
    assert_eq!(old.app.theme, "system");
    assert!(store
        .mutate(|c| {
            c.app.theme = "bad".into();
            Ok(())
        })
        .is_err());
    assert_eq!(store.load().unwrap().app.theme, "dark")
}
#[test]
fn concurrent_config_writers() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(Some(temp.path().into())).unwrap();
    let threads: Vec<_> = (0..10)
        .map(|_| {
            let s = store.clone();
            std::thread::spawn(move || {
                s.mutate(|c| {
                    c.app.startup_delay_ms += 1;
                    Ok(())
                })
                .unwrap()
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap()
    }
    assert_eq!(store.load().unwrap().app.startup_delay_ms, 10)
}
struct Fake {
    monitors: Vec<Monitor>,
    writes: Arc<Mutex<Vec<u8>>>,
}
impl platform::Hardware for Fake {
    fn monitors(&self) -> elarydil_kvm::Result<Vec<Monitor>> {
        Ok(self.monitors.clone())
    }
    fn ddc(
        &self,
        _: &Monitor,
        op: &str,
        _: u8,
        value: Option<u8>,
        _: u64,
    ) -> elarydil_kvm::Result<serde_json::Value> {
        assert_eq!(op, "write");
        self.writes.lock().unwrap().push(value.unwrap());
        Ok(json!({"verified":false}))
    }
}
#[test]
fn service_dispatch_missing_ambiguous_rate_and_confirm() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(Some(temp.path().into())).unwrap();
    let mut c = config("all", 1);
    // File ACL updates on Windows can exceed the default 100 ms between calls.
    // Use a long configured interval to test rejection independently of I/O speed.
    c.safety.global_monitor_cooldown_ms = 120_000;
    store.replace(c).unwrap();
    let writes = Arc::new(Mutex::new(vec![]));
    let mut service = Service::with_hardware(
        store,
        Box::new(Fake {
            monitors: vec![],
            writes: writes.clone(),
        }),
    )
    .unwrap();
    assert!(service.write("m", 96, 18, true, false).is_err());
    service.hardware = Arc::new(Fake {
        monitors: vec![monitor(), monitor()],
        writes: writes.clone(),
    });
    assert!(service.write("m", 96, 18, true, true).is_err());
    service.hardware = Arc::new(Fake {
        monitors: vec![monitor()],
        writes: writes.clone(),
    });
    assert!(service
        .execute("monitors.input.set", json!({"id":"m","value":18}))
        .is_err());
    service.write("m", 96, 18, true, false).unwrap();
    assert!(service.write("m", 96, 18, true, false).is_err());
    assert_eq!(*writes.lock().unwrap(), vec![18]);
}
#[test]
fn service_gui_cli_same_state_and_dry_simulation() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(Some(temp.path().into())).unwrap();
    store.replace(config("all", 1)).unwrap();
    let mut service = Service::new(store).unwrap();
    service
        .execute("rules.timing", json!({"id":"r","debounce-ms":777}))
        .unwrap();
    let r = service.execute("rules.show", json!({"id":"r"})).unwrap();
    assert_eq!(r["debounceMs"], 777);
    let sim = service
        .execute("rules.simulate", json!({"id":"r"}))
        .unwrap();
    assert_eq!(sim["hardwareWrites"], 0);
    assert_eq!(sim["simulation"], true);
    assert_eq!(sim["decisions"].as_array().unwrap().len(), 1)
}
#[test]
fn all_catalog_keys_used_by_cli_exist() {
    for op in elarydil_kvm::cli::OPS {
        for n in 1..=op.split('.').count() {
            let key = format!(
                "cli.op.{}",
                op.split('.').take(n).collect::<Vec<_>>().join(".")
            );
            assert!(elarydil_kvm::catalog()[&key].is_string(), "{key}")
        }
    }
    assert!(elarydil_kvm::catalog()
        .as_object()
        .unwrap()
        .iter()
        .all(|(_, v)| v.is_string()))
}
#[test]
fn clap_tree_is_valid() {
    elarydil_kvm::cli::command().debug_assert()
}
#[test]
fn stale_monotonic_ticks_are_ignored() {
    let c = config("all", 1);
    let mut e = Engine::default();
    arm(&mut e, &c, 1000);
    assert!(e.tick(&c, &state([false; 3]), 500).is_empty());
    assert!(e.tick(&c, &state([true; 3]), 1600).is_empty());
    e.tick(&c, &state([false; 3]), 2000);
    assert_eq!(e.tick(&c, &state([false; 3]), 2400).len(), 1)
}
#[cfg(unix)]
#[test]
fn helper_timeout_kills_process_group() {
    use std::os::unix::process::CommandExt;
    let mut c = std::process::Command::new("sh");
    c.args(["-c", "sleep 10"]).process_group(0);
    let start = std::time::Instant::now();
    let error = platform::bounded(c, None, 50).unwrap_err();
    assert_eq!(error.code, "timeout");
    assert!(start.elapsed().as_secs() < 2)
}
#[test]
fn group_truth_tables() {
    for mode in ["all", "any", "at-least"] {
        for mask in 0u8..8 {
            let c = config(mode, 2);
            let mut e = Engine::default();
            arm(&mut e, &c, 0);
            let p = state([mask & 1 == 0, mask & 2 == 0, mask & 4 == 0]);
            let edges = mask.count_ones();
            let needed = match mode {
                "all" => 3,
                "any" => 1,
                _ => 2,
            };
            let first = e.tick(&c, &p, 1000);
            assert!(first.is_empty());
            assert_eq!(
                e.tick(&c, &p, 1400).len(),
                usize::from(edges >= needed),
                "{mode} {mask}"
            );
        }
    }
}
#[test]
fn conflicting_rules_share_one_monitor_limiter_and_dry_run_writes_nothing() {
    for dry in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let store = Store::open(Some(temp.path().into())).unwrap();
        let mut c = config("all", 1);
        c.profiles[0].rules[0].dry_run = dry;
        c.profiles[0].rules[0].rearm_ms = 0;
        c.profiles[0].rules[0].debounce_ms = 0;
        let mut second = c.profiles[0].clone();
        second.id = "p2".into();
        second.rules[0].id = "r2".into();
        second.rules[0].actions[0].id = "a2".into();
        c.profiles.push(second);
        store.replace(c).unwrap();
        let writes = Arc::new(Mutex::new(vec![]));
        let mut service = Service::with_hardware(
            store,
            Box::new(Fake {
                monitors: vec![monitor()],
                writes: writes.clone(),
            }),
        )
        .unwrap();
        service.engine.reset(0, 0);
        service.epoch = std::time::Instant::now() - std::time::Duration::from_secs(10);
        let devices = (0..3)
            .map(|i| {
                let mut d = usb(&format!("usb{i}"), i);
                d.physical_id = format!("independent{i}");
                d
            })
            .collect();
        service.snapshot(devices, false);
        service.tick();
        service.snapshot(vec![], false);
        service.tick();
        service.tick();
        assert_eq!(writes.lock().unwrap().len(), usize::from(!dry));
    }
}
#[test]
fn diagnostic_export_redacts_serials_and_paths() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(Some(temp.path().into())).unwrap();
    let mut c = config("all", 1);
    c.usb_devices[0].selectors[0].serial = Some("private-serial".into());
    c.usb_devices[0].selectors[0].path = Some("private-path".into());
    store.replace(c).unwrap();
    let mut service = Service::with_hardware(
        store,
        Box::new(Fake {
            monitors: vec![monitor()],
            writes: Arc::new(Mutex::new(vec![])),
        }),
    )
    .unwrap();
    let report = service
        .execute("diagnostics.export", json!({}))
        .unwrap()
        .to_string();
    assert!(!report.contains("private-serial"));
    assert!(!report.contains("private-path"));
    assert!(report.contains("\"doctor\""));
}

#[test]
fn matching_inhibition_is_reported_as_degraded() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(Some(temp.path().into())).unwrap();
    store.replace(config("all", 1)).unwrap();
    let mut service = Service::new(store).unwrap();
    let a = usb("a", 0);
    let mut b = a.clone();
    b.physical_id = "second-physical-device".into();
    service.snapshot(vec![a, b], false);
    service.tick();
    assert_eq!(
        service.execute("status", json!({})).unwrap()["agent"],
        "degraded"
    );
    assert_eq!(service.last_inhibition.as_deref(), Some("ambiguous_device"));
}

#[test]
fn commands_match_both_directions_once_without_startup_execution() {
    let mut c = config("all", 1);
    let hooks = &mut c.profiles[0].rules[0].commands;
    hooks.enabled = true;
    hooks.on_connect = "echo connect".into();
    hooks.on_disconnect = "echo disconnect".into();
    let c = elarydil_kvm::commands::matching_config(&c);
    let mut engine = Engine::default();
    assert!(engine.tick(&c, &state([true; 3]), 0).is_empty());
    assert!(engine.tick(&c, &state([true; 3]), 500).is_empty());
    assert!(engine
        .tick(&c, &state([false, true, true]), 1000)
        .is_empty());
    assert!(engine.tick(&c, &state([false; 3]), 1100).is_empty());
    let away = engine.tick(&c, &state([false; 3]), 1500);
    assert_eq!(away.len(), 1);
    assert_eq!(away[0].rule_id, "disconnect:r");
    assert!(engine.tick(&c, &state([false; 3]), 2000).is_empty());
    engine.tick(&c, &state([true; 3]), 3000);
    let back = engine.tick(&c, &state([true; 3]), 3400);
    assert_eq!(back.len(), 1);
    assert_eq!(back[0].rule_id, "connect:r");
    assert!(engine.tick(&c, &state([true; 3]), 4000).is_empty());
}
#[test]
fn commands_respect_disabled_and_dry_run() {
    let mut c = config("any", 1);
    assert!(elarydil_kvm::commands::matching_config(&c).profiles[0]
        .rules
        .is_empty());
    let rule = &mut c.profiles[0].rules[0];
    rule.commands.enabled = true;
    rule.commands.on_disconnect = "echo away".into();
    rule.dry_run = true;
    let mut matching = elarydil_kvm::commands::matching_config(&c);
    let mut e = Engine::default();
    arm(&mut e, &matching, 0);
    e.tick(&matching, &state([false, true, true]), 1000);
    let d = e.tick(&matching, &state([false, true, true]), 1400);
    assert_eq!(d.len(), 1);
    assert!(d[0].dry_run);
    matching.profiles[0].enabled = false;
    let mut e = Engine::default();
    arm(&mut e, &matching, 0);
    e.tick(&matching, &state([false; 3]), 1000);
    assert!(e.tick(&matching, &state([false; 3]), 1400).is_empty());
}

#[test]
fn service_executes_automatic_disconnect_and_reconnect_hooks() {
    use std::time::{Duration, Instant};
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(Some(temp.path().into())).unwrap();
    let mut c = config("all", 1);
    let r = &mut c.profiles[0].rules[0];
    r.device_ids = vec!["d0".into()];
    r.commands = CommandHooks {
        enabled: true,
        working_directory: temp.path().to_string_lossy().into(),
        on_connect: if cfg!(windows) {
            "[IO.File]::WriteAllText('connect.txt','yes')"
        } else {
            "printf yes > connect.txt"
        }
        .into(),
        on_disconnect: if cfg!(windows) {
            "[IO.File]::WriteAllText('disconnect.txt','yes')"
        } else {
            "printf yes > disconnect.txt"
        }
        .into(),
        ..Default::default()
    };
    store.replace(c).unwrap();
    let mut s = Service::new(store).unwrap();
    let advance = |s: &mut Service, ms: u64| {
        s.epoch = Instant::now() - Duration::from_millis(ms);
        s.tick();
    };
    s.snapshot(vec![usb("u", 0)], false);
    advance(&mut s, 1500);
    advance(&mut s, 2100);
    assert!(!temp.path().join("connect.txt").exists());
    s.snapshot(vec![], false);
    advance(&mut s, 2200);
    advance(&mut s, 2700);
    let start = Instant::now();
    while !temp.path().join("disconnect.txt").exists() {
        s.tick();
        assert!(
            start.elapsed() < Duration::from_secs(12),
            "disconnect command did not execute"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    // Use an offset beyond shell startup time and rearm to keep ticks monotonic on Windows.
    advance(&mut s, 20000);
    advance(&mut s, 20600);
    s.snapshot(vec![usb("u", 0)], false);
    advance(&mut s, 21000);
    advance(&mut s, 21500);
    let start = Instant::now();
    while !temp.path().join("connect.txt").exists() {
        s.tick();
        assert!(
            start.elapsed() < Duration::from_secs(12),
            "connect command did not execute"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    s.cancel_commands();
    assert_eq!(
        std::fs::read_to_string(temp.path().join("connect.txt")).unwrap(),
        "yes"
    );
    assert_eq!(
        std::fs::read_to_string(temp.path().join("disconnect.txt")).unwrap(),
        "yes"
    );
}

#[test]
fn native_dispatch_preserves_typed_configuration_values() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(Some(temp.path().into())).unwrap();
    for (key, value) in [
        ("app.lastPage", json!("settings")),
        ("safety.defaultDebounceMs", json!(0)),
        ("app.launchGuiAtLogin", json!(false)),
    ] {
        elarydil_kvm::cli::dispatch(&store, "config.set", json!({"id":key,"value":value})).unwrap();
        assert_eq!(
            elarydil_kvm::cli::dispatch(&store, "config.get", json!({"id":key})).unwrap(),
            value
        );
    }
}

#[test]
fn owner_liveness_check_detects_an_exited_app() {
    assert!(elarydil_kvm::platform::process_alive(std::process::id()));
    #[cfg(unix)]
    let mut child = std::process::Command::new("true").spawn().unwrap();
    #[cfg(windows)]
    let mut child = std::process::Command::new("cmd")
        .args(["/C", "exit"])
        .spawn()
        .unwrap();
    let pid = child.id();
    child.wait().unwrap();
    assert!(!elarydil_kvm::platform::process_alive(pid));
}

#[test]
fn duplicate_profile_names_get_an_incrementing_suffix() {
    use elarydil_kvm::service::unique_profile_name;
    let profile = |id: &str, name: &str| elarydil_kvm::model::Profile {
        id: id.into(),
        name: name.into(),
        enabled: false,
        rules: vec![],
    };
    let list = vec![profile("a", "Desk"), profile("b", "Desk 2"), profile("c", "Lab 7")];
    assert_eq!(unique_profile_name(&list, "Home", None), "Home");
    assert_eq!(unique_profile_name(&list, "Desk", None), "Desk 3");
    assert_eq!(unique_profile_name(&list, "Desk 2", None), "Desk 3");
    assert_eq!(unique_profile_name(&list, "Lab 7", None), "Lab 8");
    // Renaming a profile to its own name keeps it.
    assert_eq!(unique_profile_name(&list, "Desk", Some("a")), "Desk");
}
