use crate::{engine::Engine, model::*, platform, store::Store, Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Write,
    time::{Duration, Instant},
};
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capture {
    #[serde(default)]
    pub started_mono_ms: u64,
    #[serde(default)]
    pub observations: Vec<CaptureObservation>,
    pub id: String,
    pub cycles: usize,
    pub active: bool,
    pub started: String,
    pub baseline: Vec<UsbDevice>,
    pub counts: HashMap<String, usize>,
    pub away: HashSet<String>,
    pub events: Vec<Value>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureObservation {
    pub device: UsbDevice,
    pub disconnected_at_ms: Option<u64>,
    pub connected_at_ms: Option<u64>,
    pub last_event: String,
}
pub struct Service {
    pub hardware: std::sync::Arc<dyn platform::Hardware>,
    pub monitors: crate::monitor_control::Controller,
    pub store: Store,
    pub config: Config,
    pub usb: Vec<UsbDevice>,
    pub usb_ok: bool,
    pub paused: bool,
    pub stop: bool,
    pub engine: Engine,
    command_engine: Engine,
    command_config: Config,
    command_runner: Option<crate::commands::Runner>,
    pub epoch: Instant,
    pub last_change: u64,
    pub last_write: HashMap<String, u64>,
    pub history: Vec<Value>,
    pub capture: Option<Capture>,
    pub sequence: u64,
    pub last_inhibition: Option<String>,
    /// While the setup wizard is open, rules must not switch monitors: the
    /// user is deliberately toggling the KVM. The GUI renews this lease, so a
    /// crashed window can never leave automation suspended for long.
    pub setup_hold_until: u64,
    setup_held: bool,
    /// The desktop app that owns this agent. Unless the user opted into a
    /// standalone agent, the agent exits once its owner is gone.
    pub owner_pid: Option<u32>,
}
pub fn arg<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| Error::new("invalid_input").detail(key))
}
fn text(v: &Value, k: &str, default: &str) -> String {
    v[k].as_str().unwrap_or(default).into()
}
fn uint(v: &Value, k: &str, default: u64) -> u64 {
    v[k].as_u64()
        .or_else(|| v[k].as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(default)
}
fn flag(v: &Value, k: &str) -> bool {
    v[k].as_bool().unwrap_or(false)
}
fn bytes(v: &Value, k: &str, default: u8) -> Result<u8> {
    match &v[k] {
        Value::Null => Ok(default),
        Value::String(s) => number(s),
        Value::Number(n) => n
            .as_u64()
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| Error::new("invalid_input")),
        _ => Err(Error::new("invalid_input")),
    }
}
fn find_profile<'a>(c: &'a mut Config, id: &str) -> Result<&'a mut Profile> {
    c.profiles
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| Error::new("not_found"))
}
fn find_rule<'a>(c: &'a mut Config, id: &str) -> Result<&'a mut Rule> {
    c.profiles
        .iter_mut()
        .flat_map(|p| &mut p.rules)
        .find(|r| r.id == id)
        .ok_or_else(|| Error::new("not_found"))
}
/// Profile names stay unique: a taken name gets " 2", and a name that already
/// ends in " <number>" continues from that number ("Desk 2" -> "Desk 3").
pub fn unique_profile_name(profiles: &[Profile], desired: &str, except: Option<&str>) -> String {
    let desired = desired.trim();
    let taken = |name: &str| {
        profiles
            .iter()
            .any(|p| Some(p.id.as_str()) != except && p.name == name)
    };
    if !taken(desired) {
        return desired.to_owned();
    }
    let (base, mut n) = match desired.rsplit_once(' ') {
        Some((base, number)) if !base.is_empty() && number.parse::<u64>().is_ok() => {
            (base, number.parse::<u64>().unwrap_or(1) + 1)
        }
        _ => (desired, 2),
    };
    loop {
        let candidate = format!("{base} {n}");
        if !taken(&candidate) {
            return candidate;
        }
        n += 1;
    }
}
fn split_devices(v: &Value) -> Result<Vec<String>> {
    device_list(v, "devices")
}
fn device_list(v: &Value, key: &str) -> Result<Vec<String>> {
    if let Some(a) = v[key].as_array() {
        return a
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| Error::new("invalid_input"))
            })
            .collect();
    }
    Ok(arg(v, key)?
        .split(',')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect())
}
impl Service {
    pub fn new(store: Store) -> Result<Self> {
        Self::with_hardware(store, Box::new(platform::Native))
    }
    pub fn with_hardware(store: Store, hardware: Box<dyn platform::Hardware>) -> Result<Self> {
        let config = store.load()?;
        let hardware: std::sync::Arc<dyn platform::Hardware> = hardware.into();
        let monitors = crate::monitor_control::Controller::new(hardware.clone(), store.clone());
        let mut capture: Option<Capture> = fs::read(store.root.join("capture.json"))
            .ok()
            .and_then(|v| serde_json::from_slice(&v).ok());
        if let Some(c) = &mut capture {
            c.active = false;
        }
        let history = fs::read_to_string(store.root.join("events.jsonl"))
            .unwrap_or_default()
            .lines()
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect::<Vec<Value>>();
        let sequence = history
            .last()
            .and_then(|v| v["monotonicSequence"].as_u64())
            .unwrap_or(0);
        let mut engine = Engine::default();
        engine.reset(
            0,
            config
                .safety
                .startup_grace_ms
                .saturating_add(config.app.startup_delay_ms),
        );
        let mut command_engine = Engine::default();
        command_engine.reset(0, engine.grace_until);
        let command_config = crate::commands::matching_config(&config);
        Ok(Self {
            command_engine,
            command_config,
            command_runner: None,
            hardware,
            monitors,
            store,
            config,
            usb: vec![],
            usb_ok: false,
            paused: false,
            stop: false,
            engine,
            epoch: Instant::now(),
            last_change: 0,
            last_write: HashMap::new(),
            history,
            capture,
            sequence,
            last_inhibition: None,
            setup_hold_until: 0,
            setup_held: false,
            owner_pid: std::env::var("ELARYDIL_KVM_OWNER_PID")
                .ok()
                .and_then(|v| v.parse().ok()),
        })
    }
    fn drain_commands(&mut self) {
        let events = self
            .command_runner
            .as_ref()
            .map(|r| r.drain())
            .unwrap_or_default();
        for (kind, data) in events {
            self.event(kind, data);
        }
    }
    pub fn cancel_commands(&mut self) {
        if let Some(mut runner) = self.command_runner.take() {
            runner.stop();
            for (kind, data) in runner.drain() {
                self.event(kind, data);
            }
        }
    }
    fn reset_engines(&mut self, grace: u64) {
        self.engine.reset(self.now(), grace);
        self.command_engine.reset(self.now(), grace);
        self.cancel_commands();
    }
    fn apply_config(&mut self, config: Config) {
        let previous = json!([
            self.config.profiles,
            self.config.usb_devices,
            self.config.safety,
            self.config.app.startup_delay_ms
        ]);
        let next = json!([
            config.profiles,
            config.usb_devices,
            config.safety,
            config.app.startup_delay_ms
        ]);
        self.config = config;
        if previous != next {
            self.command_config = crate::commands::matching_config(&self.config);
            self.reset_engines(self.config.safety.startup_grace_ms);
        }
    }
    fn queue_command(&mut self, rule_id: &str, event: &str, manual: bool) -> Result<Value> {
        if !["connect", "disconnect"].contains(&event) {
            return Err(Error::new("invalid_input"));
        }
        let (profile, rule) = self
            .config
            .profiles
            .iter()
            .find_map(|p| p.rules.iter().find(|r| r.id == rule_id).map(|r| (p, r)))
            .ok_or_else(|| Error::new("not_found"))?;
        rule.commands.validate()?;
        if rule.commands.command(event).trim().is_empty() {
            return Err(Error::new("invalid_command_config"));
        }
        let job = crate::commands::Job {
            id: crate::model::id("command"),
            rule_id: rule.id.clone(),
            profile_id: profile.id.clone(),
            event: event.into(),
            manual,
            hooks: rule.commands.clone(),
        };
        let metadata = job.metadata();
        self.command_runner
            .get_or_insert_with(crate::commands::Runner::new)
            .queue(job)?;
        self.event("command.queued", metadata.clone());
        Ok(metadata)
    }
    pub fn now(&self) -> u64 {
        self.epoch.elapsed().as_millis() as u64
    }
    pub fn event(&mut self, kind: &str, result: Value) {
        self.sequence += 1;
        let event = json!({"timestamp":chrono::Utc::now().to_rfc3339(),"monotonicSequence":self.sequence,"severity":if kind.contains("failed")||kind.contains("degraded"){"error"}else{"info"},"eventType":kind,"result":result});
        self.history.push(event.clone());
        if self.history.len() > 10000 {
            self.history.drain(..1000);
        }
        let path = self.store.root.join("events.jsonl");
        let max = self.config.app.log_max_bytes;
        let cutoff =
            chrono::Utc::now() - chrono::Duration::days(self.config.app.log_retention_days as i64);
        if fs::metadata(&path).is_ok_and(|m| m.len() > max) {
            self.history.retain(|e| {
                e["timestamp"]
                    .as_str()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .is_some_and(|t| t.with_timezone(&chrono::Utc) > cutoff)
            });
            let keep = self.history.len() / 2;
            self.history.drain(..keep);
            let content = self
                .history
                .iter()
                .map(|e| format!("{e}\n"))
                .collect::<String>();
            let _ = self.store.atomic("events.jsonl", content.as_bytes());
        } else if let Ok(mut f) = fs::OpenOptions::new().append(true).create(true).open(path) {
            let _ = writeln!(f, "{event}");
        }
    }
    pub fn snapshot(&mut self, usb: Vec<UsbDevice>, resync: bool) {
        self.snapshot_timed(usb, resync, self.now());
    }
    pub fn snapshot_timed(&mut self, usb: Vec<UsbDevice>, resync: bool, observed_ms: u64) {
        let previous = self.usb.clone();
        let before: HashSet<_> = self.usb.iter().map(|u| u.id.clone()).collect();
        let after: HashSet<_> = usb.iter().map(|u| u.id.clone()).collect();
        if before != after {
            self.last_change = self.now();
            for id in before.symmetric_difference(&after) {
                self.event(
                    if after.contains(id) {
                        "usb.connected"
                    } else {
                        "usb.disconnected"
                    },
                    json!({"deviceId":id}),
                );
            }
        }
        self.usb = usb;
        self.usb_ok = true;
        if resync {
            self.reset_engines(self.config.safety.resume_grace_ms);
            self.event("agent.resync", json!({}));
        }
        if let Some(c) = &mut self.capture {
            if c.active && !resync {
                for id in before.symmetric_difference(&after) {
                    let connected = after.contains(id);
                    if let Some(device) =
                        self.usb.iter().chain(previous.iter()).find(|d| &d.id == id)
                    {
                        let identity = |d: &UsbDevice| {
                            if let Some(serial) = &d.serial {
                                format!("{}:{}:{}", d.vid, d.pid, serial)
                            } else {
                                d.physical_id.clone()
                            }
                        };
                        let key = identity(device);
                        let index = c
                            .observations
                            .iter()
                            .position(|o| identity(&o.device) == key)
                            .unwrap_or_else(|| {
                                c.observations.push(CaptureObservation {
                                    device: device.clone(),
                                    disconnected_at_ms: None,
                                    connected_at_ms: None,
                                    last_event: String::new(),
                                });
                                c.observations.len() - 1
                            });
                        let observation = &mut c.observations[index];
                        observation.device = device.clone();
                        observation.device.connected = connected;
                        let elapsed = observed_ms.saturating_sub(c.started_mono_ms);
                        if connected {
                            observation.connected_at_ms = Some(elapsed);
                        } else {
                            observation.disconnected_at_ms = Some(elapsed);
                        }
                        observation.last_event =
                            if connected { "connect" } else { "disconnect" }.into();
                        if !c.baseline.iter().any(|d| identity(d) == key) {
                            c.baseline.push(device.clone());
                        }
                    }
                }
                for d in &c.baseline {
                    let present = self.usb.iter().any(|u| {
                        u.vid == d.vid
                            && u.pid == d.pid
                            && if d.serial.is_some() {
                                u.serial == d.serial
                            } else {
                                true
                            }
                    });
                    if !present {
                        c.away.insert(d.id.clone());
                    } else if c.away.remove(&d.id) {
                        *c.counts.entry(d.id.clone()).or_default() += 1;
                    }
                }
                c.events
                    .push(json!({"timestamp":chrono::Utc::now().to_rfc3339(),"present":after}));
                if c.events.len() > 2000 {
                    c.events.remove(0);
                }
                let _ = self
                    .store
                    .atomic("capture.json", &serde_json::to_vec(c).unwrap_or_default());
            }
        }
    }
    fn drain_monitors(&mut self) {
        for (kind, data) in self.monitors.drain() {
            if kind == "monitor.input_read" {
                if let (Some(id), Some(value)) =
                    (data["monitorId"].as_str(), data["value"].as_u64())
                {
                    if let Some(m) = self.config.monitors.iter().find(|m| m.id == id) {
                        let input = m
                            .inputs()
                            .into_iter()
                            .find(|i| i.value as u64 == value)
                            .or_else(|| {
                                (value > 0 && value <= 255).then(|| Input {
                                    value: value as u8,
                                    vcp_code: 96,
                                    label: crate::t("ui.inputNumber")
                                        .replace("{value}", &value.to_string()),
                                    verified: false,
                                })
                            });
                        if let Some(mut input) =
                            input.filter(|i| !i.verified && !m.hidden_inputs.contains(&i.value))
                        {
                            input.verified = true;
                            if self
                                .store
                                .mutate(|c| {
                                    if let Some(m) = c.monitors.iter_mut().find(|m| m.id == id) {
                                        m.input_options.retain(|i| {
                                            !(i.value == input.value
                                                && i.vcp_code == input.vcp_code)
                                        });
                                        m.input_options.push(input);
                                    }
                                    Ok(())
                                })
                                .is_ok()
                            {
                                if let Ok(c) = self.store.load() {
                                    self.config = c;
                                }
                            }
                        }
                    }
                }
            }
            self.event(&kind, data);
        }
    }
    fn ensure_monitor(&mut self, id: &str) -> Result<SavedMonitor> {
        if let Some(m) = self.config.monitors.iter().find(|m| m.id == id) {
            return Ok(m.clone());
        }
        self.execute("monitors.remember", json!({"id":id}))?;
        self.config
            .monitors
            .iter()
            .find(|m| m.id == id)
            .cloned()
            .ok_or_else(|| Error::new("not_found"))
    }
    pub fn tick(&mut self) {
        self.drain_monitors();
        self.drain_commands();
        if self.paused || !self.usb_ok {
            return;
        }
        let now = self.now();
        if now < self.setup_hold_until {
            self.setup_held = true;
            return;
        }
        if self.setup_held {
            // Re-arm from the current USB state so changes made during setup
            // are never replayed as a switch once the hold ends.
            self.setup_held = false;
            if let Ok(present) = presence(&self.config, &self.usb) {
                self.engine.prime(&self.config, &present, now);
                self.command_engine
                    .prime(&self.command_config, &present, now);
            }
        }
        let present = match presence(&self.config, &self.usb) {
            Ok(p) => p,
            Err(e) => {
                self.reset_engines(1000);
                if self.last_inhibition.as_ref() != Some(&e.code) {
                    self.last_inhibition = Some(e.code.clone());
                    self.event("agent.degraded", json!(e));
                }
                return;
            }
        };
        self.last_inhibition = None;
        let commands = self
            .command_engine
            .tick(&self.command_config, &present, now);
        for decision in commands {
            if let Some((event, id)) = decision.rule_id.split_once(':') {
                if decision.dry_run {
                    self.event(
                        "command.dry_run",
                        json!({"ruleId":id,"event":event,"profileId":decision.profile_id}),
                    );
                } else if let Err(error) = self.queue_command(id, event, false) {
                    self.event(
                        "command.failed",
                        json!({"ruleId":id,"event":event,"error":error}),
                    );
                }
            }
        }
        let decisions = self.engine.tick(&self.config, &present, now);
        for d in decisions {
            self.event("rule.triggered",json!({"ruleId":d.rule_id,"profileId":d.profile_id,"simulation":false,"dryRun":d.dry_run}));
            for a in d.actions {
                if d.dry_run {
                    self.event("rule.dry_run", json!(a));
                    continue;
                }
                let result = self.write(&a.monitor_id, a.vcp_code, a.input_value, false, false);
                self.event(if result.is_ok(){"monitor.ddc_write_submitted"}else{"monitor.ddc_write_failed"},json!({"ruleId":d.rule_id,"monitorId":a.monitor_id,"inputValue":a.input_value,"result":result}));
            }
        }
    }
    pub fn write(
        &mut self,
        id: &str,
        vcp: u8,
        value: u8,
        confirmed: bool,
        force: bool,
    ) -> Result<Value> {
        let saved = self
            .config
            .monitors
            .iter()
            .find(|m| m.id == id)
            .ok_or_else(|| Error::new("not_found"))?
            .clone();
        if value == 0
            || !saved
                .inputs()
                .iter()
                .any(|i| i.value == value && i.vcp_code == vcp)
        {
            return Err(Error::new("advanced_confirmation"));
        }
        if force && !confirmed {
            return Err(Error::new("confirmation_required"));
        }
        let now = self.now();
        let delay = self.config.safety.global_monitor_cooldown_ms;
        if !force
            && self
                .last_write
                .get(&saved.fingerprint)
                .is_some_and(|last| now.saturating_sub(*last) < delay)
        {
            return Err(Error::new("cooldown"));
        }
        let wall = chrono::Utc::now().timestamp_millis();
        let mut persisted: HashMap<String, i64> = fs::read(self.store.root.join("rate-state.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        if !force
            && persisted
                .get(&saved.fingerprint)
                .is_some_and(|last| wall.saturating_sub(*last) < delay as i64)
        {
            return Err(Error::new("cooldown"));
        }
        let live = self.hardware.monitors()?;
        let monitor = resolve_monitor(&saved, &live)?;
        self.last_write.insert(saved.fingerprint.clone(), now);
        persisted.insert(saved.fingerprint.clone(), wall);
        self.store
            .atomic("rate-state.json", &serde_json::to_vec(&persisted)?)?;
        let result = self.monitors.call(
            monitor,
            "write",
            vcp,
            Some(value),
            self.config.safety.command_timeout_ms,
            15000,
        )?;
        Ok(
            json!({"monitorId":id,"vcpCode":vcp,"inputValue":value,"result":result,"message":crate::t("ddc.submitted")}),
        )
    }
    fn mutate<F>(&mut self, f: F) -> Result<Value>
    where
        F: FnOnce(&mut Config) -> Result<Value>,
    {
        let out = self.store.mutate(f)?;
        self.apply_config(self.store.load()?);
        Ok(out)
    }
    pub fn execute(&mut self, op: &str, v: Value) -> Result<Value> {
        self.drain_monitors();
        if op.starts_with("wizard.") {
            if let Some(session) = v["session"].as_str() {
                if self.capture.as_ref().is_none_or(|c| c.id != session) {
                    return Err(Error::new("not_found"));
                }
            }
        }
        let mut id = text(&v, "id", "");
        if ["profiles.", "rules.", "devices.", "monitors."]
            .iter()
            .any(|prefix| op.starts_with(prefix))
            && !id.is_empty()
        {
            let candidates: Vec<(String, String)> = if op.starts_with("profiles.") {
                self.config
                    .profiles
                    .iter()
                    .map(|p| (p.id.clone(), p.name.clone()))
                    .collect()
            } else if op.starts_with("rules.") {
                self.config
                    .profiles
                    .iter()
                    .flat_map(|p| &p.rules)
                    .map(|r| (r.id.clone(), r.name.clone()))
                    .collect()
            } else if op.starts_with("devices.") {
                self.config
                    .usb_devices
                    .iter()
                    .map(|d| (d.id.clone(), d.alias.clone()))
                    .collect()
            } else {
                self.config
                    .monitors
                    .iter()
                    .map(|m| (m.id.clone(), m.alias.clone()))
                    .collect()
            };
            if !candidates.iter().any(|(key, _)| key == &id) {
                let matches: Vec<_> = candidates.iter().filter(|(_, name)| name == &id).collect();
                if matches.len() > 1 {
                    return Err(Error::new("ambiguous_selection").detail(
                        matches
                            .iter()
                            .map(|(key, _)| key.as_str())
                            .collect::<Vec<_>>()
                            .join(", "),
                    ));
                }
                if let Some((key, _)) = matches.first() {
                    id = key.clone();
                }
            }
        }
        match op {
            "diagnostics.export" => {
                let doctor = self.execute("doctor", json!({}))?;
                let mut report = json!({"schemaVersion":1,"doctor":doctor,"config":self.config,"events":self.history.iter().rev().take(500).collect::<Vec<_>>()});
                fn redact(v: &mut Value) {
                    match v {
                        Value::Object(map) => {
                            for (key, value) in map.iter_mut() {
                                if [
                                    "serial",
                                    "path",
                                    "connector",
                                    "root",
                                    "serialFingerprint",
                                    "command",
                                    "onConnect",
                                    "onDisconnect",
                                    "stdout",
                                    "stderr",
                                    "workingDirectory",
                                ]
                                .contains(&key.as_str())
                                    && !value.is_null()
                                {
                                    *value = json!("<redacted>");
                                } else {
                                    redact(value)
                                }
                            }
                        }
                        Value::Array(items) => {
                            for item in items {
                                redact(item)
                            }
                        }
                        _ => {}
                    }
                }
                redact(&mut report);
                Ok(report)
            }
            "status" | "agent.status" => Ok(
                json!({"version":env!("CARGO_PKG_VERSION"),"apiRevision":2,"features":["live-input","reversible-input-test","wizard-pair"],"agent":if self.paused{"paused"}else if self.usb_ok&&self.last_inhibition.is_none(){"running"}else{"degraded"},"usbAvailable":self.usb_ok,"usbCount":self.usb.len(),"profiles":self.config.profiles.len(),"uptimeMs":self.now(),"guiIndependent":true,"setupHold":self.now()<self.setup_hold_until,"standalone":self.owner_pid.is_none()||self.config.app.agent_standalone}),
            ),
            "agent.pause" => {
                self.paused = true;
                self.reset_engines(self.config.safety.resume_grace_ms);
                Ok(json!({"paused":true}))
            }
            "agent.resume" => {
                self.paused = false;
                let usb = platform::usb()?;
                self.snapshot(usb, true);
                Ok(json!({"paused":false}))
            }
            "agent.stop" => {
                self.stop = true;
                Ok(json!({"stopping":true}))
            }
            "agent.owner" => {
                let pid = uint(&v, "pid", 0);
                self.owner_pid = (pid > 0).then_some(pid as u32);
                Ok(json!({"ownerPid":self.owner_pid}))
            }
            "wizard.hold" => {
                if flag(&v, "release") {
                    self.setup_hold_until = 0;
                } else {
                    let ms = uint(&v, "ms", 15000).clamp(1000, 60000);
                    self.setup_hold_until = self.now() + ms;
                }
                Ok(json!({"setupHold":self.setup_hold_until>self.now()}))
            }
            "doctor" | "config.doctor" => {
                let monitors = self.hardware.monitors();
                Ok(
                    json!({"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"version":env!("CARGO_PKG_VERSION"),"agent":self.execute("status",json!({}))?,"usb":{"canEnumerate":self.usb_ok,"canHotplug":self.usb_ok,"backend":"nusb","readsKeyboardInput":false},"ddc":monitors,"startup":{"agent":self.config.app.launch_agent_at_login,"gui":self.config.app.launch_gui_at_login,"osApproval":"not_verified"},"message":crate::t("status.hardware")}),
                )
            }
            "events.list" | "logs" => {
                let limit = uint(&v, "limit", 200).min(10000) as usize;
                let events: Vec<_> = self
                    .history
                    .iter()
                    .filter(|e| {
                        v["level"]
                            .as_str()
                            .is_none_or(|level| e["severity"] == level)
                    })
                    .filter(|e| {
                        v["profile"]
                            .as_str()
                            .is_none_or(|profile| e["result"]["profileId"] == profile)
                    })
                    .filter(|e| {
                        v["since"]
                            .as_str()
                            .is_none_or(|since| e["timestamp"].as_str().is_some_and(|t| t >= since))
                    })
                    .rev()
                    .take(limit)
                    .cloned()
                    .collect();
                Ok(json!(events))
            }
            "events.clear" => {
                if !flag(&v, "confirm") {
                    return Err(Error::new("confirmation_required"));
                }
                self.history.clear();
                self.store.atomic("events.jsonl", b"")?;
                Ok(json!({}))
            }
            "config.show" => Ok(json!(self.config)),
            "config.schema" => Ok(serde_json::to_value(schemars::schema_for!(Config))?),
            "config.validate" => {
                self.config.validate()?;
                Ok(json!({"valid":true}))
            }
            "config.get" => {
                let key = arg(&v, "id")?;
                let ptr = format!("/{}", key.replace('.', "/"));
                self.config_as_value()?
                    .pointer(&ptr)
                    .cloned()
                    .ok_or_else(|| Error::new("not_found"))
            }
            "config.set" => {
                let key = arg(&v, "id")?.to_string();
                let value = v["value"].clone();
                self.mutate(|c| {
                    let mut raw = serde_json::to_value(&*c)?;
                    let ptr = format!("/{}", key.replace('.', "/"));
                    let dest = raw
                        .pointer_mut(&ptr)
                        .ok_or_else(|| Error::new("invalid_input"))?;
                    *dest = value;
                    *c = serde_json::from_value(raw)?;
                    Ok(json!({"saved":true}))
                })
            }
            "config.replace" | "config.import" => {
                let c: Config = serde_json::from_value(v["config"].clone())?;
                c.validate()?;
                if flag(&v, "dry-run") {
                    return Ok(json!({"valid":true,"config":c}));
                }
                self.mutate(|old| {
                    *old = c;
                    Ok(json!({"saved":true}))
                })
            }
            "config.reset" => {
                if !flag(&v, "confirm") {
                    return Err(Error::new("confirmation_required"));
                }
                self.mutate(|c| {
                    *c = Config::default();
                    Ok(json!({"saved":true}))
                })
            }
            "ui.theme.get" => Ok(json!(self.config.app.theme)),
            "ui.mode.get" => Ok(json!(self.config.app.ui_mode)),
            "ui.theme.set" | "ui.mode.set" => {
                let value = id.clone();
                self.mutate(|c| {
                    if op == "ui.theme.set" {
                        c.app.theme = value
                    } else {
                        c.app.ui_mode = value
                    }
                    Ok(json!({"saved":true}))
                })
            }
            "startup.status" => Ok(
                json!({"agent":self.config.app.launch_agent_at_login,"gui":self.config.app.launch_gui_at_login,"delayMs":self.config.app.startup_delay_ms,"osApproval":"not_verified"}),
            ),
            "startup.enable" | "startup.disable" | "agent.enable" | "agent.disable" => {
                let component = text(&v, "component", "agent");
                if !["agent", "gui"].contains(&component.as_str()) {
                    return Err(Error::new("invalid_input"));
                }
                let out = platform::startup(&self.store, &component, op.ends_with("enable"))?;
                self.config = self.store.load()?;
                Ok(out)
            }
            "devices.list" | "devices.watch" => Ok(
                json!({"live":self.usb,"tracked":self.config.usb_devices,"available":self.usb_ok}),
            ),
            "devices.rescan" => {
                self.snapshot(platform::usb()?, false);
                self.execute("devices.list", json!({}))
            }
            "devices.show" => self
                .usb
                .iter()
                .find(|d| d.id == id)
                .map(|d| json!(d))
                .or_else(|| {
                    self.config
                        .usb_devices
                        .iter()
                        .find(|d| d.id == id)
                        .map(|d| json!(d))
                })
                .ok_or_else(|| Error::new("not_found")),
            "devices.track" => {
                let usb = self
                    .usb
                    .iter()
                    .find(|d| d.id == id)
                    .or_else(|| {
                        self.capture
                            .as_ref()
                            .and_then(|c| c.baseline.iter().find(|d| d.id == id))
                    })
                    .ok_or_else(|| Error::new("not_found"))?
                    .clone();
                let strategy = text(
                    &v,
                    "match",
                    if usb.serial.is_some() {
                        "serial"
                    } else {
                        "vid-pid"
                    },
                );
                let device = Device {
                    id: id.clone(),
                    alias: text(&v, "name", &usb.name),
                    enabled: true,
                    selectors: vec![Selector {
                        id: crate::model::id("selector"),
                        strategy,
                        vid: usb.vid,
                        pid: usb.pid,
                        serial: usb.serial,
                        path: Some(usb.path),
                        allow_ambiguous: flag(&v, "allow-ambiguous"),
                    }],
                };
                self.mutate(|c| {
                    if let Some(existing) = c.usb_devices.iter_mut().find(|d| d.id == device.id) {
                        if v["name"].is_string() {
                            existing.alias = device.alias.clone();
                        }
                    } else {
                        c.usb_devices.push(device.clone())
                    }
                    Ok(json!(device))
                })
            }
            "devices.alias" => {
                let name = arg(&v, "name")?.to_string();
                self.mutate(|c| {
                    let d = c
                        .usb_devices
                        .iter_mut()
                        .find(|d| d.id == id)
                        .ok_or_else(|| Error::new("not_found"))?;
                    d.alias = name;
                    Ok(json!(d))
                })
            }
            "devices.selectors.show" => self
                .config
                .usb_devices
                .iter()
                .find(|d| d.id == id)
                .map(|d| json!(d.selectors))
                .ok_or_else(|| Error::new("not_found")),
            "devices.selectors.add" => {
                let selector: Selector = if v["selector"].is_object() {
                    serde_json::from_value(v["selector"].clone())?
                } else {
                    let parse16 = |k: &str| -> Result<u16> {
                        let s = arg(&v, k)?;
                        u16::from_str_radix(s.trim_start_matches("0x"), 16)
                            .map_err(|_| Error::new("invalid_input"))
                    };
                    Selector {
                        id: crate::model::id("selector"),
                        strategy: arg(&v, "match")?.into(),
                        vid: parse16("vid")?,
                        pid: parse16("pid")?,
                        serial: v["serial"].as_str().map(str::to_owned),
                        path: v["path"].as_str().map(str::to_owned),
                        allow_ambiguous: flag(&v, "allow-ambiguous"),
                    }
                };
                self.mutate(|c| {
                    let d = c
                        .usb_devices
                        .iter_mut()
                        .find(|d| d.id == id)
                        .ok_or_else(|| Error::new("not_found"))?;
                    d.selectors.push(selector.clone());
                    Ok(json!(selector))
                })
            }
            "devices.selectors.remove" => {
                let selector = arg(&v, "selector")?.to_string();
                self.mutate(|c| {
                    let d = c
                        .usb_devices
                        .iter_mut()
                        .find(|d| d.id == id)
                        .ok_or_else(|| Error::new("not_found"))?;
                    d.selectors.retain(|s| s.id != selector);
                    Ok(json!(d))
                })
            }
            "monitors.list" | "monitors.detect" => match self.hardware.monitors() {
                Ok(live) => Ok(json!({"live":live,"saved":self.config.monitors,"available":true})),
                Err(e) => {
                    Ok(json!({"live":[],"saved":self.config.monitors,"available":false,"error":e}))
                }
            },
            "monitors.remember" => {
                let list = self.hardware.monitors()?;
                let m = list
                    .iter()
                    .find(|m| m.id == id)
                    .ok_or_else(|| Error::new("not_found"))?;
                let saved = SavedMonitor {
                    id: m.id.clone(),
                    alias: m.name.clone(),
                    fingerprint: m.fingerprint.clone(),
                    connector: Some(m.connector.clone()),
                    input_options: vec![],
                    hidden_inputs: vec![],
                };
                self.mutate(|c| {
                    if !c.monitors.iter().any(|s| s.id == id) {
                        c.monitors.push(saved.clone())
                    }
                    Ok(json!(saved))
                })
            }
            "monitors.show" => {
                let list = self.hardware.monitors()?;
                let saved = self
                    .config
                    .monitors
                    .iter()
                    .find(|m| m.id == id)
                    .ok_or_else(|| Error::new("not_found"))?;
                Ok(json!({"saved":saved,"live":resolve_monitor(saved,&list)?}))
            }
            "monitors.capabilities" | "monitors.input.get" => {
                let list = self.hardware.monitors()?;
                let saved = self
                    .config
                    .monitors
                    .iter()
                    .find(|m| m.id == id)
                    .ok_or_else(|| Error::new("not_found"))?;
                let monitor = resolve_monitor(saved, &list)?;
                self.monitors.call(
                    monitor,
                    if op == "monitors.capabilities" {
                        "capabilities"
                    } else {
                        "read"
                    },
                    bytes(&v, "vcp", 96)?,
                    None,
                    self.config.safety.command_timeout_ms,
                    15000,
                )
            }
            "monitors.state" => {
                if let Some(state) = self.monitors.cached(
                    &id,
                    flag(&v, "watch"),
                    self.config.safety.command_timeout_ms,
                ) {
                    return Ok(state);
                }
                let saved = self.ensure_monitor(&id)?;
                let live = self.hardware.monitors()?;
                let monitor = resolve_monitor(&saved, &live)?;
                Ok(self.monitors.state(
                    monitor,
                    flag(&v, "watch"),
                    self.config.safety.command_timeout_ms,
                ))
            }
            "monitors.input.restore" => {
                let saved = self.ensure_monitor(&id)?;
                let live = self.hardware.monitors()?;
                let monitor = resolve_monitor(&saved, &live)?;
                self.monitors.call(
                    monitor,
                    "restore",
                    96,
                    None,
                    self.config.safety.command_timeout_ms,
                    15000,
                )
            }
            "monitors.inputs" => {
                let saved = self
                    .config
                    .monitors
                    .iter()
                    .find(|m| m.id == id)
                    .ok_or_else(|| Error::new("not_found"))?;
                Ok(
                    json!({"saved":saved.input_options,"inputs":saved.inputs(),"reference":[{"value":1,"label":"VGA 1"},{"value":3,"label":"DVI 1"},{"value":15,"label":"DisplayPort 1"},{"value":16,"label":"DisplayPort 2"},{"value":17,"label":"HDMI 1"},{"value":18,"label":"HDMI 2"}],"referenceConfidence":"unverified"}),
                )
            }
            "monitors.input.set" | "monitors.input.test" => {
                if !flag(&v, "confirm") {
                    return Err(Error::new("confirmation_required"));
                }
                let value = bytes(
                    &v,
                    if v["value"].is_null() {
                        "input"
                    } else {
                        "value"
                    },
                    0,
                )?;
                let vcp = bytes(&v, "vcp", 96)?;
                if op == "monitors.input.test" {
                    let saved = self.ensure_monitor(&id)?;
                    if !saved
                        .inputs()
                        .iter()
                        .any(|i| i.value == value && i.vcp_code == vcp)
                    {
                        return Err(Error::new("invalid_input"));
                    }
                    let live = self.hardware.monitors()?;
                    let monitor = resolve_monitor(&saved, &live)?;
                    let duration = uint(&v, "restore-ms", 15000);
                    if !(100..=60000).contains(&duration) {
                        return Err(Error::new("invalid_input")
                            .field("restore-ms")
                            .detail("100..60000 ms"));
                    }
                    return self.monitors.call(
                        monitor,
                        "test",
                        vcp,
                        Some(value),
                        self.config.safety.command_timeout_ms,
                        duration,
                    );
                }
                let out = self.write(&id, vcp, value, true, flag(&v, "force"));
                self.event(
                    if out.is_ok() {
                        "monitor.ddc_write_submitted"
                    } else {
                        "monitor.ddc_write_failed"
                    },
                    json!({"monitorId":id,"inputValue":value,"result":out}),
                );
                out
            }
            "monitors.input.alias.set" => {
                let value = bytes(&v, "value", 0)?;
                let vcp = bytes(&v, "vcp", 96)?;
                if vcp != 96 && !flag(&v, "confirm") {
                    return Err(Error::new("advanced_confirmation"));
                }
                self.ensure_monitor(&id)?;
                let old = self
                    .config
                    .monitors
                    .iter()
                    .find(|m| m.id == id)
                    .and_then(|m| {
                        m.inputs()
                            .into_iter()
                            .find(|i| i.value == value && i.vcp_code == vcp)
                    });
                let input = Input {
                    value,
                    vcp_code: vcp,
                    label: text(
                        &v,
                        "label",
                        &old.as_ref()
                            .map(|i| i.label.clone())
                            .unwrap_or_else(|| format!("Input {value}")),
                    ),
                    verified: if v.get("verified").is_some() {
                        flag(&v, "verified")
                    } else {
                        old.as_ref().is_some_and(|i| i.verified)
                    },
                };
                self.mutate(|c| {
                    let m = c
                        .monitors
                        .iter_mut()
                        .find(|m| m.id == id)
                        .ok_or_else(|| Error::new("not_found"))?;
                    m.input_options
                        .retain(|i| !(i.value == value && i.vcp_code == vcp));
                    m.hidden_inputs.retain(|v| *v != value);
                    m.input_options.push(input.clone());
                    Ok(json!(input))
                })
            }
            "monitors.input.alias.remove" => {
                let value = bytes(&v, "value", 0)?;
                self.ensure_monitor(&id)?;
                if flag(&v, "reset") {
                    let mut input = standard_inputs()
                        .into_iter()
                        .find(|i| i.value == value)
                        .ok_or_else(|| Error::new("invalid_input").field("reset"))?;
                    input.verified = self
                        .config
                        .monitors
                        .iter()
                        .find(|m| m.id == id)
                        .and_then(|m| m.inputs().into_iter().find(|i| i.value == value))
                        .is_some_and(|i| i.verified);
                    return self.mutate(|c| {
                        let m = c
                            .monitors
                            .iter_mut()
                            .find(|m| m.id == id)
                            .ok_or_else(|| Error::new("not_found"))?;
                        m.input_options.retain(|i| i.value != value);
                        m.input_options.push(input.clone());
                        m.hidden_inputs.retain(|v| *v != value);
                        Ok(json!({"reset":true,"input":input}))
                    });
                }

                if self
                    .config
                    .profiles
                    .iter()
                    .flat_map(|p| &p.rules)
                    .flat_map(|r| &r.actions)
                    .any(|a| a.monitor_id == id && a.input_value == value)
                {
                    return Err(Error::new("input_in_use"));
                }
                self.mutate(|c| {
                    let m = c
                        .monitors
                        .iter_mut()
                        .find(|m| m.id == id)
                        .ok_or_else(|| Error::new("not_found"))?;
                    m.input_options.retain(|i| i.value != value);
                    if !m.hidden_inputs.contains(&value) {
                        m.hidden_inputs.push(value);
                    }
                    Ok(json!({"removed":true}))
                })
            }
            "profiles.list" => Ok(json!(self.config.profiles)),
            "profiles.create" => {
                let p = Profile {
                    id: crate::model::id("profile"),
                    name: unique_profile_name(&self.config.profiles, arg(&v, "name")?, None),
                    enabled: self.config.profiles.iter().all(|p| !p.enabled),
                    rules: vec![],
                };
                self.mutate(|c| {
                    c.profiles.push(p.clone());
                    Ok(json!(p))
                })
            }
            "profiles.show" | "profiles.export" => self
                .config
                .profiles
                .iter()
                .find(|p| p.id == id)
                .map(|p| json!(p))
                .ok_or_else(|| Error::new("not_found")),
            "profiles.rename" | "profiles.enable" | "profiles.activate" | "profiles.disable" => {
                self.mutate(|c| {
                    if op == "profiles.enable" || op == "profiles.activate" {
                        for p in &mut c.profiles {
                            p.enabled = p.id == id;
                        }
                    }
                    let renamed = if op == "profiles.rename" {
                        Some(unique_profile_name(&c.profiles, arg(&v, "name")?, Some(&id)))
                    } else {
                        None
                    };
                    let p = find_profile(c, &id)?;
                    if let Some(name) = renamed {
                        p.name = name;
                        return Ok(json!(p));
                    }
                    match op {
                        "profiles.rename" => p.name = arg(&v, "name")?.into(),
                        "profiles.enable" | "profiles.activate" => p.enabled = true,
                        _ => p.enabled = false,
                    }
                    Ok(json!(p))
                })
            }
            "profiles.clone" => self.mutate(|c| {
                let mut p = find_profile(c, &id)?.clone();
                p.id = crate::model::id("profile");
                let desired = v["name"].as_str().unwrap_or(&p.name).to_owned();
                p.name = unique_profile_name(&c.profiles, &desired, None);
                p.enabled = false;
                for r in &mut p.rules {
                    r.id = crate::model::id("rule");
                    for a in &mut r.actions {
                        a.id = crate::model::id("action")
                    }
                }
                c.profiles.push(p.clone());
                Ok(json!(p))
            }),
            "profiles.delete" => {
                if !flag(&v, "confirm") {
                    return Err(Error::new("confirmation_required"));
                }
                self.mutate(|c| {
                    find_profile(c, &id)?;
                    c.profiles.retain(|p| p.id != id);
                    Ok(json!({}))
                })
            }
            "profiles.import" => {
                let mut p: Profile = serde_json::from_value(v["profileData"].clone())?;
                p.id = crate::model::id("profile");
                for r in &mut p.rules {
                    r.id = crate::model::id("rule");
                    for a in &mut r.actions {
                        a.id = crate::model::id("action");
                    }
                }
                if flag(&v, "dry-run") {
                    let mut c = self.config.clone();
                    c.profiles.push(p.clone());
                    c.validate()?;
                    return Ok(json!({"profile":p,"crossHostIdentityReviewRequired":true}));
                }
                self.mutate(|c| {
                    c.profiles.push(p.clone());
                    Ok(json!(p))
                })
            }
            "rules.list" => Ok(json!(self
                .config
                .profiles
                .iter()
                .filter(|p| v["profile"].as_str().is_none_or(|id| p.id == id))
                .flat_map(|p| &p.rules)
                .collect::<Vec<_>>())),
            "rules.create" => {
                let devices = split_devices(&v)?;
                let monitor = arg(&v, "monitor")?.to_string();
                let r = Rule {
                    id: crate::model::id("rule"),
                    name: text(&v, "name", "KVM"),
                    enabled: true,
                    trigger: text(&v, "on", "disconnect"),
                    minimum_count: uint(&v, "min-count", 1) as usize,
                    device_ids: devices,
                    mode: text(&v, "mode", "all"),
                    coincidence_window_ms: match uint(&v, "coincidence-window-ms", 0) {
                        0 => None,
                        n => Some(n),
                    },
                    debounce_ms: uint(&v, "debounce-ms", self.config.safety.default_debounce_ms),
                    rearm_ms: uint(&v, "rearm-ms", 500),
                    cooldown_ms: uint(&v, "cooldown-ms", 500),
                    dry_run: flag(&v, "dry-run"),
                    commands: v
                        .get("commands")
                        .map(|v| serde_json::from_value(v.clone()))
                        .transpose()?
                        .unwrap_or_default(),
                    actions: vec![Action {
                        id: crate::model::id("action"),
                        monitor_id: monitor,
                        vcp_code: bytes(&v, "vcp", 96)?,
                        input_value: bytes(&v, "input", 0)?,
                    }],
                };
                let profile = arg(&v, "profile")?.to_string();
                self.mutate(|c| {
                    find_profile(c, &profile)?.rules.push(r.clone());
                    Ok(json!(r))
                })
            }
            "rules.commands.get" => Ok(json!(find_rule(&mut self.config, &id)?.commands)),
            "rules.commands.set" | "rules.commands.enable" | "rules.commands.disable" => self
                .mutate(|c| {
                    let r = find_rule(c, &id)?;
                    if let Some(commands) = v.get("commands") {
                        r.commands = serde_json::from_value(commands.clone())?;
                    }
                    for (option, field) in [
                        ("on-connect", &mut r.commands.on_connect),
                        ("on-disconnect", &mut r.commands.on_disconnect),
                        ("shell", &mut r.commands.shell),
                        ("working-dir", &mut r.commands.working_directory),
                    ] {
                        if let Some(value) = v[option].as_str() {
                            *field = value.into();
                        }
                    }
                    if !v["timeout-ms"].is_null() {
                        r.commands.timeout_ms = uint(&v, "timeout-ms", 0);
                    }
                    if op == "rules.commands.enable" {
                        r.commands.enabled = true;
                    }
                    if op == "rules.commands.disable" {
                        r.commands.enabled = false;
                    }
                    r.commands.validate()?;
                    Ok(json!(r.commands))
                }),
            "rules.commands.run" => {
                if !flag(&v, "confirm") {
                    return Err(Error::new("confirmation_required"));
                }
                self.queue_command(&id, arg(&v, "event")?, true)
            }
            "commands.list" => {
                self.drain_commands();
                Ok(json!(self
                    .history
                    .iter()
                    .rev()
                    .filter(|e| e["eventType"]
                        .as_str()
                        .is_some_and(|s| s.starts_with("command.")))
                    .take(uint(&v, "limit", 100).min(1000) as usize)
                    .collect::<Vec<_>>()))
            }
            "commands.show" => {
                self.drain_commands();
                self.history
                    .iter()
                    .rev()
                    .find(|e| e["result"]["jobId"].as_str() == Some(&id))
                    .cloned()
                    .ok_or_else(|| Error::new("not_found"))
            }
            "rules.show" => Ok(json!(find_rule(&mut self.config, &id)?)),
            "rules.delete" => {
                if !flag(&v, "confirm") {
                    return Err(Error::new("confirmation_required"));
                }
                self.mutate(|c| {
                    find_rule(c, &id)?;
                    for p in &mut c.profiles {
                        p.rules.retain(|r| r.id != id)
                    }
                    Ok(json!({}))
                })
            }
            "rules.set"
            | "rules.timing"
            | "rules.enable"
            | "rules.disable"
            | "rules.dry-run"
            | "rules.add-device"
            | "rules.remove-device"
            | "rules.add-action"
            | "rules.remove-action" => self.mutate(|c| {
                let r = find_rule(c, &id)?;
                if let Some(s) = v["name"].as_str() {
                    r.name = s.into()
                }
                if let Some(s) = v["on"].as_str() {
                    r.trigger = s.into()
                }
                if let Some(s) = v["mode"].as_str() {
                    r.mode = s.into()
                }
                if !v["devices"].is_null() {
                    r.device_ids = split_devices(&v)?
                }
                if !v["min-count"].is_null() {
                    r.minimum_count = uint(&v, "min-count", 1) as usize
                }
                for key in [
                    "debounce-ms",
                    "rearm-ms",
                    "cooldown-ms",
                    "coincidence-window-ms",
                ] {
                    if !v[key].is_null() {
                        let value = uint(&v, key, 0);
                        match key {
                            "debounce-ms" => r.debounce_ms = value,
                            "rearm-ms" => r.rearm_ms = value,
                            "cooldown-ms" => r.cooldown_ms = value,
                            _ => {
                                r.coincidence_window_ms =
                                    if value == 0 { None } else { Some(value) }
                            }
                        }
                    }
                }
                if op == "rules.enable" {
                    r.enabled = true
                }
                if op == "rules.disable" {
                    r.enabled = false
                }
                if op == "rules.dry-run" {
                    r.dry_run = arg(&v, "value")? == "on"
                }
                if op == "rules.add-device" {
                    r.device_ids.push(arg(&v, "value")?.into())
                }
                if op == "rules.remove-device" {
                    r.device_ids.retain(|d| v["value"].as_str() != Some(d))
                }
                if op == "rules.add-action" {
                    r.actions.push(Action {
                        id: crate::model::id("action"),
                        monitor_id: arg(&v, "monitor")?.into(),
                        vcp_code: bytes(&v, "vcp", 96)?,
                        input_value: bytes(&v, "input", 0)?,
                    })
                }
                if op == "rules.remove-action" {
                    r.actions.retain(|a| v["value"].as_str() != Some(&a.id))
                }
                if op == "rules.set" {
                    if let Some(value) = v.get("commands") {
                        r.commands = serde_json::from_value(value.clone())?;
                    }
                    if let Some(value) = v["dry-run"].as_bool() {
                        r.dry_run = value;
                    }
                    if let Some(a) = r.actions.first_mut() {
                        if let Some(m) = v["monitor"].as_str() {
                            a.monitor_id = m.into()
                        }
                        if !v["input"].is_null() {
                            a.input_value = bytes(&v, "input", 0)?
                        }
                    }
                }
                Ok(json!(r))
            }),
            "rules.simulate" => {
                let r = find_rule(&mut self.config, &id)?.clone();
                let mut config = self.config.clone();
                config
                    .profiles
                    .retain(|p| p.rules.iter().any(|x| x.id == id));
                for p in &mut config.profiles {
                    p.rules.retain(|x| x.id == id)
                }
                if v["event"].as_str().is_some_and(|event| event != r.trigger) {
                    return Ok(json!({"simulation":true,"hardwareWrites":0,"decisions":[]}));
                }
                let mut engine = Engine::default();
                let opposite = r.trigger != "connect";
                let before = r.device_ids.iter().map(|d| (d.clone(), opposite)).collect();
                engine.tick(&config, &before, 0);
                engine.tick(&config, &before, r.rearm_ms);
                let after = r
                    .device_ids
                    .iter()
                    .map(|d| (d.clone(), !opposite))
                    .collect();
                let t = r.rearm_ms + 1;
                engine.tick(&config, &after, t);
                let decisions = engine.tick(&config, &after, t + r.debounce_ms);
                Ok(
                    json!({"simulation":true,"hardwareWrites":0,"decisions":decisions.iter().map(|d|json!({"ruleId":d.rule_id,"actions":d.actions})).collect::<Vec<_>>()}),
                )
            }
            "wizard.start" | "wizard.capture.start" => {
                if !self.usb_ok {
                    return Err(Error::new("usb_unavailable"));
                }
                let c = Capture {
                    started_mono_ms: self.now(),
                    observations: vec![],
                    id: crate::model::id("capture"),
                    cycles: uint(&v, "cycles", 2).clamp(1, 20) as usize,
                    active: true,
                    started: chrono::Utc::now().to_rfc3339(),
                    baseline: self.usb.clone(),
                    counts: HashMap::new(),
                    away: HashSet::new(),
                    events: vec![],
                };
                self.store
                    .atomic("capture.json", &serde_json::to_vec(&c)?)?;
                self.capture = Some(c.clone());
                self.setup_hold_until = self.setup_hold_until.max(self.now() + 60000);
                Ok(json!({"session":c,"instruction":crate::t("wizard.switch")}))
            }
            "wizard.capture.status" | "wizard.candidates" => {
                let c = self
                    .capture
                    .as_ref()
                    .ok_or_else(|| Error::new("not_found"))?;
                Ok(
                    json!({"session":c,"observations":c.observations,"candidates":c.baseline.iter().map(|d|json!({"device":d,"matchedCycles":c.counts.get(&d.id).unwrap_or(&0),"reliable":c.counts.get(&d.id).copied().unwrap_or(0)>=c.cycles,"stillAbsent":c.away.contains(&d.id)})).collect::<Vec<_>>()}),
                )
            }
            "wizard.capture.finish" | "wizard.cancel" => {
                if op == "wizard.cancel" {
                    self.setup_hold_until = 0;
                }
                let c = self
                    .capture
                    .as_mut()
                    .ok_or_else(|| Error::new("not_found"))?;
                c.active = false;
                self.store
                    .atomic("capture.json", &serde_json::to_vec(&c)?)?;
                self.execute("wizard.candidates", json!({}))
            }
            "wizard.complete" => {
                // Manual setup completes without a capture: devices then come
                // from the live USB inventory. Each direction may watch its own
                // device set; `devices` remains the shared fallback.
                let capture = self.capture.clone();
                if capture.is_none() && !v["session"].is_null() {
                    return Err(Error::new("capture_incomplete"));
                }
                let shared = if v["devices"].is_null() {
                    vec![]
                } else {
                    split_devices(&v)?
                };
                let mut per_event = HashMap::new();
                for event in ["connect", "disconnect"] {
                    let key = format!("{event}-devices");
                    let ids = if v[key.as_str()].is_null() {
                        shared.clone()
                    } else {
                        device_list(&v, &key)?
                    };
                    if ids.is_empty() {
                        return Err(Error::new("invalid_input").field(if v[key.as_str()].is_null() {
                            "devices".to_owned()
                        } else {
                            key
                        }));
                    }
                    per_event.insert(event, ids);
                }
                let mut selected: Vec<String> = vec![];
                for id in per_event["disconnect"].iter().chain(per_event["connect"].iter()) {
                    if !selected.contains(id) {
                        selected.push(id.clone());
                    }
                }
                let mut tracked = vec![];
                for id in &selected {
                    let device = capture
                        .iter()
                        .flat_map(|c| c.observations.iter().map(|o| &o.device).chain(c.baseline.iter()))
                        .chain(self.usb.iter())
                        .find(|d| &d.id == id)
                        .ok_or_else(|| Error::new("not_found"))?;
                    tracked.push(Device {
                        id: id.clone(),
                        alias: device.name.clone(),
                        enabled: true,
                        selectors: vec![Selector {
                            id: crate::model::id("selector"),
                            strategy: if device.serial.is_some() {
                                "serial"
                            } else {
                                "vid-pid"
                            }
                            .into(),
                            vid: device.vid,
                            pid: device.pid,
                            serial: device.serial.clone(),
                            path: Some(device.path.clone()),
                            allow_ambiguous: false,
                        }],
                    });
                }
                let live = self.hardware.monitors()?;
                let mut monitors = vec![];
                let mut rules = vec![];
                for event in ["connect", "disconnect"] {
                    let mid = arg(&v, &format!("{event}-monitor"))?;
                    let input = bytes(&v, &format!("{event}-input"), 0)?;
                    let monitor = live
                        .iter()
                        .find(|m| m.id == mid)
                        .ok_or_else(|| Error::new("monitor_missing"))?;
                    let saved = self
                        .config
                        .monitors
                        .iter()
                        .find(|m| m.id == mid)
                        .cloned()
                        .unwrap_or(SavedMonitor {
                            id: mid.into(),
                            alias: monitor.name.clone(),
                            fingerprint: monitor.fingerprint.clone(),
                            connector: Some(monitor.connector.clone()),
                            input_options: vec![],
                            hidden_inputs: vec![],
                        });
                    if !saved
                        .inputs()
                        .iter()
                        .any(|i| i.value == input && i.vcp_code == 96)
                    {
                        return Err(Error::new("invalid_input").field(format!("{event}-input")));
                    }
                    monitors.push(saved);
                    rules.push(Rule {
                        id: crate::model::id("rule"),
                        name: crate::t(&format!("wizard.rule.{event}")),
                        enabled: true,
                        trigger: event.into(),
                        device_ids: per_event[event].clone(),
                        mode: text(&v, "mode", "all"),
                        minimum_count: uint(&v, "min-count", 1) as usize,
                        coincidence_window_ms: None,
                        debounce_ms: 0,
                        rearm_ms: 500,
                        cooldown_ms: 500,
                        dry_run: flag(&v, "dry-run"),
                        actions: vec![Action {
                            id: crate::model::id("action"),
                            monitor_id: mid.into(),
                            vcp_code: 96,
                            input_value: input,
                        }],
                        commands: CommandHooks::default(),
                    });
                }
                let profile = Profile {
                    id: crate::model::id("profile"),
                    name: unique_profile_name(
                        &self.config.profiles,
                        &text(&v, "name", &crate::t("wizard.default_profile")),
                        None,
                    ),
                    enabled: true,
                    rules,
                };
                let snapshot = self.usb.clone();
                let result = self.mutate(|c| {
                    for device in tracked {
                        if !c.usb_devices.iter().any(|d| d.id == device.id) {
                            c.usb_devices.push(device);
                        }
                    }
                    for monitor in monitors {
                        if !c.monitors.iter().any(|m| m.id == monitor.id) {
                            c.monitors.push(monitor);
                        }
                    }
                    for p in &mut c.profiles {
                        p.enabled = false;
                    }
                    c.profiles.push(profile.clone());
                    presence(c, &snapshot)?;
                    Ok(json!(profile))
                })?;
                let baseline = presence(&self.config, &self.usb)?;
                self.engine.prime(&self.config, &baseline, self.now());
                self.command_engine
                    .prime(&self.command_config, &baseline, self.now());
                if let Some(c) = &mut self.capture {
                    c.active = false;
                    self.store.atomic("capture.json", &serde_json::to_vec(c)?)?;
                }
                // The new profile was just primed from the live baseline.
                self.setup_hold_until = 0;
                self.setup_held = false;
                Ok(result)
            }
            "wizard.apply" => {
                let c = self
                    .capture
                    .as_ref()
                    .ok_or_else(|| Error::new("not_found"))?
                    .clone();
                let devices = split_devices(&v)?;
                for id in &devices {
                    let d = c
                        .baseline
                        .iter()
                        .find(|d| d.id == *id)
                        .ok_or_else(|| Error::new("not_found"))?
                        .clone();
                    if !self.config.usb_devices.iter().any(|d| d.id == *id) {
                        let device = Device {
                            id: id.clone(),
                            alias: d.name.clone(),
                            enabled: true,
                            selectors: vec![Selector {
                                id: crate::model::id("selector"),
                                strategy: if d.serial.is_some() {
                                    "serial"
                                } else {
                                    "vid-pid"
                                }
                                .into(),
                                vid: d.vid,
                                pid: d.pid,
                                serial: d.serial,
                                path: Some(d.path),
                                allow_ambiguous: false,
                            }],
                        };
                        self.mutate(|cfg| {
                            cfg.usb_devices.push(device);
                            Ok(json!({}))
                        })?;
                    }
                }
                let mut v = v.clone();
                if v["profile"].is_null() {
                    let p =
                        self.execute("profiles.create", json!({"name":text(&v,"name","KVM")}))?;
                    v["profile"] = p["id"].clone()
                }
                self.execute("rules.create", v)
            }
            _ => Err(Error::new("unsupported").detail(op)),
        }
    }
    fn config_as_value(&self) -> Result<Value> {
        Ok(serde_json::to_value(&self.config)?)
    }
}
pub fn run(store: Store) -> Result<()> {
    let _lock = store.instance_lock()?;
    let mut service = Service::new(store.clone())?;
    let (tx, rx) = std::sync::mpsc::sync_channel(64);
    crate::ipc::listen(store.clone(), tx)?;
    let (usb_tx, usb_rx) = std::sync::mpsc::sync_channel(64);
    let epoch = service.epoch;
    std::thread::spawn(move || {
        use futures_lite::StreamExt;
        let mut backoff = 1;
        loop {
            match nusb::watch_devices() {
                Ok(mut watcher) => {
                    let _ = usb_tx.send(
                        platform::usb().map(|u| (u, true, epoch.elapsed().as_millis() as u64)),
                    );
                    backoff = 1;
                    futures_lite::future::block_on(async {
                        while watcher.next().await.is_some() {
                            if usb_tx
                                .send(
                                    platform::usb()
                                        .map(|u| (u, false, epoch.elapsed().as_millis() as u64)),
                                )
                                .is_err()
                            {
                                return;
                            }
                        }
                    });
                }
                Err(e) => {
                    let _ = usb_tx.send(Err(Error::new("usb_unavailable").detail(e)));
                }
            }
            std::thread::sleep(Duration::from_secs(backoff));
            backoff = (backoff * 2).min(30);
        }
    });
    service.event("agent.started", json!({"pid":std::process::id()}));
    let mut last_tick = std::time::SystemTime::now();
    let mut last_reload = Instant::now();
    let mut modified = fs::metadata(store.path()).and_then(|m| m.modified()).ok();
    while !service.stop {
        if let Ok((req, reply)) = rx.recv_timeout(Duration::from_millis(50)) {
            let result = service.execute(&req.command, req.payload);
            let _ = reply.send(crate::ipc::Response::from_result(req.id, result));
        }
        let wall = std::time::SystemTime::now();
        if wall.duration_since(last_tick).unwrap_or_default() > Duration::from_secs(3) {
            service.reset_engines(service.config.safety.resume_grace_ms);
            if let Ok(usb) = platform::usb() {
                service.snapshot(usb, true)
            } else {
                service.usb_ok = false
            }
        }
        last_tick = wall;
        while let Ok(result) = usb_rx.try_recv() {
            match result {
                Ok((usb, resync, observed_ms)) => service.snapshot_timed(usb, resync, observed_ms),
                Err(e) => {
                    service.usb_ok = false;
                    service.reset_engines(service.config.safety.resume_grace_ms);
                    service.event("agent.degraded", json!(e));
                }
            }
        }
        if last_reload.elapsed() > Duration::from_secs(1) {
            last_reload = Instant::now();
            if let Some(pid) = service.owner_pid {
                if !service.config.app.agent_standalone && !platform::process_alive(pid) {
                    service.event("agent.owner_exited", json!({"pid":pid}));
                    break;
                }
            }
            let next = fs::metadata(store.path()).and_then(|m| m.modified()).ok();
            if next != modified {
                modified = next;
                match store.load() {
                    Ok(c) => {
                        service.apply_config(c);
                    }
                    Err(e) => service.event("config.failed", json!(e)),
                }
            }
        }
        service.tick();
    }
    service.cancel_commands();
    service.event("agent.stopped", json!({}));
    std::thread::sleep(Duration::from_millis(100));
    Ok(())
}
