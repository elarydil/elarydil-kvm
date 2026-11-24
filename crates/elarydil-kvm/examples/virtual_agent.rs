//! Deterministic test-only IPC agent. Never packaged as a native hardware backend.
use elarydil_kvm::{model::*, platform::Hardware, service::Service, store::Store};
use serde_json::{json, Value};
struct Virtual(std::sync::atomic::AtomicU8);
impl Hardware for Virtual {
    fn monitors(&self) -> elarydil_kvm::Result<Vec<Monitor>> {
        Ok(vec![Monitor {
            id: "monitor_desk".into(),
            name: "Acer XV272U".into(),
            fingerprint: "test-edid".into(),
            connector: "HDMI-A-1".into(),
            backend: "virtual-test-only".into(),
            path: "test".into(),
            edid_available: true,
            permissions: "granted".into(),
            limitations: vec!["simulation_not_hardware".into()],
        }])
    }
    fn ddc(
        &self,
        _: &Monitor,
        op: &str,
        _: u8,
        value: Option<u8>,
        _: u64,
    ) -> elarydil_kvm::Result<Value> {
        Ok(match op {
            "read" => {
                json!({"value":self.0.load(std::sync::atomic::Ordering::SeqCst),"simulation":true})
            }
            "write" => {
                self.0
                    .store(value.unwrap(), std::sync::atomic::Ordering::SeqCst);
                json!({"outcome":"write_api_success","value":value,"simulation":true,"verified":false})
            }
            _ => json!({"raw":"vcp(60(0F 10 11 12))","values":[15,16,17,18],"simulation":true}),
        })
    }
}
fn usb() -> Vec<UsbDevice> {
    ["KVM keyboard", "Wireless mouse", "USB receiver"]
        .iter()
        .enumerate()
        .map(|(i, name)| UsbDevice {
            id: format!("usb_{i}"),
            physical_id: format!("test-physical-{i}"),
            name: name.to_string(),
            model: Some(["MX Keys Mini", "MX Master 3S", "Unifying Receiver"][i].into()),
            manufacturer: "Logitech".into(),
            vid: 1133,
            pid: 1000 + i as u16,
            serial: None,
            path: format!("test-port-{i}"),
            parent: Some("test-hub".into()),
            category: ["keyboard", "mouse", "receiver"][i].into(),
            connected: true,
            last_seen: chrono::Utc::now().to_rfc3339(),
        })
        .collect()
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let store = Store::open(None).unwrap();
    if args.get(1).is_some_and(|s| s == "request") {
        let result =
            elarydil_kvm::ipc::call(&store, &args[2], serde_json::from_str(&args[3]).unwrap());
        println!("{}", serde_json::to_string(&result).unwrap());
        return;
    }
    let _lock = store.instance_lock().unwrap();
    let mut service = Service::with_hardware(
        store.clone(),
        Box::new(Virtual(std::sync::atomic::AtomicU8::new(17))),
    )
    .unwrap();
    service.snapshot(usb(), true);
    let (tx, rx) = std::sync::mpsc::sync_channel(64);
    elarydil_kvm::ipc::listen(store, tx).unwrap();
    while !service.stop {
        if let Ok((req, reply)) = rx.recv_timeout(std::time::Duration::from_millis(25)) {
            let result = if req.command == "test.usb" {
                let selected = req.payload["present"].as_array().unwrap();
                service.snapshot(
                    usb()
                        .into_iter()
                        .filter(|u| selected.iter().any(|x| x == &u.id))
                        .collect(),
                    false,
                );
                Ok(json!({"simulation":true}))
            } else if req.command == "test.input.reset" {
                service.monitors.call(
                    &Virtual(std::sync::atomic::AtomicU8::new(17))
                        .monitors()
                        .unwrap()[0],
                    "write",
                    96,
                    Some(17),
                    1000,
                    15000,
                )
            } else {
                service.execute(&req.command, req.payload)
            };
            let _ = reply.send(elarydil_kvm::ipc::Response::from_result(req.id, result));
        }
        service.tick();
    }
}
