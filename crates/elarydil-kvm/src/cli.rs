use crate::{ipc, model::*, platform, service::Service, store::Store, Error, Result};
use clap::{Arg, ArgAction, Command};
use serde_json::{json, Value};
use std::{
    fs,
    io::IsTerminal,
    process::{Command as Process, Stdio},
    time::Duration,
};
pub const OPS: &[&str] = &[
    "status",
    "doctor",
    "diagnostics.export",
    "logs",
    "events.list",
    "events.clear",
    "agent.start",
    "agent.stop",
    "agent.restart",
    "agent.status",
    "agent.enable",
    "agent.disable",
    "agent.pause",
    "agent.resume",
    "startup.status",
    "startup.enable",
    "startup.disable",
    "startup.set",
    "gui.open",
    "gui.close",
    "ui.mode.get",
    "ui.mode.set",
    "ui.theme.get",
    "ui.theme.set",
    "devices.list",
    "devices.show",
    "devices.watch",
    "devices.rescan",
    "devices.track",
    "devices.alias",
    "devices.selectors.show",
    "devices.selectors.add",
    "devices.selectors.remove",
    "wizard.start",
    "wizard.capture.start",
    "wizard.capture.status",
    "wizard.capture.finish",
    "wizard.candidates",
    "wizard.apply",
    "wizard.cancel",
    "monitors.state",
    "monitors.input.restore",
    "monitors.input.watch",
    "profiles.activate",
    "wizard.complete",
    "wizard.watch",
    "monitors.list",
    "monitors.show",
    "monitors.detect",
    "monitors.remember",
    "monitors.capabilities",
    "monitors.inputs",
    "monitors.input.get",
    "monitors.input.set",
    "monitors.input.test",
    "monitors.input.alias.set",
    "monitors.input.alias.remove",
    "profiles.list",
    "profiles.create",
    "profiles.show",
    "profiles.rename",
    "profiles.clone",
    "profiles.enable",
    "profiles.disable",
    "profiles.delete",
    "profiles.export",
    "profiles.import",
    "commands.list",
    "commands.show",
    "rules.commands.get",
    "rules.commands.set",
    "rules.commands.enable",
    "rules.commands.disable",
    "rules.commands.run",
    "rules.list",
    "rules.create",
    "rules.show",
    "rules.set",
    "rules.timing",
    "rules.add-device",
    "rules.remove-device",
    "rules.add-action",
    "rules.remove-action",
    "rules.enable",
    "rules.disable",
    "rules.simulate",
    "rules.dry-run",
    "rules.delete",
    "config.show",
    "config.get",
    "config.set",
    "config.edit",
    "config.validate",
    "config.doctor",
    "config.export",
    "config.import",
    "config.reset",
    "config.schema",
    "completion",
];
const VALUES: &[&str] = &[
    "connect-monitor",
    "disconnect-monitor",
    "connect-input",
    "disconnect-input",
    "restore-ms",
    "on-connect",
    "on-disconnect",
    "shell",
    "working-dir",
    "timeout-ms",
    "name",
    "vid",
    "pid",
    "serial",
    "path",
    "match",
    "selector",
    "devices",
    "connect-devices",
    "disconnect-devices",
    "mode",
    "min-count",
    "on",
    "event",
    "monitor",
    "input",
    "vcp",
    "value",
    "label",
    "profile",
    "cycles",
    "session",
    "debounce-ms",
    "rearm-ms",
    "cooldown-ms",
    "coincidence-window-ms",
    "component",
    "agent",
    "gui",
    "delay-ms",
    "output",
    "export",
    "limit",
    "since",
    "level",
    "duration",
];
const FLAGS: &[&str] = &[
    "reset",
    "watch",
    "wait",
    "confirm",
    "force",
    "yes",
    "non-interactive",
    "dry-run",
    "backup",
    "verified",
    "allow-ambiguous",
    "follow",
    "all",
    "connected",
    "tracked",
    "include-standard",
    "refresh",
];
fn branch(name: &str, prefix: &str) -> Command {
    let mut cmd = Command::new(name.to_string())
        .about(crate::t(&format!("cli.op.{prefix}")))
        .disable_help_subcommand(true);
    let next: std::collections::BTreeSet<_> = OPS
        .iter()
        .filter_map(|op| op.strip_prefix(&format!("{prefix}.")))
        .filter_map(|s| s.split('.').next())
        .collect();
    if next.is_empty() {
        if let Some(label) = crate::cli_help::positional(prefix) {
            cmd = cmd.arg(
                Arg::new("id")
                    .index(1)
                    .value_name(label)
                    .help(crate::t("cli.id")),
            );
        }
        if [
            "config.set",
            "rules.add-device",
            "rules.remove-device",
            "rules.remove-action",
            "rules.dry-run",
        ]
        .contains(&prefix)
        {
            cmd = cmd.arg(
                Arg::new("extra")
                    .index(2)
                    .value_name("VALUE")
                    .help(crate::t("cli.value")),
            );
        }
        let allowed = crate::cli_help::options(prefix)
            .split_whitespace()
            .collect::<Vec<_>>();
        cmd = cmd.after_long_help(format!("Examples:\n  elarydil-kvm {}\n\nDocs: docs/CLI_REFERENCE.md, docs/CLI.md, docs/COMMAND_HOOKS.md", crate::t(&format!("cli.example.{prefix}"))));
        for k in VALUES.iter().filter(|k| allowed.contains(k)) {
            cmd = cmd.arg(
                Arg::new(*k)
                    .long(*k)
                    .help(crate::t(&format!("cli.arg.{k}"))),
            )
        }
        for k in FLAGS.iter().filter(|k| allowed.contains(k)) {
            cmd = cmd.arg(
                Arg::new(*k)
                    .long(*k)
                    .action(ArgAction::SetTrue)
                    .help(crate::t(&format!("cli.arg.{k}"))),
            )
        }
    } else {
        cmd = cmd.subcommand_required(true);
        for n in next {
            cmd = cmd.subcommand(branch(n, &format!("{prefix}.{n}")))
        }
    }
    cmd
}
pub fn command() -> Command {
    let mut root = Command::new("elarydil-kvm")
        .version(env!("CARGO_PKG_VERSION"))
        .about(crate::t("cli.about"))
        .subcommand_required(true)
        .arg(
            Arg::new("json")
                .long("json")
                .global(true)
                .action(ArgAction::SetTrue)
                .help(crate::t("cli.json")),
        )
        .arg(
            Arg::new("home")
                .long("home")
                .global(true)
                .help(crate::t("cli.home")),
        );
    let groups: std::collections::BTreeSet<_> =
        OPS.iter().map(|s| s.split('.').next().unwrap()).collect();
    for group in groups {
        root = root.subcommand(branch(group, group))
    }
    root
}
pub fn parse() -> (String, Value, Option<std::path::PathBuf>, bool) {
    let matches = command().get_matches();
    let json = matches.get_flag("json");
    let home = matches.get_one::<String>("home").map(Into::into);
    let mut m = &matches;
    let mut path = vec![];
    while let Some((name, child)) = m.subcommand() {
        path.push(name.to_string());
        m = child;
    }
    let mut v = json!({});
    for key in VALUES {
        if let Ok(Some(s)) = m.try_get_one::<String>(key) {
            v[*key] = json!(s)
        }
    }
    for key in FLAGS {
        if let Ok(Some(true)) = m.try_get_one::<bool>(key) {
            v[*key] = json!(true)
        }
    }
    if let Ok(Some(id)) = m.try_get_one::<String>("id") {
        v["id"] = json!(id)
    }
    if let Ok(Some(extra)) = m.try_get_one::<String>("extra") {
        v["extra"] = json!(extra)
    }
    (path.join("."), v, home, json)
}
fn local(store: &Store, op: &str, v: Value) -> Result<Value> {
    let mut service = Service::new(store.clone())?;
    service.execute(op, v)
}
fn rpc(store: &Store, op: &str, v: Value) -> Result<Value> {
    match ipc::call(store, op, v.clone()) {
        Err(e)
            if e.code == "agent_not_running"
                && op != "rules.commands.run"
                && [
                    "config.",
                    "profiles.",
                    "rules.",
                    "ui.",
                    "startup.",
                    "devices.alias",
                    "devices.selectors.",
                ]
                .iter()
                .any(|p| op.starts_with(p)) =>
        {
            // A transient busy/unavailable IPC endpoint is not evidence that
            // the agent is stopped. Keep one authority while it owns its lock.
            for _ in 0..20 {
                match store.instance_lock() {
                    Ok(offline_guard) => {
                        // Login registration can start the agent through the OS
                        // service manager. Its config writes still use the store
                        // lock, but it must be able to acquire its instance lock.
                        if op.starts_with("startup.") {
                            drop(offline_guard);
                        }
                        return local(store, op, v);
                    }
                    Err(e) if e.code == "already_running" => {
                        std::thread::sleep(Duration::from_millis(25));
                        match ipc::call(store, op, v.clone()) {
                            Err(e) if e.code == "agent_not_running" => continue,
                            result => return result,
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
            Err(Error::new("agent_not_running"))
        }
        other => other,
    }
}
/// Where failed agent starts are recorded, outside the configuration directory so
/// the file is easy to find and survives a broken configuration:
/// `/tmp/elarydil-kvm-<uid>/agent-start.log` on macOS/Linux, `%TEMP%` on Windows.
pub fn agent_start_log_path() -> std::path::PathBuf {
    #[cfg(unix)]
    let dir = std::path::PathBuf::from(format!("/tmp/elarydil-kvm-{}", unsafe { libc::getuid() }));
    #[cfg(not(unix))]
    let dir = std::env::temp_dir().join("elarydil-kvm");
    dir.join("agent-start.log")
}

/// Appends why `agent.start` failed, with the agent's own stderr, to
/// [`agent_start_log_path`]. Logging problems are ignored.
fn record_agent_start_failure(store: &Store, error: &Error) {
    let path = agent_start_log_path();
    let Some(dir) = path.parent() else { return };
    if fs::create_dir_all(dir).is_err() {
        return;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(dir, fs::Permissions::from_mode(0o700));
    }
    if fs::metadata(&path).is_ok_and(|m| m.len() > 262_144) {
        let _ = fs::write(&path, b"");
    }
    let stderr_path = store.root.join("agent.stderr.log");
    let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
    let tail: String = {
        let chars: Vec<char> = stderr.chars().collect();
        chars[chars.len().saturating_sub(4000)..].iter().collect()
    };
    let entry = format!(
        "=== {time} agent start failed (elarydil-kvm {version}, pid {pid})\n\
         code: {code}\nmessage: {message}\ndetail: {detail}\n\
         configuration: {root}\nagent executable: {agent}\n\
         agent stderr ({stderr_path}):\n{tail}\n\n",
        time = chrono::Utc::now().to_rfc3339(),
        version = env!("CARGO_PKG_VERSION"),
        pid = std::process::id(),
        code = error.code,
        message = error.message,
        detail = error.detail.as_deref().unwrap_or("-"),
        root = store.root.display(),
        agent = platform::sibling("elarydil-kvm-agent")
            .map(|p| p.display().to_string())
            .unwrap_or_else(|e| format!("not found ({})", e.message)),
        stderr_path = stderr_path.display(),
        tail = if tail.trim().is_empty() { "(empty)" } else { tail.trim_end() },
    );
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        use std::io::Write;
        let _ = file.write_all(entry.as_bytes());
    }
}

pub fn dispatch(store: &Store, op: &str, v: Value) -> Result<Value> {
    let result = dispatch_op(store, op, v);
    if op == "agent.start" {
        if let Err(error) = &result {
            record_agent_start_failure(store, error);
        }
    }
    result
}

fn dispatch_op(store: &Store, op: &str, mut v: Value) -> Result<Value> {
    if op == "wizard.start"
        && std::io::stdin().is_terminal()
        && v["non-interactive"].as_bool() != Some(true)
        && v["yes"].as_bool() != Some(true)
    {
        return crate::wizard_cli::run(store, &v);
    }
    if op == "wizard.watch" {
        let start = std::time::Instant::now();
        let seconds = v["duration"]
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(30)
            .min(86400);
        let mut previous = Value::Null;
        while start.elapsed() < Duration::from_secs(seconds) {
            let capture = ipc::call(store, "wizard.candidates", json!({}))?;
            if capture["observations"] != previous {
                previous = capture["observations"].clone();
                println!("{}", json!({"schemaVersion":1,"observations":previous}));
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        return Ok(Value::Null);
    }
    if op == "monitors.input.watch" {
        let start = std::time::Instant::now();
        let seconds = v["duration"]
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(30)
            .min(86400);
        while start.elapsed() < Duration::from_secs(seconds) {
            let state = ipc::call(store, "monitors.state", json!({"id":v["id"],"watch":true}))?;
            println!("{}", json!({"schemaVersion":1,"result":state}));
            std::thread::sleep(Duration::from_millis(300));
        }
        return Ok(Value::Null);
    }
    if op == "devices.watch" || (op == "logs" && v["follow"].as_bool() == Some(true)) {
        let seconds = v["duration"]
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(30)
            .min(86400);
        let start = std::time::Instant::now();
        let mut sequence = 0;
        let mut previous = Value::Null;
        while start.elapsed() < Duration::from_secs(seconds) {
            let result = ipc::call(
                store,
                if op == "logs" {
                    "events.list"
                } else {
                    "devices.list"
                },
                json!({"limit":1000}),
            )?;
            if op == "logs" {
                if let Some(events) = result.as_array() {
                    for event in events.iter().rev() {
                        let n = event["monotonicSequence"].as_u64().unwrap_or(0);
                        if n > sequence {
                            println!("{}", json!({"schemaVersion":1,"event":event}));
                            sequence = n;
                        }
                    }
                }
            } else if result != previous {
                println!("{}", json!({"schemaVersion":1,"devices":result}));
                previous = result;
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        return Ok(Value::Null);
    }

    match op {
        "agent.start" => {
            if let Ok(status) = ipc::call(store, "status", json!({})) {
                if status["version"] == env!("CARGO_PKG_VERSION")
                    && status["apiRevision"].as_u64().unwrap_or(0) >= 2
                {
                    // A desktop app adopting an already running agent becomes
                    // its owner, so the agent stops when this app quits.
                    if let Some(pid) = std::env::var("ELARYDIL_KVM_OWNER_PID")
                        .ok()
                        .and_then(|v| v.parse::<u32>().ok())
                    {
                        let _ = ipc::call(store, "agent.owner", json!({"pid":pid}));
                    }
                    return Ok(status);
                }
                return dispatch(store, "agent.restart", json!({}));
            }
            // Validate before spawning so configuration errors reach the caller directly.
            store.load()?;
            let stderr_path = store.root.join("agent.stderr.log");
            if fs::metadata(&stderr_path).is_ok_and(|m| m.len() > 65536) {
                fs::write(&stderr_path, b"")?;
            }
            let stderr = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&stderr_path)?;
            let mut cmd = Process::new(platform::sibling("elarydil-kvm-agent")?);
            cmd.env("ELARYDIL_KVM_HOME", &store.root)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::from(stderr));
            let mut child = platform::spawn_background(&mut cmd)?;
            // Up to 10 s: macOS may check a newly installed agent before it runs.
            for _ in 0..200 {
                if let Ok(status) = ipc::call(store, "status", json!({})) {
                    return Ok(status);
                }
                if child.try_wait()?.is_some() {
                    let detail = fs::read_to_string(&stderr_path).unwrap_or_default();
                    return Err(Error::new("agent_not_running")
                        .detail(detail.chars().take(8192).collect::<String>()));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            return Err(Error::new("agent_not_running").detail(stderr_path.display()));
        }
        "agent.stop" if v["wait"].as_bool() == Some(true) => {
            let out = ipc::call(store, "agent.stop", json!({}))?;
            for _ in 0..600 {
                match store.instance_lock() {
                    Ok(guard) => {
                        drop(guard);
                        return Ok(out);
                    }
                    Err(e) if e.code == "already_running" => {
                        std::thread::sleep(Duration::from_millis(50))
                    }
                    Err(e) => return Err(e),
                }
            }
            return Err(Error::new("timeout"));
        }
        "agent.restart" => {
            let _ = ipc::call(store, "agent.stop", json!({}));
            for _ in 0..600 {
                if ipc::call(store, "status", json!({})).is_err() {
                    match store.instance_lock() {
                        Ok(guard) => {
                            drop(guard);
                            return dispatch(store, "agent.start", json!({}));
                        }
                        Err(e) if e.code == "already_running" => {}
                        Err(e) => return Err(e),
                    }
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            return Err(Error::new("timeout"));
        }
        "gui.open" => {
            let mut c = Process::new(platform::sibling("elarydil-kvm-gui")?);
            c.env("ELARYDIL_KVM_HOME", &store.root)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            platform::spawn_background(&mut c)?;
            return Ok(json!({"opened":true}));
        }
        "gui.close" => {
            // An agent started by the app stops with it unless it is standalone.
            let agent_continues = ipc::call(store, "status", json!({}))
                .map(|s| s["standalone"].as_bool().unwrap_or(true))
                .unwrap_or(false);
            let id = crate::model::id("close");
            store.atomic(
                "gui-close-request.json",
                &serde_json::to_vec(
                    &json!({"id":id,"createdAt":chrono::Utc::now().timestamp_millis()}),
                )?,
            )?;
            for _ in 0..60 {
                if fs::read(store.root.join("gui-close-ack.json"))
                    .ok()
                    .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
                    .is_some_and(|v| v["id"] == id)
                {
                    return Ok(json!({"closed":true,"agentContinues":agent_continues}));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            if fs::read(store.root.join("gui-close-request.json"))
                .ok()
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
                .is_some_and(|v| v["id"] == id)
            {
                let _ = fs::remove_file(store.root.join("gui-close-request.json"));
            }
            return Err(Error::new("gui_not_running"));
        }
        "completion" => {
            let shell: clap_complete::Shell = crate::service::arg(&v, "id")?
                .parse()
                .map_err(|_| Error::new("invalid_input"))?;
            clap_complete::generate(
                shell,
                &mut command(),
                "elarydil-kvm",
                &mut std::io::stdout(),
            );
            return Ok(Value::Null);
        }
        "config.export" => {
            let config = rpc(store, "config.show", json!({}))?;
            fs::write(
                crate::service::arg(&v, "id")?,
                serde_json::to_vec_pretty(&config)?,
            )?;
            return Ok(json!({"exported":true}));
        }
        "config.import" => {
            v["config"] = serde_json::from_slice(&fs::read(crate::service::arg(&v, "id")?)?)?;
        }
        "config.validate" => {
            if let Some(path) = v["id"].as_str() {
                let c: Config = serde_json::from_slice(&fs::read(path)?)?;
                c.validate()?;
                return Ok(json!({"valid":true}));
            }
        }
        "config.edit" => {
            if !std::io::stdin().is_terminal() {
                return Err(Error::new("invalid_input"));
            }
            let editor = std::env::var("EDITOR").map_err(|_| Error::new("invalid_input"))?;
            let tmp = store.root.join("config.edit.json");
            fs::write(
                &tmp,
                serde_json::to_vec_pretty(&rpc(store, "config.show", json!({}))?)?,
            )?;
            let status = Process::new(editor).arg(&tmp).status()?;
            if !status.success() {
                return Err(Error::new("invalid_input"));
            }
            v["config"] = serde_json::from_slice(&fs::read(&tmp)?)?;
            let result = rpc(store, "config.replace", v);
            let _ = fs::remove_file(tmp);
            return result;
        }
        "profiles.import" => {
            v["profileData"] = serde_json::from_slice(&fs::read(crate::service::arg(&v, "id")?)?)?;
        }
        "profiles.export" => {
            let result = rpc(store, op, v.clone())?;
            fs::write(
                crate::service::arg(&v, "output")?,
                serde_json::to_vec_pretty(&result)?,
            )?;
            return Ok(json!({"exported":true}));
        }
        "rules.add-device" | "rules.remove-device" | "rules.remove-action" | "rules.dry-run" => {
            if v["value"].is_null() {
                v["value"] = v["extra"].clone()
            }
        }
        "startup.set" => {
            let mut results = vec![];
            for component in ["agent", "gui"] {
                if let Some(on) = v[component].as_str() {
                    if !["on", "off"].contains(&on) {
                        return Err(Error::new("invalid_input"));
                    }
                    results.push(rpc(
                        store,
                        if on == "on" {
                            "startup.enable"
                        } else {
                            "startup.disable"
                        },
                        json!({"component":component}),
                    )?);
                }
            }
            if !v["delay-ms"].is_null() {
                let delay = v["delay-ms"]
                    .as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .ok_or_else(|| Error::new("invalid_input"))?;
                rpc(
                    store,
                    "config.set",
                    json!({"id":"app.startupDelayMs","value":delay}),
                )?;
            }
            return Ok(json!(results));
        }
        _ => {}
    }
    let out = rpc(
        store,
        if op == "doctor" && v["export"].is_string() {
            "diagnostics.export"
        } else {
            op
        },
        v.clone(),
    )?;
    if op == "rules.commands.run" && v["wait"].as_bool() == Some(true) {
        let start = std::time::Instant::now();
        let timeout = out["timeoutMs"].as_u64().unwrap_or(10000) + 35000;
        while start.elapsed().as_millis() < timeout as u128 {
            let event = ipc::call(store, "commands.show", json!({"id":out["jobId"]}))?;
            if [
                "command.finished",
                "command.failed",
                "command.cancelled",
                "command.skipped",
            ]
            .contains(&event["eventType"].as_str().unwrap_or(""))
            {
                return Ok(event);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        return Err(Error::new("timeout"));
    }
    if op == "doctor" || op == "diagnostics.export" {
        if let Some(path) = v["export"].as_str() {
            fs::write(path, serde_json::to_vec_pretty(&out)?)?;
        }
    }
    Ok(out)
}
pub fn main() {
    let (op, mut v, home, json_output) = parse();
    if json_output {
        v["non-interactive"] = json!(true)
    }
    let result = (|| {
        // Only terminal arguments contain JSON encoded as text. Native IPC
        // callers already pass typed JSON values to the shared dispatcher.
        if op == "config.set" {
            v["value"] = serde_json::from_str(
                v["value"]
                    .as_str()
                    .or(v["extra"].as_str())
                    .ok_or_else(|| Error::new("invalid_input"))?,
            )
            .map_err(|_| Error::new("invalid_input"))?;
        }
        Store::open(home).and_then(|s| dispatch(&s, &op, v))
    })();
    match result {
        Ok(v) => {
            let exit_code = if op == "rules.commands.run" && v["eventType"].is_string() {
                if v["result"]["result"]["Err"]["code"] == "timeout" {
                    124
                } else if v["eventType"] == "command.cancelled"
                    || v["eventType"] == "command.skipped"
                {
                    130
                } else {
                    v["result"]["result"]["Ok"]["exitCode"]
                        .as_i64()
                        .unwrap_or(1)
                        .clamp(0, 255) as i32
                }
            } else {
                0
            };
            if json_output {
                println!("{}", json!({"schemaVersion":1,"result":v}))
            } else if !v.is_null() {
                println!("{}", serde_json::to_string_pretty(&v).unwrap())
            }
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
        }
        Err(e) => {
            if json_output {
                println!("{}", json!({"schemaVersion":1,"error":e}))
            } else {
                eprintln!("{e}")
            }
            std::process::exit(match e.code.as_str() {
                "invalid_input" | "invalid_config" | "config_unreadable" | "invalid_command_config" => 2,
                "agent_not_running" => 3,
                "monitor_missing" => 4,
                "ddc_unavailable" | "unsupported" => 5,
                "permission_denied" | "authentication" => 6,
                "confirmation_required" | "cooldown" | "advanced_confirmation" => 7,
                "timeout" => 8,
                "ambiguous_monitor" | "ambiguous_device" => 9,
                _ => 1,
            })
        }
    }
}
