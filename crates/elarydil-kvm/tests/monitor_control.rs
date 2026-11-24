use elarydil_kvm::{
    model::*, monitor_control::Controller, platform::Hardware, service::Service, store::Store,
};
use serde_json::{json, Value};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
struct HardwareMock {
    input: AtomicU8,
    ignore_writes: AtomicBool,
    delay: AtomicUsize,
    active: AtomicUsize,
    peak: AtomicUsize,
    writes: Mutex<Vec<u8>>,
    fingerprint: Mutex<String>,
}
impl HardwareMock {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            input: AtomicU8::new(17),
            ignore_writes: AtomicBool::new(false),
            delay: AtomicUsize::new(0),
            active: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
            writes: Mutex::new(vec![]),
            fingerprint: Mutex::new("edid".into()),
        })
    }
    fn monitor(&self) -> Monitor {
        Monitor {
            id: "m".into(),
            name: "Desk".into(),
            fingerprint: self.fingerprint.lock().unwrap().clone(),
            connector: "DP-1".into(),
            backend: "virtual-test".into(),
            path: "test".into(),
            edid_available: true,
            permissions: "granted".into(),
            limitations: vec![],
        }
    }
}
impl Hardware for HardwareMock {
    fn monitors(&self) -> elarydil_kvm::Result<Vec<Monitor>> {
        Ok(vec![self.monitor()])
    }
    fn ddc(
        &self,
        _: &Monitor,
        op: &str,
        _: u8,
        value: Option<u8>,
        _: u64,
    ) -> elarydil_kvm::Result<Value> {
        let n = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.peak.fetch_max(n, Ordering::SeqCst);
        let result = if op == "write" {
            if !self.ignore_writes.load(Ordering::SeqCst) {
                self.input.store(value.unwrap(), Ordering::SeqCst);
            }
            self.writes.lock().unwrap().push(value.unwrap());
            json!({"outcome":"write_api_success","verified":false})
        } else {
            std::thread::sleep(Duration::from_millis(
                self.delay.load(Ordering::SeqCst) as u64
            ));
            json!({"value":self.input.load(Ordering::SeqCst)})
        };
        self.active.fetch_sub(1, Ordering::SeqCst);
        Ok(result)
    }
}
fn until(timeout: Duration, mut condition: impl FnMut() -> bool) {
    let start = Instant::now();
    while !condition() {
        assert!(start.elapsed() < timeout, "operation timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
}
#[test]
fn default_fifteen_second_test_returns_without_gui() {
    let temp = tempfile::tempdir().unwrap();
    let hw = HardwareMock::new();
    let c = Controller::new(hw.clone(), Store::open(Some(temp.path().into())).unwrap());
    let start = Instant::now();
    let test = c
        .call(&hw.monitor(), "test", 96, Some(18), 1000, 15000)
        .unwrap();
    assert_eq!(test["originalInput"], 17);
    assert_eq!(hw.input.load(Ordering::SeqCst), 18);
    until(Duration::from_secs(18), || {
        hw.input.load(Ordering::SeqCst) == 17
    });
    assert!(start.elapsed() >= Duration::from_millis(14900));
    assert_eq!(*hw.writes.lock().unwrap(), vec![18, 17]);
    assert_eq!(
        c.state(&hw.monitor(), false, 1000)["test"]["phase"],
        "restored"
    );
}
#[test]
fn manual_return_and_interrupted_test_recovery() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(Some(temp.path().into())).unwrap();
    let hw = HardwareMock::new();
    let c = Controller::new(hw.clone(), store.clone());
    c.call(&hw.monitor(), "test", 96, Some(18), 1000, 15000)
        .unwrap();
    let journal = std::fs::read(store.root.join("monitor-tests.json")).unwrap();
    c.call(&hw.monitor(), "restore", 96, None, 1000, 15000)
        .unwrap();
    assert_eq!(hw.input.load(Ordering::SeqCst), 17);
    drop(c);
    hw.input.store(18, Ordering::SeqCst);
    std::fs::write(store.root.join("monitor-tests.json"), journal).unwrap();
    let c = Controller::new(hw.clone(), store);
    until(Duration::from_secs(2), || {
        hw.input.load(Ordering::SeqCst) == 17
    });
    assert_eq!(
        c.state(&hw.monitor(), false, 1000)["test"]["phase"],
        "restored"
    );
}
#[test]
fn live_cache_does_not_block_and_ddc_operations_are_serialized() {
    let temp = tempfile::tempdir().unwrap();
    let hw = HardwareMock::new();
    hw.delay.store(600, Ordering::SeqCst);
    let c = Controller::new(hw.clone(), Store::open(Some(temp.path().into())).unwrap());
    c.state(&hw.monitor(), true, 1000);
    until(Duration::from_secs(1), || {
        hw.active.load(Ordering::SeqCst) > 0
    });
    let start = Instant::now();
    for _ in 0..100 {
        c.cached("m", true, 1000).unwrap();
    }
    assert!(start.elapsed() < Duration::from_millis(150));
    c.call(&hw.monitor(), "write", 96, Some(18), 1000, 15000)
        .unwrap();
    assert_eq!(hw.input.load(Ordering::SeqCst), 18);
    assert_eq!(hw.peak.load(Ordering::SeqCst), 1);
}
#[test]
fn test_refuses_unknown_return_and_never_restores_a_replacement_monitor() {
    let temp = tempfile::tempdir().unwrap();
    let hw = HardwareMock::new();
    let c = Controller::new(hw.clone(), Store::open(Some(temp.path().into())).unwrap());
    hw.input.store(0, Ordering::SeqCst);
    assert_eq!(
        c.call(&hw.monitor(), "test", 96, Some(18), 1000, 100)
            .unwrap_err()
            .code,
        "current_input_unknown"
    );
    assert!(hw.writes.lock().unwrap().is_empty());
    hw.input.store(17, Ordering::SeqCst);
    let original = hw.monitor();
    c.call(&original, "test", 96, Some(18), 1000, 150).unwrap();
    *hw.fingerprint.lock().unwrap() = "replacement".into();
    until(Duration::from_secs(2), || {
        c.state(&original, false, 1000)["test"]["phase"] == "restore-failed"
    });
    assert_eq!(*hw.writes.lock().unwrap(), vec![18]);
}
fn usb(i: u16) -> UsbDevice {
    UsbDevice {
        id: format!("usb_{i}"),
        physical_id: format!("physical_{i}"),
        name: format!("Device {i}"),
        model: None,
        manufacturer: "Test".into(),
        vid: 1,
        pid: i,
        serial: Some(format!("serial{i}")),
        path: format!("port{i}"),
        parent: None,
        category: "keyboard".into(),
        connected: true,
        last_seen: String::new(),
    }
}
#[test]
fn wizard_reports_first_edges_and_atomically_creates_two_live_rules() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(Some(temp.path().into())).unwrap();
    let mut config = Config::default();
    config.safety.startup_grace_ms = 0;
    store.replace(config).unwrap();
    let hw = HardwareMock::new();
    struct Wrapper(Arc<HardwareMock>);
    impl Hardware for Wrapper {
        fn monitors(&self) -> elarydil_kvm::Result<Vec<Monitor>> {
            self.0.monitors()
        }
        fn ddc(
            &self,
            m: &Monitor,
            op: &str,
            v: u8,
            value: Option<u8>,
            timeout: u64,
        ) -> elarydil_kvm::Result<Value> {
            self.0.ddc(m, op, v, value, timeout)
        }
    }
    let mut s = Service::with_hardware(store, Box::new(Wrapper(hw.clone()))).unwrap();
    s.snapshot(vec![usb(0), usb(1)], false);
    s.execute("wizard.start", json!({})).unwrap();
    assert_eq!(
        s.execute("wizard.candidates", json!({})).unwrap()["observations"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let capture_start = s.capture.as_ref().unwrap().started_mono_ms;
    s.snapshot_timed(vec![usb(1)], false, capture_start + 120);
    s.snapshot_timed(vec![], false, capture_start + 150);
    s.snapshot_timed(vec![usb(0)], false, capture_start + 300);
    s.snapshot_timed(vec![usb(0), usb(1)], false, capture_start + 450);
    let capture = s.execute("wizard.capture.finish", json!({})).unwrap();
    assert_eq!(capture["observations"].as_array().unwrap().len(), 2);
    assert_eq!(capture["observations"][0]["disconnectedAtMs"], 120);
    assert_eq!(capture["observations"][1]["connectedAtMs"], 450);
    let profile=s.execute("wizard.complete",json!({"devices":["usb_0","usb_1"],"connect-monitor":"m","connect-input":17,"disconnect-monitor":"m","disconnect-input":18})).unwrap();
    assert_eq!(profile["enabled"], true);
    assert_eq!(profile["rules"].as_array().unwrap().len(), 2);
    for r in profile["rules"].as_array().unwrap() {
        assert_eq!(r["dryRun"], false);
        assert_eq!(r["debounceMs"], 0);
    }
    assert!(hw.writes.lock().unwrap().is_empty());
    // The wizard primes its known baseline: the first switch is usable immediately.
    s.snapshot(vec![], false);
    s.tick();
    assert_eq!(hw.input.load(Ordering::SeqCst), 18);
}
#[test]
fn wizard_completes_with_separate_trigger_devices_and_without_a_capture() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(Some(temp.path().into())).unwrap();
    let mut config = Config::default();
    config.safety.startup_grace_ms = 0;
    store.replace(config).unwrap();
    let hw = HardwareMock::new();
    struct Wrapper(Arc<HardwareMock>);
    impl Hardware for Wrapper {
        fn monitors(&self) -> elarydil_kvm::Result<Vec<Monitor>> {
            self.0.monitors()
        }
        fn ddc(
            &self,
            m: &Monitor,
            op: &str,
            v: u8,
            value: Option<u8>,
            timeout: u64,
        ) -> elarydil_kvm::Result<Value> {
            self.0.ddc(m, op, v, value, timeout)
        }
    }
    let mut s = Service::with_hardware(store, Box::new(Wrapper(hw.clone()))).unwrap();
    s.snapshot(vec![usb(0), usb(1), usb(2)], false);
    // Manual setup: no capture session, devices come from the live inventory,
    // and each direction watches its own device set.
    let profile = s
        .execute(
            "wizard.complete",
            json!({"disconnect-devices":["usb_0"],"connect-devices":["usb_0","usb_2"],"connect-monitor":"m","connect-input":17,"disconnect-monitor":"m","disconnect-input":18}),
        )
        .unwrap();
    let rules = profile["rules"].as_array().unwrap();
    let ids = |event: &str| {
        rules.iter().find(|r| r["trigger"] == event).unwrap()["deviceIds"].clone()
    };
    assert_eq!(ids("disconnect"), json!(["usb_0"]));
    assert_eq!(ids("connect"), json!(["usb_0", "usb_2"]));
    let tracked = s.execute("config.show", json!({})).unwrap()["usbDevices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["id"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(tracked, vec!["usb_0", "usb_2"]);
    // An empty direction is rejected rather than creating a rule that never fires.
    let err = s
        .execute(
            "wizard.complete",
            json!({"disconnect-devices":[],"connect-devices":["usb_0"],"connect-monitor":"m","connect-input":17,"disconnect-monitor":"m","disconnect-input":18}),
        )
        .unwrap_err();
    assert_eq!(err.code, "invalid_input");
}
#[test]
fn setup_hold_suspends_switching_and_rearms_without_replaying_changes() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(Some(temp.path().into())).unwrap();
    let mut config = Config::default();
    config.safety.startup_grace_ms = 0;
    store.replace(config).unwrap();
    let hw = HardwareMock::new();
    struct Wrapper(Arc<HardwareMock>);
    impl Hardware for Wrapper {
        fn monitors(&self) -> elarydil_kvm::Result<Vec<Monitor>> {
            self.0.monitors()
        }
        fn ddc(
            &self,
            m: &Monitor,
            op: &str,
            v: u8,
            value: Option<u8>,
            timeout: u64,
        ) -> elarydil_kvm::Result<Value> {
            self.0.ddc(m, op, v, value, timeout)
        }
    }
    let mut s = Service::with_hardware(store, Box::new(Wrapper(hw.clone()))).unwrap();
    s.snapshot(vec![usb(0), usb(1)], false);
    s.execute(
        "wizard.complete",
        json!({"devices":["usb_0","usb_1"],"connect-monitor":"m","connect-input":17,"disconnect-monitor":"m","disconnect-input":18}),
    )
    .unwrap();
    s.execute("wizard.hold", json!({"ms":60000})).unwrap();
    assert_eq!(s.execute("status", json!({})).unwrap()["setupHold"], true);
    // Switching the KVM while setup is open must not touch the monitor.
    s.snapshot(vec![], false);
    s.tick();
    assert!(hw.writes.lock().unwrap().is_empty());
    s.execute("wizard.hold", json!({"release":true})).unwrap();
    s.tick();
    assert!(hw.writes.lock().unwrap().is_empty(), "held change must not replay");
    // Automation is armed again for the next real switch.
    s.snapshot(vec![usb(0), usb(1)], false);
    s.tick();
    assert_eq!(hw.input.load(Ordering::SeqCst), 17);
}
#[test]
fn ignored_return_never_fakes_the_current_input_and_readback_recovers() {
    let temp = tempfile::tempdir().unwrap();
    let hw = HardwareMock::new();
    let c = Controller::new(hw.clone(), Store::open(Some(temp.path().into())).unwrap());
    c.call(&hw.monitor(), "test", 96, Some(18), 1000, 150)
        .unwrap();
    hw.ignore_writes.store(true, Ordering::SeqCst);
    until(Duration::from_secs(3), || {
        c.state(&hw.monitor(), false, 1000)["test"]["phase"] == "restore-failed"
    });
    assert_eq!(c.state(&hw.monitor(), false, 1000)["currentInput"], 18);
    hw.input.store(17, Ordering::SeqCst);
    c.state(&hw.monitor(), true, 1000);
    until(Duration::from_secs(2), || {
        c.state(&hw.monitor(), true, 1000)["test"]["phase"] == "restored"
    });
}
