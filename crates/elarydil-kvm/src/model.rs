use crate::{Error, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub fn id(prefix: &str) -> String {
    format!("{prefix}_{}", uuid::Uuid::new_v4().simple())
}
pub fn hash(s: &[u8]) -> String {
    use sha2::Digest;
    hex::encode(sha2::Sha256::digest(s))[..24].to_string()
}
pub fn number(s: &str) -> Result<u8> {
    let v = if let Some(x) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u8::from_str_radix(x, 16)
    } else {
        s.parse()
    };
    v.map_err(|_| Error::new("invalid_input"))
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub schema_version: u32,
    pub app: App,
    pub safety: Safety,
    pub usb_devices: Vec<Device>,
    pub monitors: Vec<SavedMonitor>,
    pub profiles: Vec<Profile>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: 1,
            app: App::default(),
            safety: Safety::default(),
            usb_devices: vec![],
            monitors: vec![],
            profiles: vec![],
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct App {
    pub locale: String,
    pub theme: String,
    pub ui_mode: String,
    pub launch_agent_at_login: bool,
    pub launch_gui_at_login: bool,
    /// Keep the agent running after the desktop app quits. Off by default:
    /// automation normally lives and dies with the app in the menu bar.
    pub agent_standalone: bool,
    pub close_window_action: String,
    pub notifications: String,
    pub startup_delay_ms: u64,
    pub last_page: String,
    pub log_retention_days: u32,
    pub log_max_bytes: u64,
}
impl Default for App {
    fn default() -> Self {
        Self {
            locale: "en-US".into(),
            theme: "system".into(),
            ui_mode: "both".into(),
            launch_agent_at_login: false,
            launch_gui_at_login: false,
            agent_standalone: false,
            close_window_action: "hide".into(),
            notifications: "errors-only".into(),
            startup_delay_ms: 0,
            last_page: "dashboard".into(),
            log_retention_days: 14,
            log_max_bytes: 20 * 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct Safety {
    pub global_monitor_cooldown_ms: u64,
    pub startup_grace_ms: u64,
    pub resume_grace_ms: u64,
    pub default_debounce_ms: u64,
    pub command_timeout_ms: u64,
}
impl Default for Safety {
    fn default() -> Self {
        Self {
            global_monitor_cooldown_ms: 100,
            startup_grace_ms: 3000,
            resume_grace_ms: 4000,
            default_debounce_ms: 0,
            command_timeout_ms: 2000,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Selector {
    pub id: String,
    pub strategy: String,
    pub vid: u16,
    pub pid: u16,
    pub serial: Option<String>,
    pub path: Option<String>,
    #[serde(default)]
    pub allow_ambiguous: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Device {
    pub id: String,
    pub alias: String,
    pub enabled: bool,
    pub selectors: Vec<Selector>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbDevice {
    pub id: String,
    pub physical_id: String,
    pub name: String,
    #[serde(default)]
    pub model: Option<String>,
    pub manufacturer: String,
    pub vid: u16,
    pub pid: u16,
    pub serial: Option<String>,
    pub path: String,
    pub parent: Option<String>,
    pub category: String,
    pub connected: bool,
    pub last_seen: String,
}
impl Selector {
    pub fn matches(&self, d: &UsbDevice) -> bool {
        self.vid == d.vid
            && self.pid == d.pid
            && match self.strategy.as_str() {
                "vid-pid" => true,
                "serial" => self.serial.is_some() && self.serial == d.serial,
                "instance" | "topology" => self.path.as_ref() == Some(&d.path),
                _ => false,
            }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Input {
    pub value: u8,
    pub vcp_code: u8,
    pub label: String,
    pub verified: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedMonitor {
    pub id: String,
    pub alias: String,
    pub fingerprint: String,
    pub connector: Option<String>,
    pub input_options: Vec<Input>,
    #[serde(default)]
    pub hidden_inputs: Vec<u8>,
}
pub fn standard_inputs() -> Vec<Input> {
    [
        (15, "DisplayPort 1"),
        (16, "DisplayPort 2"),
        (17, "HDMI 1"),
        (18, "HDMI 2"),
    ]
    .into_iter()
    .map(|(value, label)| Input {
        value,
        vcp_code: 96,
        label: label.into(),
        verified: false,
    })
    .collect()
}
impl SavedMonitor {
    pub fn inputs(&self) -> Vec<Input> {
        let mut inputs = standard_inputs()
            .into_iter()
            .filter(|i| !self.hidden_inputs.contains(&i.value))
            .collect::<Vec<_>>();
        for i in &self.input_options {
            if let Some(old) = inputs
                .iter_mut()
                .find(|old| old.value == i.value && old.vcp_code == i.vcp_code)
            {
                *old = i.clone();
            } else {
                inputs.push(i.clone());
            }
        }
        inputs
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    pub id: String,
    pub name: String,
    pub fingerprint: String,
    pub connector: String,
    pub backend: String,
    pub path: String,
    pub edid_available: bool,
    pub permissions: String,
    pub limitations: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub rules: Vec<Rule>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub trigger: String,
    pub device_ids: Vec<String>,
    pub mode: String,
    pub minimum_count: usize,
    pub coincidence_window_ms: Option<u64>,
    pub debounce_ms: u64,
    pub rearm_ms: u64,
    pub cooldown_ms: u64,
    pub dry_run: bool,
    pub actions: Vec<Action>,
    #[serde(default)]
    pub commands: CommandHooks,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct CommandHooks {
    pub enabled: bool,
    pub on_connect: String,
    pub on_disconnect: String,
    pub shell: String,
    pub working_directory: String,
    pub timeout_ms: u64,
}
impl Default for CommandHooks {
    fn default() -> Self {
        Self {
            enabled: false,
            on_connect: String::new(),
            on_disconnect: String::new(),
            shell: "system".into(),
            working_directory: String::new(),
            timeout_ms: 10000,
        }
    }
}
impl CommandHooks {
    pub fn validate(&self) -> Result<()> {
        if !["system", "bash", "powershell", "cmd"].contains(&self.shell.as_str())
            || !(100..=300000).contains(&self.timeout_ms)
            || self.on_connect.len() > 16384
            || self.on_disconnect.len() > 16384
            || self.working_directory.len() > 4096
            || [
                &self.on_connect,
                &self.on_disconnect,
                &self.working_directory,
            ]
            .iter()
            .any(|s| s.contains('\0'))
            || (self.enabled
                && self.on_connect.trim().is_empty()
                && self.on_disconnect.trim().is_empty())
        {
            return Err(Error::new("invalid_command_config"));
        }
        Ok(())
    }
    pub fn command(&self, event: &str) -> &str {
        if event == "connect" {
            &self.on_connect
        } else {
            &self.on_disconnect
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Action {
    pub id: String,
    pub monitor_id: String,
    pub vcp_code: u8,
    pub input_value: u8,
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        let invalid = || Error::new("invalid_config");
        if self.schema_version != 1 {
            return Err(Error::new("schema_version"));
        }
        if self.app.locale != "en-US"
            || !["system", "light", "dark"].contains(&self.app.theme.as_str())
            || ![
                "tray-only",
                "menu-bar-only",
                "window-only",
                "both",
                "headless",
            ]
            .contains(&self.app.ui_mode.as_str())
            || !["hide", "quit-gui"].contains(&self.app.close_window_action.as_str())
            || !["off", "errors-only", "all"].contains(&self.app.notifications.as_str())
            || self.app.log_retention_days == 0
            || !(4096..=100 * 1024 * 1024).contains(&self.app.log_max_bytes)
            || self.app.startup_delay_ms > 120000
        {
            return Err(invalid());
        }
        for (field, value, min, max) in [
            (
                "safety.globalMonitorCooldownMs",
                self.safety.global_monitor_cooldown_ms,
                0,
                120000,
            ),
            (
                "safety.defaultDebounceMs",
                self.safety.default_debounce_ms,
                0,
                60000,
            ),
            (
                "safety.startupGraceMs",
                self.safety.startup_grace_ms,
                0,
                120000,
            ),
            (
                "safety.resumeGraceMs",
                self.safety.resume_grace_ms,
                0,
                120000,
            ),
            (
                "safety.commandTimeoutMs",
                self.safety.command_timeout_ms,
                100,
                30000,
            ),
        ] {
            if value < min || value > max {
                return Err(Error::new("invalid_config")
                    .field(field)
                    .detail(format!("{min}..{max} ms")));
            }
        }
        let mut ids = HashSet::new();
        for id in self
            .usb_devices
            .iter()
            .map(|d| &d.id)
            .chain(self.monitors.iter().map(|m| &m.id))
            .chain(self.profiles.iter().map(|p| &p.id))
        {
            if id.is_empty() || !ids.insert(id.clone()) {
                return Err(invalid());
            }
        }
        for d in &self.usb_devices {
            if d.selectors.is_empty() {
                return Err(invalid());
            }
            for s in &d.selectors {
                if !ids.insert(s.id.clone())
                    || !["vid-pid", "serial", "instance", "topology"].contains(&s.strategy.as_str())
                    || (s.strategy == "serial" && s.serial.as_ref().is_none_or(|s| s.is_empty()))
                    || (["instance", "topology"].contains(&s.strategy.as_str())
                        && s.path.as_ref().is_none_or(|s| s.is_empty()))
                {
                    return Err(invalid());
                }
            }
        }
        for m in &self.monitors {
            if m.fingerprint.is_empty()
                || m.input_options
                    .iter()
                    .any(|i| i.value == 0 || i.vcp_code == 0)
            {
                return Err(invalid());
            }
        }
        for p in &self.profiles {
            for r in &p.rules {
                for (field, value, max) in [
                    ("cooldownMs", r.cooldown_ms, 120000),
                    ("debounceMs", r.debounce_ms, 60000),
                    ("rearmMs", r.rearm_ms, 60000),
                    (
                        "coincidenceWindowMs",
                        r.coincidence_window_ms.unwrap_or(0),
                        120000,
                    ),
                ] {
                    if value > max {
                        return Err(Error::new("invalid_config")
                            .field(format!("rules.{}.{field}", r.id))
                            .detail(format!("0..{max} ms")));
                    }
                }
                r.commands.validate()?;
                let mut devices = HashSet::new();
                let mut monitors = HashSet::new();
                if !ids.insert(r.id.clone())
                    || r.device_ids.is_empty()
                    || r.actions.is_empty()
                    || !["connect", "disconnect"].contains(&r.trigger.as_str())
                    || !["all", "any", "at-least"].contains(&r.mode.as_str())
                    || (r.mode == "at-least"
                        && (r.minimum_count == 0 || r.minimum_count > r.device_ids.len()))
                    || r.cooldown_ms > 120000
                    || r.debounce_ms > 60000
                    || r.rearm_ms > 60000
                {
                    return Err(invalid());
                }
                for d in &r.device_ids {
                    if !devices.insert(d) || !self.usb_devices.iter().any(|v| &v.id == d) {
                        return Err(invalid());
                    }
                }
                for a in &r.actions {
                    if !ids.insert(a.id.clone())
                        || !monitors.insert(&a.monitor_id)
                        || a.input_value == 0
                        || a.vcp_code == 0
                        || !self.monitors.iter().any(|m| {
                            m.id == a.monitor_id
                                && m.inputs()
                                    .iter()
                                    .any(|i| i.value == a.input_value && i.vcp_code == a.vcp_code)
                        })
                    {
                        return Err(invalid());
                    }
                }
            }
        }
        Ok(())
    }
}
pub fn presence(
    config: &Config,
    usb: &[UsbDevice],
) -> Result<std::collections::HashMap<String, bool>> {
    let mut used = HashSet::new();
    let mut out = std::collections::HashMap::new();
    let active_ids: HashSet<&str> = config
        .profiles
        .iter()
        .filter(|p| p.enabled)
        .flat_map(|p| p.rules.iter().filter(|r| r.enabled))
        .flat_map(|r| r.device_ids.iter().map(String::as_str))
        .collect();
    for d in config
        .usb_devices
        .iter()
        .filter(|d| d.enabled && active_ids.contains(d.id.as_str()))
    {
        let mut matches = usb
            .iter()
            .filter(|u| d.selectors.iter().any(|s| s.matches(u)))
            .collect::<Vec<_>>();
        matches.sort_by_key(|u| &u.physical_id);
        matches.dedup_by_key(|u| &u.physical_id);
        if matches.len() > 1 && !d.selectors.iter().all(|s| s.allow_ambiguous) {
            return Err(Error::new("ambiguous_device"));
        }
        for u in &matches {
            if !used.insert(u.physical_id.clone()) {
                return Err(Error::new("dependent_devices"));
            }
        }
        out.insert(d.id.clone(), !matches.is_empty());
    }
    Ok(out)
}
pub fn resolve_monitor<'a>(m: &SavedMonitor, live: &'a [Monitor]) -> Result<&'a Monitor> {
    let found: Vec<_> = live
        .iter()
        .filter(|x| {
            x.fingerprint == m.fingerprint && m.connector.as_ref().is_none_or(|c| c == &x.connector)
        })
        .collect();
    match found.len() {
        0 => Err(Error::new("monitor_missing")),
        1 => Ok(found[0]),
        _ => Err(Error::new("ambiguous_monitor")),
    }
}
