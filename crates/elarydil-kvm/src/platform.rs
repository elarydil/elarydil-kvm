use crate::{model::*, Error, Result};
use nusb::MaybeFuture;
use serde_json::{json, Value};
use std::{
    fs,
    io::{Read, Write},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub fn usb() -> Result<Vec<UsbDevice>> {
    nusb::list_devices()
        .wait()
        .map(|it| it.map(usb_info).collect())
        .map_err(|e| Error::new("usb_unavailable").detail(e))
}
pub fn usb_info(d: nusb::DeviceInfo) -> UsbDevice {
    let path = format!("{}:{:?}", d.bus_id(), d.port_chain());
    let model = d.product_string().map(str::to_owned);
    let interfaces = d
        .interfaces()
        .map(|i| (i.class(), i.subclass(), i.protocol()))
        .collect::<Vec<_>>();
    let category = usb_category(
        (d.class(), d.subclass(), d.protocol()),
        &interfaces,
        d.product_string(),
    );
    UsbDevice {
        id: format!("usb_{}", hash(path.as_bytes())),
        physical_id: path.clone(),
        name: model
            .clone()
            .unwrap_or_else(|| format!("USB {:04X}:{:04X}", d.vendor_id(), d.product_id())),
        model,
        manufacturer: d.manufacturer_string().unwrap_or_default().into(),
        vid: d.vendor_id(),
        pid: d.product_id(),
        serial: d.serial_number().map(str::to_owned),
        parent: path.rsplit_once(',').map(|(x, _)| x.into()),
        path,
        category: category.into(),
        connected: true,
        last_seen: chrono::Utc::now().to_rfc3339(),
    }
}
pub fn usb_category(
    device: (u8, u8, u8),
    interfaces: &[(u8, u8, u8)],
    product: Option<&str>,
) -> &'static str {
    let classes = std::iter::once(&device)
        .chain(interfaces.iter())
        .copied()
        .collect::<Vec<_>>();
    if classes.iter().any(|c| c.0 == 9) {
        return "hub";
    }
    let keyboard = classes.iter().any(|c| *c == (3, 1, 1));
    let mouse = classes.iter().any(|c| *c == (3, 1, 2));
    let hid = classes.iter().any(|c| c.0 == 3);
    if hid && product.is_some_and(|s| s.to_ascii_lowercase().contains("receiver")) {
        return "receiver";
    }
    if keyboard && mouse {
        return "input-composite";
    }
    if keyboard {
        return "keyboard";
    }
    if mouse {
        return "mouse";
    }
    for (class, _, _) in classes {
        match class {
            1 => return "audio",
            8 => return "storage",
            14 => return "camera",
            7 => return "printer",
            2 | 10 => return "network",
            224 => return "wireless",
            _ => (),
        }
    }
    if hid {
        "hid"
    } else {
        "other"
    }
}
/// Whether a process with this PID still exists (used to tie the agent's
/// lifetime to the desktop app that started it).
pub fn process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        // Signal 0 performs only the existence and permission check.
        let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
        result == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::{CloseHandle, STILL_ACTIVE};
        use windows_sys::Win32::System::Threading::{
            GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
        };
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                return false;
            }
            let mut code = 0u32;
            let ok = GetExitCodeProcess(handle, &mut code) != 0;
            CloseHandle(handle);
            ok && code == STILL_ACTIVE as u32
        }
    }
}
static HELPER_DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// Points [`sibling`] at the directory with the built helpers first. Tests use
/// it because Cargo may run test executables from a separate build directory,
/// away from the binaries it builds.
pub fn set_helper_dir(dir: std::path::PathBuf) {
    let _ = HELPER_DIR.set(dir);
}

