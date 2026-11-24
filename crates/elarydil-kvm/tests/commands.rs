use elarydil_kvm::{
    commands::{Job, Runner},
    model::CommandHooks,
};
use serde_json::Value;
use std::time::{Duration, Instant};
/// Finds the helpers Cargo built for this package, wherever the test runs from.
fn use_built_helpers() {
    let command = std::path::Path::new(env!("CARGO_BIN_EXE_elarydil-kvm-command"));
    elarydil_kvm::platform::set_helper_dir(command.parent().unwrap().to_path_buf());
}
fn job(command: &str, timeout: u64) -> Job {
    use_built_helpers();
    Job {
        id: "test".into(),
        rule_id: "rule".into(),
        profile_id: "profile".into(),
        event: "connect".into(),
        manual: true,
        hooks: CommandHooks {
            on_connect: command.into(),
            timeout_ms: timeout,
            ..Default::default()
        },
    }
}
fn finish(runner: &Runner) -> Value {
    let start = Instant::now();
    loop {
        for (kind, value) in runner.drain() {
            if [
                "command.finished",
                "command.failed",
                "command.cancelled",
                "command.skipped",
            ]
            .contains(&kind)
            {
                return value;
            }
        }
        assert!(start.elapsed() < Duration::from_secs(40), "worker stalled");
        std::thread::sleep(Duration::from_millis(10));
    }
}
#[test]
fn executes_with_metadata_and_nonzero_exit() {
    let runner = Runner::new();
    let command = if cfg!(windows) {
        "Write-Output $env:ELARYDIL_KVM_EVENT; [Console]::Error.WriteLine('problem'); exit 7"
    } else {
        "printf '%s' \"$ELARYDIL_KVM_EVENT\"; printf problem >&2; exit 7"
    };
    // Cold PowerShell startup on shared Windows runners can exceed ten seconds.
    runner.queue(job(command, 30000)).unwrap();
    let event = finish(&runner);
    assert_eq!(event["result"]["Ok"]["exitCode"], 7, "{}", event);
    assert!(event["result"]["Ok"]["stdout"]
        .as_str()
        .unwrap()
        .contains("connect"));
    assert!(event["result"]["Ok"]["stderr"]
        .as_str()
        .unwrap()
        .contains("problem"));
}
#[test]
fn output_is_bounded_and_background_stdio_does_not_block() {
    let runner = Runner::new();
    let command = if cfg!(windows) {
        "[Console]::Write('x' * 20000)"
    } else {
        "head -c 20000 /dev/zero | tr '\\000' x; sleep 2 &"
    };
    // Cold PowerShell startup on shared Windows runners can exceed ten seconds.
    runner.queue(job(command, 30000)).unwrap();
    let event = finish(&runner);
    let result = &event["result"]["Ok"];
    assert_eq!(result["exitCode"], 0, "{}", event);
    assert_eq!(result["stdout"].as_str().unwrap().len(), 4096);
    assert_eq!(result["stdoutTruncated"], true);
    #[cfg(unix)]
    assert!(result["durationMs"].as_u64().unwrap() < 1500);
}
#[test]
fn timeout_and_cancellation_release_worker() {
    let mut runner = Runner::new();
    let command = if cfg!(windows) {
        "Start-Sleep -Seconds 20"
    } else {
        "sleep 20"
    };
    runner.queue(job(command, 100)).unwrap();
    assert_eq!(finish(&runner)["result"]["Err"]["code"], "timeout");
    runner.queue(job(command, 20000)).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    let start = Instant::now();
    runner.stop();
    assert!(start.elapsed() < Duration::from_secs(2));
    assert!(runner
        .drain()
        .iter()
        .any(|(kind, _)| *kind == "command.cancelled"));
}
#[cfg(unix)]
#[test]
fn timeout_kills_grandchildren() {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("leaked");
    let mut j = job("(sleep 0.7; touch leaked) & wait", 100);
    j.hooks.working_directory = temp.path().to_string_lossy().into();
    let runner = Runner::new();
    runner.queue(j).unwrap();
    assert_eq!(finish(&runner)["result"]["Err"]["code"], "timeout");
    std::thread::sleep(Duration::from_secs(1));
    assert!(!marker.exists(), "timed-out descendant survived");
}
#[test]
fn command_validation_and_usb_types() {
    let mut hooks = CommandHooks::default();
    hooks.enabled = true;
    assert!(hooks.validate().is_err());
    hooks.on_connect = "echo okay".into();
    assert!(hooks.validate().is_ok());
    hooks.timeout_ms = 300001;
    assert!(hooks.validate().is_err());
    use elarydil_kvm::platform::usb_category;
    assert_eq!(usb_category((0, 0, 0), &[(3, 1, 1)], None), "keyboard");
    assert_eq!(usb_category((0, 0, 0), &[(3, 1, 2)], None), "mouse");
    assert_eq!(
        usb_category((0, 0, 0), &[(3, 1, 1), (3, 1, 2)], None),
        "input-composite"
    );
    assert_eq!(
        usb_category((0, 0, 0), &[(3, 1, 1)], Some("Unifying Receiver")),
        "receiver"
    );
    assert_eq!(usb_category((9, 0, 0), &[], None), "hub");
    assert_eq!(usb_category((0, 0, 0), &[(8, 6, 80)], None), "storage");
}

#[cfg(windows)]
#[test]
fn windows_timeout_kills_descendant_job() {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("leaked.txt");
    let mut j=job("start /b cmd /d /c \"ping -n 3 127.0.0.1 >nul & echo leaked > leaked.txt\" & ping -n 20 127.0.0.1 >nul", 1000);
    j.hooks.shell = "cmd".into();
    j.hooks.working_directory = temp.path().to_string_lossy().into();
    let runner = Runner::new();
    runner.queue(j).unwrap();
    assert_eq!(finish(&runner)["result"]["Err"]["code"], "timeout");
    std::thread::sleep(Duration::from_secs(3));
    assert!(
        !marker.exists(),
        "Windows Job Object left a descendant running"
    );
}
