use crate::{ipc, model::number, store::Store, t, Error, Result};
use serde_json::{json, Value};
use std::io::{self, Write};
fn prompt(key: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", t(key), default);
    io::stdout().flush()?;
    let mut input = String::new();
    if io::stdin().read_line(&mut input)? == 0 {
        return Err(Error::new("invalid_input"));
    }
    let value = input.trim();
    Ok(if value.is_empty() {
        default.into()
    } else {
        value.into()
    })
}
fn choose(key: &str, len: usize) -> Result<usize> {
    let n = prompt(key, "1")?
        .parse::<usize>()
        .map_err(|_| Error::new("invalid_input"))?;
    if n == 0 || n > len {
        return Err(Error::new("invalid_input"));
    }
    Ok(n - 1)
}
pub fn run(store: &Store, v: &Value) -> Result<Value> {
    crate::cli::dispatch(store, "agent.start", json!({}))?;
    let start = ipc::call(store, "wizard.start", v.clone())?;
    println!("{}", t("wizard.switch"));
    prompt("wizard.finishPrompt", "")?;
    let captured = ipc::call(store, "wizard.capture.finish", json!({}))?;
    let observations = captured["observations"]
        .as_array()
        .ok_or_else(|| Error::new("capture_incomplete"))?;
    if observations.is_empty() {
        return Err(Error::new("capture_incomplete"));
    }
    for (i, o) in observations.iter().enumerate() {
        println!(
            "{}",
            t("wizard.observation")
                .replace("{row}", &(i + 1).to_string())
                .replace("{name}", o["device"]["name"].as_str().unwrap_or_default())
                .replace("{disconnect}", &o["disconnectedAtMs"].to_string())
                .replace("{connect}", &o["connectedAtMs"].to_string())
        );
    }
    let default = (1..=observations.len())
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let selection = prompt("wizard.selectPrompt", &default)?;
    let mut devices = vec![];
    for item in selection.split(',') {
        let n = item
            .trim()
            .parse::<usize>()
            .map_err(|_| Error::new("invalid_input"))?;
        let o = observations
            .get(
                n.checked_sub(1)
                    .ok_or_else(|| Error::new("invalid_input"))?,
            )
            .ok_or_else(|| Error::new("invalid_input"))?;
        devices.push(o["device"]["id"].clone());
    }
    let monitors = ipc::call(store, "monitors.list", json!({}))?;
    let live = monitors["live"]
        .as_array()
        .ok_or_else(|| Error::new("monitor_missing"))?;
    if live.is_empty() {
        return Err(Error::new("monitor_missing"));
    }
    for (i, m) in live.iter().enumerate() {
        println!("{}. {}", i + 1, m["name"].as_str().unwrap_or_default());
    }
    let mut payload = json!({"session":start["session"]["id"],"devices":devices,"name":t("wizard.default_profile"),"dry-run":v["dry-run"].as_bool().unwrap_or(false)});
    for (event, default) in [("connect", "17"), ("disconnect", "18")] {
        let monitor = &live[choose(&format!("wizard.{event}MonitorPrompt"), live.len())?];
        let id = monitor["id"]
            .as_str()
            .ok_or_else(|| Error::new("invalid_input"))?;
        let input = number(&prompt(&format!("wizard.{event}InputPrompt"), default)?)?;
        ipc::call(store, "monitors.remember", json!({"id":id}))?;
        if ![15, 16, 17, 18].contains(&input) {
            ipc::call(
                store,
                "monitors.input.alias.set",
                json!({"id":id,"value":input,"label":t("ui.inputNumber").replace("{value}",&input.to_string())}),
            )?;
        }
        payload[format!("{event}-monitor")] = json!(id);
        payload[format!("{event}-input")] = json!(input);
    }
    ipc::call(store, "wizard.complete", payload)
}