pub fn sibling(name: &str) -> Result<std::path::PathBuf> {
    let file = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    if let Some(path) = HELPER_DIR.get().map(|dir| dir.join(&file)) {
        if path.is_file() {
            return Ok(path);
        }
    }
    let exe = std::env::current_exe()?;
    let parent = exe.parent().ok_or_else(|| Error::new("io"))?;
    for path in [
        parent.join(&file),
        parent.join("bin").join(&file),
        parent.join("..").join(&file),
        parent.join("../../MacOS").join(&file),
        parent.join("../Resources/bin").join(&file),
        parent.join("../lib/elarydil-kvm/bin").join(&file),
    ] {
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(Error::new(if name == "elarydil-kvm-gui" {
        "gui_unavailable"
    } else {
        "ddc_unavailable"
    })
    .detail(name))
}
pub fn spawn_background(command: &mut Command) -> Result<std::process::Child> {
    #[cfg(windows)]
    {
        use std::os::windows::{io::AsRawHandle, process::CommandExt};
        use windows_sys::Win32::Foundation::{
            SetHandleInformation, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE,
        };
        // Rust's Windows process launcher inherits all inheritable handles.
        // Even with redirected child stdio, copies of the CLI's original pipe
        // handles would keep shell/Python readers waiting for EOF until exit
        // of the long-lived agent or GUI. Command duplicates its explicit
        // child stdio separately, so those handles remain available to it.
        for handle in [
            std::io::stdin().as_raw_handle(),
            std::io::stdout().as_raw_handle(),
            std::io::stderr().as_raw_handle(),
        ] {
            if !handle.is_null()
                && handle != INVALID_HANDLE_VALUE
                && unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0) } == 0
            {
                return Err(std::io::Error::last_os_error().into());
            }
        }
        command.creation_flags(0x08000000);
    }
    Ok(command.spawn()?)
}
pub fn bounded(command: Command, input: Option<&[u8]>, timeout: u64) -> Result<Vec<u8>> {
    bounded_cancel(command, input, timeout, None)
}
pub fn bounded_cancel(
    mut command: Command,
    input: Option<&[u8]>,
    timeout: u64,
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> Result<Vec<u8>> {
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = spawn_background(&mut command)?;
    if let Some(data) = input {
        child.stdin.take().unwrap().write_all(data)?;
    } else {
        drop(child.stdin.take());
    }
    let mut stdout = child.stdout.take().unwrap();
    let reader = std::thread::spawn(move || {
        let mut data = Vec::new();
        stdout
            .by_ref()
            .take(1024 * 1024)
            .read_to_end(&mut data)
            .map(|_| data)
    });
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            let data = reader.join().map_err(|_| Error::new("io"))??;
            return if status.success() {
                Ok(data)
            } else {
                Err(Error::new("ddc_unavailable").detail(String::from_utf8_lossy(&data)))
            };
        }
        if start.elapsed().as_millis() > timeout as u128
            || cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::SeqCst))
        {
            #[cfg(unix)]
            {
                let pid = child.id() as i32;
                if unsafe { libc::getpgid(pid) } == pid {
                    unsafe {
                        libc::kill(-pid, libc::SIGKILL);
                    }
                } else {
                    let _ = child.kill();
                }
            }
            #[cfg(windows)]
            let _ = child.kill();
            let _ = child.wait();
            drop(reader);
            return Err(Error::new(
                if cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::SeqCst)) {
                    "command_cancelled"
                } else {
                    "timeout"
                },
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
pub fn ddc(m: &Monitor, op: &str, vcp: u8, value: Option<u8>, timeout: u64) -> Result<Value> {
    let mut c = Command::new(sibling("elarydil-kvm-ddc")?);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        c.process_group(0);
    }
    c.arg(op)
        .arg(vcp.to_string())
        .arg(value.unwrap_or(0).to_string());
    let raw = bounded(c, Some(&serde_json::to_vec(m)?), timeout)?;
    let result: std::result::Result<Value, Error> = serde_json::from_slice(&raw)?;
    result
}
#[cfg(target_os = "linux")]
pub fn monitors() -> Result<Vec<Monitor>> {
    let mut out = vec![];
    for entry in fs::read_dir("/sys/class/drm")
        .map_err(|_| Error::new("ddc_unavailable").detail("/sys/class/drm unavailable"))?
    {
        let entry = entry?;
        let path = entry.path();
        if fs::read_to_string(path.join("status"))
            .unwrap_or_default()
            .trim()
            != "connected"
        {
            continue;
        }
        let connector = entry.file_name().to_string_lossy().into_owned();
        if connector.contains("eDP") || connector.contains("LVDS") {
            continue;
        }
        let edid = fs::read(path.join("edid")).unwrap_or_default();
        if !valid_edid(&edid) {
            continue;
        }
        let mut fingerprint_edid = edid[..128].to_vec();
        fingerprint_edid[126] = 0;
        fingerprint_edid[127] = 0;
        let fingerprint = hash(&fingerprint_edid);
        let bus = fs::read_link(path.join("ddc"))
            .ok()
            .and_then(|p| {
                p.file_name()
                    .map(|n| format!("/dev/{}", n.to_string_lossy()))
            })
            .unwrap_or_default();
        let permissions = if bus.is_empty() {
            "unavailable"
        } else if fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&bus)
            .is_ok()
        {
            "granted"
        } else {
            "denied"
        };
        let name = edid[54..126]
            .chunks(18)
            .find(|d| d.get(..5) == Some(&[0, 0, 0, 0xfc, 0]))
            .map(|d| String::from_utf8_lossy(&d[5..]).trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("EDID {:02X}{:02X}", edid[10], edid[11]));
        out.push(Monitor {
            id: format!(
                "monitor_{}",
                hash(format!("{fingerprint}:{connector}").as_bytes())
            ),
            name,
            fingerprint,
            connector,
            backend: "linux-i2c-dev".into(),
            path: bus,
            edid_available: true,
            permissions: permissions.into(),
            limitations: vec!["hardware_not_verified".into()],
        });
    }
    Ok(out)
}
#[cfg(target_os = "linux")]
pub fn ddc_direct(m: &Monitor, op: &str, vcp: u8, value: u8) -> Result<Value> {
    // Revalidate EDID and route in the helper immediately before opening the bus.
    let live = monitors()?;
    let matches: Vec<_> = live
        .iter()
        .filter(|x| {
            x.fingerprint == m.fingerprint && x.connector == m.connector && x.path == m.path
        })
        .collect();
    if matches.len() != 1 {
        return Err(Error::new("monitor_missing"));
    }
    use std::os::fd::AsRawFd;
    let mut f = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&m.path)?;
    if unsafe { libc::ioctl(f.as_raw_fd(), 0x0703, 0x37) } < 0 {
        return Err(Error::new("ddc_unavailable"));
    }
    let send = |f: &mut fs::File, payload: &[u8]| -> Result<()> {
        let mut packet = vec![0x51, 0x80 | payload.len() as u8];
        packet.extend_from_slice(payload);
        packet.push(packet.iter().fold(0x6e, |c, x| c ^ x));
        f.write_all(&packet)?;
        std::thread::sleep(Duration::from_millis(50));
        Ok(())
    };
    if op == "write" {
        send(&mut f, &[0x03, vcp, 0, value])?;
        return Ok(json!({"outcome":"write_api_success","verified":false,"value":value}));
    }
    if op == "read" {
        send(&mut f, &[0x01, vcp])?;
        let mut response = [0u8; 11];
        f.read_exact(&mut response)?;
        if response[0] != 0x6e
            || response[2] != 2
            || response[3] != 0
            || response[4] != vcp
            || response.iter().fold(0x50, |c, x| c ^ x) != 0
        {
            return Err(Error::new("ddc_unavailable"));
        }
        return Ok(
            json!({"outcome":"read_back","value":u16::from_be_bytes([response[8],response[9]]),"maximum":u16::from_be_bytes([response[6],response[7]])}),
        );
    }
    if op == "capabilities" {
        let mut data = Vec::new();
        for _ in 0..128 {
            let off = data.len() as u16;
            send(&mut f, &[0xf3, (off >> 8) as u8, off as u8])?;
            let mut buf = [0u8; 38];
            let n = f.read(&mut buf)?;
            if n < 6
                || buf[0] != 0x6e
                || buf[2] != 0xe3
                || u16::from_be_bytes([buf[3], buf[4]]) != off
            {
                return Err(Error::new("ddc_unavailable"));
            }
            let len = (buf[1] & 0x7f) as usize;
            if len < 3 || len + 3 > n || buf[..len + 3].iter().fold(0x50, |c, x| c ^ x) != 0 {
                return Err(Error::new("ddc_unavailable"));
            }
            let chunk = &buf[5..len + 2];
            if chunk.is_empty() {
                break;
            }
            for b in chunk {
                if *b == 0 {
                    return Ok(
                        json!({"raw":String::from_utf8_lossy(&data),"values":capability_inputs(&String::from_utf8_lossy(&data))}),
                    );
                }
                data.push(*b)
            }
        }
        return Ok(
            json!({"raw":String::from_utf8_lossy(&data),"values":capability_inputs(&String::from_utf8_lossy(&data))}),
        );
    }
    Err(Error::new("unsupported"))
}
pub fn valid_edid(bytes: &[u8]) -> bool {
    bytes.len() >= 128
        && bytes[..8] == [0, 255, 255, 255, 255, 255, 255, 0]
        && bytes[..128].iter().fold(0u8, |sum, b| sum.wrapping_add(*b)) == 0
}

