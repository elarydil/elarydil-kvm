//! Serialized DDC I/O, leased live reads, and agent-owned reversible tests.
use crate::{model::*, platform::Hardware, store::Store, Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant},
};
fn wall() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Test {
    id: String,
    monitor: Monitor,
    original: u8,
    target: u8,
    vcp: u8,
    deadline_ms: i64,
    timeout_ms: u64,
    phase: String,
    #[serde(default)]
    error: Option<Error>,
}
#[derive(Clone)]
struct Watch {
    monitor: Monitor,
    expires: Instant,
    next: Instant,
    timeout: u64,
    failures: u32,
}
#[derive(Default)]
struct Shared {
    known: HashMap<String, Monitor>,
    watches: HashMap<String, Watch>,
    states: HashMap<String, Value>,
}
struct Request {
    monitor: Monitor,
    op: String,
    vcp: u8,
    value: Option<u8>,
    timeout: u64,
    restore_ms: u64,
    reply: mpsc::SyncSender<Result<Value>>,
}
pub struct Controller {
    shared: Arc<Mutex<Shared>>,
    tx: Option<mpsc::SyncSender<Request>>,
    events: mpsc::Receiver<(String, Value)>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Controller {
    pub fn new(hardware: Arc<dyn Hardware>, store: Store) -> Self {
        let shared = Arc::new(Mutex::new(Shared::default()));
        let worker = shared.clone();
        let (tx, rx) = mpsc::sync_channel::<Request>(16);
        let (events, out) = mpsc::channel();
        let thread = std::thread::spawn(move || {
            let mut tests: HashMap<String, Test> =
                std::fs::read(store.root.join("monitor-tests.json"))
                    .ok()
                    .and_then(|v| serde_json::from_slice(&v).ok())
                    .unwrap_or_default();
            let mut deadlines: HashMap<String, Instant> = tests
                .keys()
                .map(|id| (id.clone(), Instant::now()))
                .collect();
            // A restarted agent returns interrupted tests before resuming automation.
            for test in tests.values_mut() {
                test.deadline_ms = wall();
            }
            let persist = |tests: &HashMap<String, Test>| {
                let _ = store.atomic("monitor-tests.json", &serde_json::to_vec(tests).unwrap());
            };
            let publish = |test: &Test, shared: &Arc<Mutex<Shared>>| {
                let mut s = shared.lock().unwrap();
                let state = s
                    .states
                    .entry(test.monitor.id.clone())
                    .or_insert_with(|| json!({"monitorId":test.monitor.id}));
                state["test"] = json!({"id":test.id,"originalInput":test.original,"targetInput":test.target,"deadlineMs":test.deadline_ms,"phase":test.phase,"error":test.error,"finishedAtMs":if test.phase=="restored"{Some(wall())}else{None}});
            };
            let restore = |test: &mut Test| -> Result<Value> {
                let live = hardware.monitors()?;
                let saved = SavedMonitor {
                    id: test.monitor.id.clone(),
                    alias: test.monitor.name.clone(),
                    fingerprint: test.monitor.fingerprint.clone(),
                    connector: Some(test.monitor.connector.clone()),
                    input_options: vec![],
                    hidden_inputs: vec![],
                };
                let current = resolve_monitor(&saved, &live)?;
                let submitted = hardware.ddc(
                    current,
                    "write",
                    test.vcp,
                    Some(test.original),
                    test.timeout_ms,
                )?;
                let mut failure = Error::new("restore_not_verified");
                for attempt in 0..2 {
                    let read = hardware.ddc(current, "read", test.vcp, None, test.timeout_ms);
                    update(&worker, &events, &test.monitor.id, &read);
                    if let Ok(value) = &read {
                        if value["value"].as_u64() == Some(test.original as u64) {
                            return Ok(
                                json!({"submitted":submitted,"value":test.original,"verified":true}),
                            );
                        }
                    }
                    failure = read
                        .err()
                        .unwrap_or_else(|| Error::new("restore_not_verified"));
                    if attempt == 0 {
                        std::thread::sleep(Duration::from_millis(150));
                    }
                }
                Err(failure)
            };
            loop {
                let due = tests
                    .iter()
                    .filter(|(id, _)| deadlines.get(*id).is_some_and(|d| *d <= Instant::now()))
                    .map(|(id, _)| id.clone())
                    .collect::<Vec<_>>();
                for id in due {
                    if let Some(mut test) = tests.remove(&id) {
                        deadlines.remove(&id);
                        test.phase = "restoring".into();
                        publish(&test, &worker);
                        let result = restore(&mut test);
                        test.phase = if result.is_ok() {
                            "restored"
                        } else {
                            "restore-failed"
                        }
                        .into();
                        test.error = result.as_ref().err().cloned();
                        publish(&test, &worker);
                        let _ = events.send((
                            if result.is_ok() {
                                "monitor.test_restored"
                            } else {
                                "monitor.test_restore_failed"
                            }
                            .into(),
                            json!({"monitorId":id,"inputValue":test.original,"result":result}),
                        ));
                        // Retain failed rollback on disk for recovery after restart or manual retry.
                        if result.is_err() {
                            test.deadline_ms = wall() + 60000;
                            deadlines.insert(id.clone(), Instant::now() + Duration::from_secs(60));
                            tests.insert(id, test);
                        }
                        persist(&tests);
                    }
                }
                match rx.recv_timeout(Duration::from_millis(20)) {
                    Ok(req) => {
                        let id = req.monitor.id.clone();
                        let result = (|| -> Result<Value> {
                            if req.op == "restore" {
                                let mut test =
                                    tests.remove(&id).ok_or_else(|| Error::new("not_found"))?;
                                deadlines.remove(&id);
                                let result = restore(&mut test);
                                test.phase = if result.is_ok() {
                                    "restored"
                                } else {
                                    "restore-failed"
                                }
                                .into();
                                test.error = result.as_ref().err().cloned();
                                publish(&test, &worker);
                                if result.is_err() {
                                    test.deadline_ms = wall() + 60000;
                                    deadlines.insert(
                                        id.clone(),
                                        Instant::now() + Duration::from_secs(60),
                                    );
                                    tests.insert(id.clone(), test);
                                }
                                persist(&tests);
                                return result;
                            }
                            if req.op == "test" {
                                if tests.contains_key(&id) {
                                    return Err(Error::new("test_in_progress"));
                                }
                                let read = hardware.ddc(
                                    &req.monitor,
                                    "read",
                                    req.vcp,
                                    None,
                                    req.timeout,
                                )?;
                                let original = read["value"]
                                    .as_u64()
                                    .and_then(|v| u8::try_from(v).ok())
                                    .filter(|v| *v > 0)
                                    .ok_or_else(|| Error::new("current_input_unknown"))?;
                                let mut test = Test {
                                    id: crate::model::id("test"),
                                    monitor: req.monitor.clone(),
                                    original,
                                    target: req.value.unwrap(),
                                    vcp: req.vcp,
                                    deadline_ms: wall() + req.restore_ms as i64,
                                    timeout_ms: req.timeout,
                                    phase: "switching".into(),
                                    error: None,
                                };
                                deadlines.insert(
                                    id.clone(),
                                    Instant::now() + Duration::from_millis(req.restore_ms),
                                );
                                tests.insert(id.clone(), test.clone());
                                persist(&tests);
                                if let Err(e) = hardware.ddc(
                                    &req.monitor,
                                    "write",
                                    req.vcp,
                                    req.value,
                                    req.timeout,
                                ) {
                                    test.phase = "restore-failed".into();
                                    test.error = Some(e.clone());
                                    test.deadline_ms = wall();
                                    deadlines.insert(id.clone(), Instant::now());
                                    tests.insert(id.clone(), test.clone());
                                    publish(&test, &worker);
                                    persist(&tests);
                                    return Err(e);
                                }
                                test.deadline_ms = wall() + req.restore_ms as i64;
                                test.phase = "active".into();
                                deadlines.insert(
                                    id.clone(),
                                    Instant::now() + Duration::from_millis(req.restore_ms),
                                );
                                tests.insert(id.clone(), test.clone());
                                persist(&tests);
                                publish(&test, &worker);
                                let _=events.send(("monitor.test_started".into(),json!({"monitorId":id,"target":test.target,"original":original,"deadlineMs":test.deadline_ms})));
                                return Ok(
                                    json!({"testId":test.id,"monitorId":id,"originalInput":original,"targetInput":test.target,"deadlineMs":test.deadline_ms,"phase":"active"}),
                                );
                            }
                            if tests.contains_key(&id) && req.op == "write" {
                                return Err(Error::new("test_in_progress"));
                            }
                            hardware.ddc(&req.monitor, &req.op, req.vcp, req.value, req.timeout)
                        })();
                        if req.op == "read" {
                            update(&worker, &events, &id, &result);
                        }
                        let _ = req.reply.send(result);
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        for test in tests.values_mut() {
                            let result = restore(test);
                            test.error = result.as_ref().err().cloned();
                            test.phase = if result.is_ok() {
                                "restored"
                            } else {
                                "restore-failed"
                            }
                            .into();
                        }
                        tests.retain(|_, t| t.phase != "restored");
                        persist(&tests);
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                let watch = {
                    let mut s = worker.lock().unwrap();
                    s.watches.retain(|_, w| w.expires > Instant::now());
                    s.watches
                        .values()
                        .filter(|w| w.next <= Instant::now())
                        .min_by_key(|w| w.next)
                        .cloned()
                };
                if let Some(w) = watch {
                    // Reserve enough time for the due rollback before starting a read.
                    if deadlines.values().any(|due| {
                        due.saturating_duration_since(Instant::now())
                            <= Duration::from_millis(w.timeout + 50)
                    }) {
                        continue;
                    }

                    let read_started = Instant::now();
                    let result = hardware.ddc(&w.monitor, "read", 96, None, w.timeout);
                    update(&worker, &events, &w.monitor.id, &result);
                    if tests.get(&w.monitor.id).is_some_and(|t| {
                        t.phase == "restore-failed"
                            && result.as_ref().ok().and_then(|v| v["value"].as_u64())
                                == Some(t.original as u64)
                    }) {
                        let mut t = tests.remove(&w.monitor.id).unwrap();
                        deadlines.remove(&w.monitor.id);
                        t.phase = "restored".into();
                        t.error = None;
                        publish(&t, &worker);
                        persist(&tests);
                    }
                    let mut s = worker.lock().unwrap();
                    if let Some(w) = s.watches.get_mut(&w.monitor.id) {
                        w.failures = if result.is_ok() {
                            0
                        } else {
                            w.failures.saturating_add(1)
                        };
                        w.next = (if w.failures == 0 {
                            read_started
                        } else {
                            Instant::now()
                        }) + Duration::from_millis(if w.failures == 0 {
                            300
                        } else {
                            (500u64.saturating_mul(1u64 << w.failures.min(3))).min(4000)
                        });
                    }
                }
            }
        });
        Self {
            shared,
            tx: Some(tx),
            events: out,
            thread: Some(thread),
        }
    }
    pub fn call(
        &self,
        monitor: &Monitor,
        op: &str,
        vcp: u8,
        value: Option<u8>,
        timeout: u64,
        restore_ms: u64,
    ) -> Result<Value> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.tx
            .as_ref()
            .unwrap()
            .try_send(Request {
                monitor: monitor.clone(),
                op: op.into(),
                vcp,
                value,
                timeout,
                restore_ms,
                reply: tx,
            })
            .map_err(|_| Error::new("ddc_busy"))?;
        rx.recv_timeout(Duration::from_millis(timeout.saturating_mul(4) + 1000))
            .map_err(|_| Error::new("timeout"))?
    }
    pub fn state(&self, monitor: &Monitor, watch: bool, timeout: u64) -> Value {
        let mut s = self.shared.lock().unwrap();
        s.known.insert(monitor.id.clone(), monitor.clone());
        if watch {
            s.watches
                .entry(monitor.id.clone())
                .and_modify(|w| {
                    w.monitor = monitor.clone();
                    w.expires = Instant::now() + Duration::from_secs(2);
                    w.timeout = timeout;
                })
                .or_insert(Watch {
                    monitor: monitor.clone(),
                    expires: Instant::now() + Duration::from_secs(2),
                    next: Instant::now(),
                    timeout,
                    failures: 0,
                });
        }
        s.states.get(&monitor.id).cloned().unwrap_or_else(
            || json!({"monitorId":monitor.id,"currentInput":null,"ddcAvailable":null}),
        )
    }
    pub fn cached(&self, id: &str, watch: bool, timeout: u64) -> Option<Value> {
        let monitor = self.shared.lock().unwrap().known.get(id).cloned()?;
        Some(self.state(&monitor, watch, timeout))
    }
    pub fn drain(&self) -> Vec<(String, Value)> {
        self.events.try_iter().collect()
    }
}
fn update(
    shared: &Arc<Mutex<Shared>>,
    events: &mpsc::Sender<(String, Value)>,
    id: &str,
    result: &Result<Value>,
) {
    let mut s = shared.lock().unwrap();
    let state = s
        .states
        .entry(id.into())
        .or_insert_with(|| json!({"monitorId":id}));
    match result {
        Ok(v) => {
            if let Some(value) = v["value"].as_u64() {
                let changed = state["currentInput"] != value || state["ddcAvailable"] != true;
                state["currentInput"] = json!(value);
                state["ddcAvailable"] = json!(true);
                state["lastReadMs"] = json!(wall());
                state["error"] = Value::Null;
                if changed {
                    let _ = events.send((
                        "monitor.input_read".into(),
                        json!({"monitorId":id,"value":value}),
                    ));
                }
            }
        }
        Err(e) => {
            let changed = state["ddcAvailable"] != false;
            state["ddcAvailable"] = json!(false);
            state["error"] = json!(e);
            if changed {
                let _ = events.send((
                    "monitor.read_failed".into(),
                    json!({"monitorId":id,"error":e}),
                ));
            }
        }
    }
}
impl Drop for Controller {
    fn drop(&mut self) {
        self.tx.take();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
