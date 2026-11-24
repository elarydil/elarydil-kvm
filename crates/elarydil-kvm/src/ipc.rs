use crate::{store::Store, Error, Result};
#[cfg(unix)]
use interprocess::local_socket::GenericFilePath;
#[cfg(windows)]
use interprocess::local_socket::GenericNamespaced;
use interprocess::local_socket::{prelude::*, ListenerOptions, Stream};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    io::{BufRead, BufReader, Read, Write},
    sync::mpsc,
    time::Duration,
};
pub const VERSION: u32 = 1;
#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub version: u32,
    pub id: String,
    pub token: String,
    pub command: String,
    #[serde(default)]
    pub payload: Value,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub version: u32,
    pub id: String,
    pub result: Option<Value>,
    pub error: Option<Error>,
}
impl Response {
    pub fn from_result(id: String, result: Result<Value>) -> Self {
        match result {
            Ok(v) => Self {
                version: VERSION,
                id,
                result: Some(v),
                error: None,
            },
            Err(e) => Self {
                version: VERSION,
                id,
                result: None,
                error: Some(e),
            },
        }
    }
}
fn name(store: &Store) -> Result<interprocess::local_socket::Name<'static>> {
    #[cfg(unix)]
    return Ok(store
        .root
        .join("agent.sock")
        .to_fs_name::<GenericFilePath>()?);
    #[cfg(windows)]
    return Ok(format!(
        "elarydil-kvm-{}",
        crate::model::hash(store.root.to_string_lossy().as_bytes())
    )
    .to_ns_name::<GenericNamespaced>()?);
}
pub fn call(store: &Store, command: &str, payload: Value) -> Result<Value> {
    let token = std::fs::read_to_string(store.root.join("token"))
        .map_err(|_| Error::new("agent_not_running"))?;
    let req = Request {
        version: VERSION,
        id: uuid::Uuid::new_v4().to_string(),
        token,
        command: command.into(),
        payload,
    };
    let endpoint = name(store)?;
    let (tx, rx) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let result = (|| {
            let mut stream =
                Stream::connect(endpoint).map_err(|_| Error::new("agent_not_running"))?;
            serde_json::to_writer(&mut stream, &req)?;
            stream.write_all(b"\n")?;
            let mut raw = String::new();
            let count = BufReader::new(stream).take(1048577).read_line(&mut raw)?;
            if count == 0 {
                return Err(Error::new("agent_not_running"));
            }
            if raw.len() > 1048576 {
                return Err(Error::new("invalid_input"));
            }
            let response: Response = serde_json::from_str(&raw)?;
            if response.id != req.id || response.version != VERSION {
                return Err(Error::new("protocol_version"));
            }
            if let Some(e) = response.error {
                return Err(e);
            }
            Ok(response.result.unwrap_or(Value::Null))
        })();
        let _ = tx.send(result);
    });
    rx.recv_timeout(Duration::from_secs(30))
        .map_err(|_| Error::new("timeout"))?
}
pub type Incoming = (Request, mpsc::SyncSender<Response>);
pub fn listen(store: Store, tx: mpsc::SyncSender<Incoming>) -> Result<()> {
    #[cfg(unix)]
    {
        let path = store.root.join("agent.sock");
        if path.exists() {
            if std::fs::symlink_metadata(&path)?.file_type().is_symlink() {
                return Err(Error::new("unsafe_path"));
            }
            std::fs::remove_file(path)?;
        }
    }
    let token = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    store.atomic("token", token.as_bytes())?;
    let options = ListenerOptions::new().name(name(&store)?);
    #[cfg(all(unix, not(target_os = "macos")))]
    let options = {
        use interprocess::os::unix::local_socket::ListenerOptionsExt;
        options.mode(0o600)
    };
    #[cfg(windows)]
    let options = {
        use interprocess::os::windows::{
            local_socket::ListenerOptionsExt, security_descriptor::SecurityDescriptor,
        };
        let sddl = widestring::U16CString::from_str(format!(
            "D:P(A;;GA;;;{})",
            crate::platform::current_sid()?
        ))
        .map_err(|_| Error::new("permission_denied"))?;
        options.security_descriptor(SecurityDescriptor::deserialize(&sddl)?)
    };
    let listener = options.create_sync()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // macOS cannot fchmod an unbound socket, which ListenerOptions::mode uses.
        // The owner-only store directory protects the path during bind + chmod;
        // no process-wide umask change is needed in this multithreaded process.
        std::fs::set_permissions(
            store.root.join("agent.sock"),
            std::fs::Permissions::from_mode(0o600),
        )?;
    }
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let token = token.clone();
            let tx = tx.clone();
            // Bound simultaneous requests: each authenticated local client gets a short-lived worker.
            std::thread::spawn(move || {
                let result = (|| -> Result<()> {
                    let mut reader = BufReader::new(stream);
                    let mut raw = String::new();
                    reader.by_ref().take(1048577).read_line(&mut raw)?;
                    if raw.len() > 1048576 {
                        return Err(Error::new("invalid_input"));
                    }
                    let req: Request = serde_json::from_str(&raw)?;
                    let id = req.id.clone();
                    let response = if req.version != VERSION {
                        Response::from_result(id, Err(Error::new("protocol_version")))
                    } else if req.token != token {
                        Response::from_result(id, Err(Error::new("authentication")))
                    } else {
                        let (reply, rx) = mpsc::sync_channel(1);
                        tx.send((req, reply))
                            .map_err(|_| Error::new("agent_not_running"))?;
                        rx.recv_timeout(Duration::from_secs(25))
                            .map_err(|_| Error::new("timeout"))?
                    };
                    serde_json::to_writer(reader.get_mut(), &response)?;
                    reader.get_mut().write_all(b"\n")?;
                    Ok(())
                })();
                let _ = result;
            });
        }
    });
    Ok(())
}