pub fn capability_inputs(s: &str) -> Vec<u8> {
    let lower = s.to_lowercase();
    let Some(vcp) = lower.find("vcp(") else {
        return vec![];
    };
    let bytes = lower.as_bytes();
    let mut i = vcp + 4;
    let mut depth = 1;
    while i < bytes.len() && depth > 0 {
        if bytes[i] == b'(' {
            depth += 1;
            i += 1;
            continue;
        }
        if bytes[i] == b')' {
            depth -= 1;
            i += 1;
            continue;
        }
        if depth == 1 && bytes[i].is_ascii_hexdigit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_hexdigit() {
                i += 1
            }
            let code = &lower[start..i];
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1
            }
            if code == "60" && bytes.get(i) == Some(&b'(') {
                let end = lower[i + 1..].find(')').map(|n| i + 1 + n).unwrap_or(i + 1);
                return lower[i + 1..end]
                    .split_whitespace()
                    .filter_map(|v| u8::from_str_radix(v, 16).ok())
                    .collect();
            }
        } else {
            i += 1
        }
    }
    vec![]
}

#[cfg(windows)]
pub use crate::windows_ddc::{direct as ddc_direct, monitors};

#[cfg(target_os = "macos")]
pub fn monitors() -> Result<Vec<Monitor>> {
    if std::env::consts::ARCH != "aarch64" {
        return Err(Error::new("unsupported"));
    }
    let mut c = Command::new(sibling("m1ddc")?);
    c.args(["display", "list", "json"]);
    let raw = bounded(c, None, 3000)?;
    let list: Vec<Value> = serde_json::from_slice(&raw)?;
    Ok(list
        .into_iter()
        .filter_map(|v| {
            let uuid = v["uuid"].as_str()?.to_owned();
            Some(Monitor {
                id: format!("monitor_{}", hash(uuid.as_bytes())),
                name: v["name"].as_str().unwrap_or_default().into(),
                fingerprint: uuid.clone(),
                connector: v["location"].as_str().unwrap_or_default().into(),
                path: uuid,
                backend: "apple-silicon-ioavservice".into(),
                edid_available: true,
                permissions: "unknown".into(),
                limitations: vec![
                    "private_api_connector_dependent".into(),
                    "hardware_not_verified".into(),
                ],
            })
        })
        .collect())
}
#[cfg(target_os = "macos")]
pub fn ddc_direct(m: &Monitor, op: &str, vcp: u8, value: u8) -> Result<Value> {
    let live = monitors()?;
    let same: Vec<_> = live
        .iter()
        .filter(|d| d.fingerprint == m.fingerprint)
        .collect();
    if same.len() > 1 {
        return Err(Error::new("ambiguous_monitor"));
    }
    if !same.iter().any(|d| d.connector == m.connector) {
        return Err(Error::new("monitor_missing"));
    }
    let attr = match vcp {
        0x60 => "input",
        0xf4 => "input-alt",
        _ => return Err(Error::new("unsupported")),
    };
    let mut c = Command::new(sibling("m1ddc")?);
    c.args([
        "display",
        &format!("uuid={}", m.fingerprint),
        if op == "write" {
            "set"
        } else if op == "read" {
            "get"
        } else {
            return Err(Error::new("unsupported"));
        },
        attr,
    ]);
    if op == "write" {
        c.arg(value.to_string());
    }
    let out = bounded(c, None, 2000)?;
    if op == "write" {
        Ok(json!({"outcome":"write_api_success","verified":false,"value":value}))
    } else {
        let s = String::from_utf8_lossy(&out);
        let value = s
            .trim()
            .parse::<u16>()
            .map_err(|_| Error::new("ddc_unavailable"))?;
        Ok(json!({"value":value}))
    }
}

