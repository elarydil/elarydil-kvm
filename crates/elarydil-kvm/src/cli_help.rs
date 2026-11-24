pub fn options(op: &str) -> &'static str {
    match op {
        "wizard.watch" => "duration",
        "monitors.state" => "watch",
        "monitors.input.restore" => "",
        "monitors.input.watch" => "duration",
        "profiles.activate" => "",
        "wizard.complete" => "devices connect-devices disconnect-devices name mode min-count connect-monitor disconnect-monitor connect-input disconnect-input dry-run session",
        "status" => "",
        "doctor" => "export",
        "diagnostics.export" => "export",
        "logs" => "limit since level profile follow duration",
        "events.list" => "limit since level profile",
        "events.clear" => "confirm",
        "agent.start" => "",
        "agent.stop" => "wait",
        "agent.restart" => "",
        "agent.status" => "",
        "agent.enable" => "",
        "agent.disable" => "",
        "agent.pause" => "",
        "agent.resume" => "",
        "startup.status" => "component",
        "startup.enable" => "component",
        "startup.disable" => "component",
        "startup.set" => "agent gui delay-ms",
        "gui.open" => "",
        "gui.close" => "",
        "ui.theme.get" => "",
        "ui.theme.set" => "",
        "ui.mode.get" => "",
        "ui.mode.set" => "",
        "devices.list" => "connected tracked",
        "devices.show" => "",
        "devices.track" => "name",
        "devices.alias" => "name",
        "devices.selectors.show" => "",
        "devices.watch" => "duration",
        "devices.rescan" => "",
        "devices.selectors.add" => "match vid pid serial path allow-ambiguous",
        "devices.selectors.remove" => "selector",
        "wizard.start" => "cycles non-interactive yes",
        "wizard.capture.start" => "cycles non-interactive yes",
        "wizard.capture.status" => "session",
        "wizard.capture.finish" => "session",
        "wizard.candidates" => "session",
        "wizard.cancel" => "session",
        "wizard.apply" => "name devices mode min-count on monitor input vcp debounce-ms rearm-ms cooldown-ms coincidence-window-ms session profile dry-run",
        "monitors.list" => "",
        "monitors.detect" => "",
        "monitors.show" => "",
        "monitors.remember" => "name",
        "monitors.capabilities" => "",
        "monitors.inputs" => "",
        "monitors.input.get" => "vcp",
        "monitors.input.set" => "value input vcp confirm force",
        "monitors.input.test" => "value input vcp confirm force restore-ms",
        "monitors.input.alias.set" => "value vcp label verified confirm",
        "monitors.input.alias.remove" => "value reset",
        "profiles.list" => "",
        "profiles.create" => "name",
        "profiles.show" => "",
        "profiles.rename" => "name",
        "profiles.clone" => "name",
        "profiles.enable" => "",
        "profiles.disable" => "",
        "profiles.delete" => "confirm",
        "profiles.export" => "output",
        "profiles.import" => "dry-run",
        "rules.list" => "profile",
        "rules.create" => "name devices mode min-count on monitor input vcp debounce-ms rearm-ms cooldown-ms coincidence-window-ms profile dry-run",
        "rules.show" => "",
        "rules.set" => "name devices mode min-count on monitor input vcp debounce-ms rearm-ms cooldown-ms coincidence-window-ms",
        "rules.timing" => "debounce-ms rearm-ms cooldown-ms coincidence-window-ms",
        "rules.add-device" => "value",
        "rules.remove-device" => "value",
        "rules.add-action" => "monitor input vcp",
        "rules.remove-action" => "value",
        "rules.enable" => "",
        "rules.disable" => "",
        "rules.simulate" => "event",
        "rules.dry-run" => "value",
        "rules.delete" => "confirm",
        "rules.commands.get" => "",
        "rules.commands.set" => "on-connect on-disconnect shell working-dir timeout-ms",
        "rules.commands.enable" => "",
        "rules.commands.disable" => "",
        "rules.commands.run" => "event confirm wait",
        "commands.list" => "limit",
        "commands.show" => "",
        "config.show" => "",
        "config.get" => "",
        "config.set" => "value",
        "config.edit" => "",
        "config.validate" => "",
        "config.doctor" => "",
        "config.export" => "",
        "config.import" => "dry-run",
        "config.reset" => "confirm",
        "config.schema" => "",
        "completion" => "",
        _ => "",
    }
}

pub fn positional(op: &str) -> Option<&'static str> {
    if op.starts_with("rules.") && !["rules.create", "rules.list"].contains(&op) {
        return Some("RULE");
    }
    if op.starts_with("profiles.") && !["profiles.create", "profiles.list"].contains(&op) {
        return Some(if op == "profiles.import" {
            "PATH"
        } else {
            "PROFILE"
        });
    }
    if op.starts_with("devices.")
        && !["devices.list", "devices.watch", "devices.rescan"].contains(&op)
    {
        return Some("DEVICE");
    }
    if op.starts_with("monitors.") && !["monitors.list", "monitors.detect"].contains(&op) {
        return Some("MONITOR");
    }
    match op {
        "commands.show" => Some("JOB"),
        "config.get" | "config.set" => Some("KEY"),
        "config.validate" | "config.import" | "config.export" => Some("PATH"),
        "ui.theme.set" | "ui.mode.set" => Some("VALUE"),
        "completion" => Some("SHELL"),
        _ => None,
    }
}
