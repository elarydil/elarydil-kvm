//! User-configured, local shell commands. The agent never waits on a shell.
use crate::{model::*, platform, Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::{Duration, Instant},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub rule_id: String,
    pub profile_id: String,
    pub event: String,
    pub manual: bool,
    pub hooks: CommandHooks,
}
impl Job {
    pub fn metadata(&self) -> Value {
        json!({"jobId":self.id,"ruleId":self.rule_id,"profileId":self.profile_id,"event":self.event,"manual":self.manual,"shell":self.hooks.shell,"timeoutMs":self.hooks.timeout_ms})
    }
}
pub struct Runner {
    tx: Option<mpsc::SyncSender<(Job, Instant)>>,
    rx: mpsc::Receiver<(&'static str, Value)>,
    cancel: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}
impl Runner {
    pub fn new() -> Self {
        let (tx, jobs) = mpsc::sync_channel::<(Job, Instant)>(16);
        let (events, rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let stop = cancel.clone();
        let thread = std::thread::spawn(move || {
            while let Ok((job, queued)) = jobs.recv() {
                let mut data = job.metadata();
                if stop.load(Ordering::SeqCst) || queued.elapsed() > Duration::from_secs(30) {
                    let _ = events.send(("command.skipped", data));
                    continue;
                }
                let _ = events.send(("command.started", data.clone()));
                let result = (|| -> Result<Value> {
                    #[allow(unused_mut)]
                    let mut command = Command::new(platform::sibling("elarydil-kvm-command")?);
                    #[cfg(unix)]
                    {
                        use std::os::unix::process::CommandExt;
                        command.process_group(0);
                    }
                    let raw = platform::bounded_cancel(
                        command,
                        Some(&serde_json::to_vec(&job)?),
                        job.hooks.timeout_ms,
                        Some(&stop),
                    )?;
                    serde_json::from_slice::<Result<Value>>(&raw)?
                })();
                let kind = match &result {
                    Ok(v) if v["exitCode"] == 0 => "command.finished",
                    Err(e) if e.code == "command_cancelled" => "command.cancelled",
                    _ => "command.failed",
                };
                data["result"] = json!(result);
                let _ = events.send((kind, data));
            }
        });
        Self {
            tx: Some(tx),
            rx,
            cancel,
            thread: Some(thread),
        }
    }
    pub fn queue(&self, job: Job) -> Result<()> {
        self.tx
            .as_ref()
            .ok_or_else(|| Error::new("command_cancelled"))?
            .try_send((job, Instant::now()))
            .map_err(|_| Error::new("command_busy"))
    }
    pub fn drain(&self) -> Vec<(&'static str, Value)> {
        self.rx.try_iter().collect()
    }
    pub fn stop(&mut self) {
        self.cancel.store(true, Ordering::SeqCst);
        self.tx.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
impl Drop for Runner {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Synthetic rules reuse the exact USB grouping, debounce, rearm and dry-run logic.
pub fn matching_config(config: &Config) -> Config {
    let mut copy = config.clone();
    for profile in &mut copy.profiles {
        profile.rules = profile
            .rules
            .iter()
            .flat_map(|rule| {
                ["connect", "disconnect"]
                    .into_iter()
                    .filter_map(move |event| {
                        if !rule.commands.enabled || rule.commands.command(event).trim().is_empty()
                        {
                            return None;
                        }
                        let mut r = rule.clone();
                        r.id = format!("{event}:{}", rule.id);
                        r.trigger = event.into();
                        r.actions.clear();
                        r.commands = CommandHooks::default();
                        Some(r)
                    })
            })
            .collect();
    }
    copy
}

// Read only bytes already available. A deliberately launched background process
// may retain stdio handles after its shell exits; waiting for EOF would hang.
fn available<T: Read>(
    pipe: &mut T,
    ready: usize,
    output: &mut Vec<u8>,
    truncated: &mut bool,
) -> std::io::Result<()> {
    let mut left = ready.min(65536);
    let mut buf = [0u8; 4096];
    while left > 0 {
        let cap = left.min(buf.len());
        let count = pipe.read(&mut buf[..cap])?;
        if count == 0 {
            break;
        }
        let keep = count.min(4096usize.saturating_sub(output.len()));
        output.extend_from_slice(&buf[..keep]);
        *truncated |= keep < count;
        left -= count;
    }
    Ok(())
}
#[cfg(unix)]
fn ready<T: std::os::fd::AsRawFd>(pipe: &T) -> usize {
    let mut count: libc::c_int = 0;
    if unsafe { libc::ioctl(pipe.as_raw_fd(), libc::FIONREAD, &mut count) } == 0 {
        count.max(0) as usize
    } else {
        0
    }
}
#[cfg(windows)]
fn ready<T: std::os::windows::io::AsRawHandle>(pipe: &T) -> usize {
    let mut count = 0;
    unsafe {
        windows_sys::Win32::System::Pipes::PeekNamedPipe(
            pipe.as_raw_handle(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            &mut count,
            std::ptr::null_mut(),
        );
    }
    count as usize
}

#[cfg(windows)]
fn process_job() -> Result<windows_sys::Win32::Foundation::HANDLE> {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::{JobObjects::*, Threading::GetCurrentProcess},
    };
    // Non-inheritable handle owned only by this one-shot helper. Killing the
    // helper closes the job and terminates its entire shell process tree.
    let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
    if handle.is_null() {
        return Err(std::io::Error::last_os_error().into());
    }
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    if unsafe {
        SetInformationJobObject(
            handle,
            JobObjectExtendedLimitInformation,
            &limits as *const _ as *const _,
            std::mem::size_of_val(&limits) as u32,
        )
    } == 0
        || unsafe { AssignProcessToJobObject(handle, GetCurrentProcess()) } == 0
    {
        let error = std::io::Error::last_os_error();
        unsafe {
            CloseHandle(handle);
        }
        return Err(error.into());
    }
    Ok(handle)
}
#[cfg(windows)]
fn allow_background(handle: windows_sys::Win32::Foundation::HANDLE) -> Result<()> {
    use windows_sys::Win32::System::JobObjects::*;
    let limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe {
        SetInformationJobObject(
            handle,
            JobObjectExtendedLimitInformation,
            &limits as *const _ as *const _,
            std::mem::size_of_val(&limits) as u32,
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}
fn run_shell(job: &Job) -> Result<Value> {
    job.hooks.validate()?;
    if !["connect", "disconnect"].contains(&job.event.as_str())
        || job.hooks.command(&job.event).trim().is_empty()
    {
        return Err(Error::new("invalid_command_config"));
    }
    let shell = match job.hooks.shell.as_str() {
        "system" if cfg!(windows) => "powershell.exe",
        "system" if cfg!(target_os = "macos") => "/bin/zsh",
        "system" => "/bin/sh",
        "bash" if cfg!(windows) => "bash.exe",
        "bash" => "/bin/bash",
        "powershell" if cfg!(windows) => "powershell.exe",
        "powershell" => "pwsh",
        "cmd" if cfg!(windows) => "cmd.exe",
        _ => return Err(Error::new("shell_unavailable")),
    };
    let mut command = Command::new(shell);
    if shell.contains("powershell") || shell == "pwsh" {
        command.args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command"]);
        command.arg(format!(
            "[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new(); {}",
            job.hooks.command(&job.event)
        ));
    } else if shell == "cmd.exe" {
        command
            .args(["/D", "/S", "/C"])
            .arg(job.hooks.command(&job.event));
    } else {
        command.arg("-c").arg(job.hooks.command(&job.event));
    }
    let home = directories::UserDirs::new()
        .ok_or_else(|| Error::new("io"))?
        .home_dir()
        .to_path_buf();
    let directory = if job.hooks.working_directory.is_empty() {
        home
    } else {
        home.join(&job.hooks.working_directory)
    };
    command
        .current_dir(directory)
        .env("ELARYDIL_KVM_EVENT", &job.event)
        .env("ELARYDIL_KVM_RULE_ID", &job.rule_id)
        .env("ELARYDIL_KVM_PROFILE_ID", &job.profile_id)
        .env("ELARYDIL_KVM_JOB_ID", &job.id)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    let process_group = process_job()?;
    let start = Instant::now();
    let mut child = platform::spawn_background(&mut command)
        .map_err(|e| Error::new("shell_unavailable").detail(e))?;
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let (mut out, mut err) = (vec![], vec![]);
    let (mut out_truncated, mut err_truncated) = (false, false);
    let status = loop {
        let count = ready(&stdout);
        available(&mut stdout, count, &mut out, &mut out_truncated)?;
        let count = ready(&stderr);
        available(&mut stderr, count, &mut err, &mut err_truncated)?;
        if let Some(status) = child.try_wait()? {
            break status;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    for _ in 0..4 {
        let (a, b) = (ready(&stdout), ready(&stderr));
        available(&mut stdout, a, &mut out, &mut out_truncated)?;
        available(&mut stderr, b, &mut err, &mut err_truncated)?;
        if a == 0 && b == 0 {
            break;
        }
    }
    #[cfg(windows)]
    allow_background(process_group)?;
    Ok(
        json!({"exitCode":status.code().unwrap_or(1),"stdout":String::from_utf8_lossy(&out),"stderr":String::from_utf8_lossy(&err),"stdoutTruncated":out_truncated,"stderrTruncated":err_truncated,"durationMs":start.elapsed().as_millis() as u64}),
    )
}
pub fn helper_main() {
    let result = (|| -> Result<Value> {
        let mut input = vec![];
        std::io::stdin().take(262144).read_to_end(&mut input)?;
        run_shell(&serde_json::from_slice(&input)?)
    })();
    println!("{}", serde_json::to_string(&result).unwrap());
    let _ = std::io::stdout().flush();
}