pub fn startup(store: &crate::store::Store, component: &str, enable: bool) -> Result<Value> {
    let exe = sibling(if component == "gui" {
        "elarydil-kvm-gui"
    } else {
        "elarydil-kvm-agent"
    })?;
    let uid = format!("com.elarydil.kvm.{component}");
    #[cfg(target_os = "linux")]
    {
        let dirs = directories::BaseDirs::new().ok_or_else(|| Error::new("io"))?;
        let path = dirs.config_dir().join("systemd/user");
        fs::create_dir_all(&path)?;
        let file = path.join(format!("elarydil-kvm-{component}.service"));
        if enable {
            let escaped = exe
                .to_string_lossy()
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('%', "%%");
            fs::write(&file,format!("[Unit]\nDescription=Elarydil KVM {component}\nAfter=graphical-session.target\n[Service]\nExecStart=\"{escaped}\"\nRestart=on-failure\nRestartSec=10\nStartLimitBurst=3\n[Install]\nWantedBy=default.target\n"))?;
            if !Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .status()?
                .success()
                || !Command::new("systemctl")
                    .args([
                        "--user",
                        "enable",
                        &format!("elarydil-kvm-{component}.service"),
                    ])
                    .status()?
                    .success()
            {
                return Err(Error::new("io").detail("systemctl --user"));
            }
        } else {
            let _ = Command::new("systemctl")
                .args([
                    "--user",
                    "disable",
                    &format!("elarydil-kvm-{component}.service"),
                ])
                .status();
            if file.exists() {
                fs::remove_file(file)?;
            }
            let _ = Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .status();
        }
    }
    #[cfg(target_os = "macos")]
    {
        let home = directories::BaseDirs::new().ok_or_else(|| Error::new("io"))?;
        let dir = home.home_dir().join("Library/LaunchAgents");
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{uid}.plist"));
        let _ = Command::new("launchctl").arg("unload").arg(&path).status();
        if enable {
            let escape = |s: &str| {
                s.replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;")
            };
            fs::write(&path,format!("<?xml version=\"1.0\"?><!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\"><plist version=\"1.0\"><dict><key>Label</key><string>{uid}</string><key>ProgramArguments</key><array><string>{}</string></array><key>RunAtLoad</key><true/><key>KeepAlive</key><dict><key>SuccessfulExit</key><false/></dict><key>ThrottleInterval</key><integer>10</integer></dict></plist>",escape(&exe.to_string_lossy())))?;
            if !Command::new("launchctl")
                .arg("load")
                .arg(&path)
                .status()?
                .success()
            {
                return Err(Error::new("permission_denied"));
            }
        } else if path.exists() {
            fs::remove_file(path)?;
        }
    }
    #[cfg(windows)]
    {
        let dir = std::env::var_os("APPDATA")
            .map(std::path::PathBuf::from)
            .ok_or_else(|| Error::new("io"))?
            .join("Microsoft/Windows/Start Menu/Programs/Startup");
        let old = dir.join(format!("elarydil-kvm {component}.cmd"));
        if old.exists() {
            fs::remove_file(old)?;
        }
        let file = dir.join(format!("Elarydil KVM {component}.cmd"));
        if enable {
            fs::write(
                file,
                format!(
                    "@start \"\" \"{}\"\r\n",
                    exe.to_string_lossy().replace('%', "%%")
                ),
            )?;
        } else if file.exists() {
            fs::remove_file(file)?;
        }
    }
    let _ = uid;
    store.mutate(|c| {
        if component == "gui" {
            c.app.launch_gui_at_login = enable
        } else {
            c.app.launch_agent_at_login = enable
        }
        Ok(())
    })?;
    Ok(
        json!({"component":component,"enabled":enable,"registration":"written","osApproval":"not_verified"}),
    )
}

/// The service uses this contract for native backends and deterministic virtual tests.
pub trait Hardware: Send + Sync {
    fn monitors(&self) -> Result<Vec<Monitor>>;
    fn ddc(&self, m: &Monitor, op: &str, vcp: u8, value: Option<u8>, timeout: u64)
        -> Result<Value>;
}
pub struct Native;
impl Hardware for Native {
    fn monitors(&self) -> Result<Vec<Monitor>> {
        monitors()
    }
    fn ddc(
        &self,
        m: &Monitor,
        op: &str,
        vcp: u8,
        value: Option<u8>,
        timeout: u64,
    ) -> Result<Value> {
        ddc(m, op, vcp, value, timeout)
    }
}

#[cfg(windows)]
pub fn current_sid() -> Result<String> {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, LocalFree},
        Security::Authorization::ConvertSidToStringSidW,
        Security::{GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER},
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    unsafe {
        let mut token = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(Error::new("permission_denied"));
        }
        let mut length = 0;
        GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut length);
        let mut data = vec![0u64; (length as usize).div_ceil(8)];
        if GetTokenInformation(
            token,
            TokenUser,
            data.as_mut_ptr().cast(),
            length,
            &mut length,
        ) == 0
        {
            CloseHandle(token);
            return Err(Error::new("permission_denied"));
        }
        CloseHandle(token);
        let user = &*(data.as_ptr().cast::<TOKEN_USER>());
        let mut sid = std::ptr::null_mut();
        if ConvertSidToStringSidW(user.User.Sid, &mut sid) == 0 {
            return Err(Error::new("permission_denied"));
        }
        let value = widestring::U16CStr::from_ptr_str(sid).to_string_lossy();
        LocalFree(sid.cast());
        Ok(value)
    }
}
#[cfg(windows)]
pub fn secure_directory(path: &std::path::Path) -> Result<()> {
    use interprocess::os::windows::security_descriptor::{
        AsSecurityDescriptorExt, SecurityDescriptor,
    };
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::{
            Authorization::{GetNamedSecurityInfoW, SetNamedSecurityInfoW, SE_FILE_OBJECT},
            GetAce, GetSecurityDescriptorControl, ACE_HEADER, DACL_SECURITY_INFORMATION,
            PROTECTED_DACL_SECURITY_INFORMATION, SE_DACL_PROTECTED,
        },
    };
    let text = widestring::U16CString::from_str(format!("D:P(A;OICI;FA;;;{})", current_sid()?))
        .map_err(|_| Error::new("unsafe_path"))?;
    let sd = SecurityDescriptor::deserialize(&text)?;
    let dacl = sd.dacl()?.ok_or_else(|| Error::new("permission_denied"))?.0;
    let mut name =
        widestring::U16CString::from_os_str(path).map_err(|_| Error::new("unsafe_path"))?;
    // Reapplying inheritable ACLs on every CLI connection opens child files
    // during propagation and can prevent the agent's atomic config replacement.
    // Preserve an already correct protected DACL; repair any different DACL.
    unsafe {
        let mut current_dacl = std::ptr::null_mut();
        let mut current_sd = std::ptr::null_mut();
        let result = GetNamedSecurityInfoW(
            name.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut current_dacl,
            std::ptr::null_mut(),
            &mut current_sd,
        );
        if result != 0 {
            return Err(Error::new("permission_denied")
                .detail(result)
                .context("directory.acl.read"));
        }
        let mut control = 0;
        let mut revision = 0;
        let mut current_ace = std::ptr::null_mut();
        let mut expected_ace = std::ptr::null_mut();
        let protected = GetSecurityDescriptorControl(current_sd, &mut control, &mut revision) != 0
            && control & SE_DACL_PROTECTED != 0;
        let equal = protected
            && !current_dacl.is_null()
            && (*current_dacl).AceCount == 1
            && GetAce(current_dacl, 0, &mut current_ace) != 0
            && GetAce(dacl.cast_mut(), 0, &mut expected_ace) != 0
            && {
                let current_size = (*current_ace.cast::<ACE_HEADER>()).AceSize as usize;
                let expected_size = (*expected_ace.cast::<ACE_HEADER>()).AceSize as usize;
                current_size == expected_size
                    && std::slice::from_raw_parts(current_ace.cast::<u8>(), current_size)
                        == std::slice::from_raw_parts(expected_ace.cast::<u8>(), expected_size)
            };
        LocalFree(current_sd);
        if equal {
            return Ok(());
        }
    }
    let result = unsafe {
        SetNamedSecurityInfoW(
            name.as_mut_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            dacl as *mut _,
            std::ptr::null_mut(),
        )
    };
    if result != 0 {
        return Err(Error::new("permission_denied").detail(result));
    }
    Ok(())
}
