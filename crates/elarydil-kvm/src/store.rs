use crate::{model::Config, Error, Result};
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
};
#[derive(Clone)]
pub struct Store {
    pub root: PathBuf,
}
impl Store {
    pub fn open(root: Option<PathBuf>) -> Result<Self> {
        let root = root
            .or_else(|| std::env::var_os("ELARYDIL_KVM_HOME").map(PathBuf::from))
            .unwrap_or_else(|| {
                directories::ProjectDirs::from("", "", "elarydil-kvm")
                    .expect("user directory")
                    .config_dir()
                    .to_path_buf()
            });
        if root.exists() && fs::symlink_metadata(&root)?.file_type().is_symlink() {
            return Err(Error::new("unsafe_path"));
        }
        fs::create_dir_all(&root)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            let m = fs::metadata(&root)?;
            if m.uid() != unsafe { libc::geteuid() } {
                return Err(Error::new("permission_denied"));
            }
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
        }
        #[cfg(windows)]
        crate::platform::secure_directory(&root)?;
        Ok(Self { root })
    }
    pub fn path(&self) -> PathBuf {
        self.root.join("config.json")
    }
    pub fn load(&self) -> Result<Config> {
        let lock = self.config_lock()?;
        FileExt::lock_shared(&lock)?;
        self.load_unlocked()
    }
    fn load_unlocked(&self) -> Result<Config> {
        if !self.path().exists() {
            return Ok(Config::default());
        }
        let mut c: Config = serde_json::from_slice(
            &fs::read(self.path()).map_err(|e| Error::from(e).context("config.read"))?,
        )
        .map_err(|e| {
            Error::new("config_unreadable").detail(format!("{}: {e}", self.path().display()))
        })?;
        let mut active = false;
        for p in &mut c.profiles {
            if p.enabled {
                if active {
                    p.enabled = false;
                } else {
                    active = true;
                }
            }
        }
        c.validate()?;
        Ok(c)
    }
    fn config_lock(&self) -> Result<File> {
        Ok(OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.root.join("config.lock"))
            .map_err(|e| Error::from(e).context("config.lock.open"))?)
    }
    pub fn mutate<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Config) -> Result<T>,
    {
        let lock = self.config_lock()?;
        lock.lock_exclusive()
            .map_err(|e| Error::from(e).context("config.lock.exclusive"))?;
        let mut c = self.load_unlocked()?;
        let out = f(&mut c)?;
        c.validate()?;
        self.save(&c)?;
        Ok(out)
    }
    pub fn replace(&self, c: Config) -> Result<()> {
        self.mutate(|v| {
            *v = c;
            Ok(())
        })
    }
    fn save(&self, c: &Config) -> Result<()> {
        if self.path().exists() {
            fs::copy(self.path(), self.root.join("config.json.bak"))
                .map_err(|e| Error::from(e).context("config.backup"))?;
        }
        self.atomic("config.json", &serde_json::to_vec_pretty(c)?)
    }
    pub fn atomic(&self, name: &str, data: &[u8]) -> Result<()> {
        let tmp = self.root.join(format!(".{}.tmp", uuid::Uuid::new_v4()));
        let mut opts = OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut f = opts.open(&tmp)?;
        f.write_all(data)?;
        f.sync_all()?;
        drop(f);
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::Foundation::{
                ERROR_ACCESS_DENIED, ERROR_LOCK_VIOLATION, ERROR_SHARING_VIOLATION,
            };
            use windows_sys::Win32::Storage::FileSystem::{
                MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
            };
            let src: Vec<u16> = tmp.as_os_str().encode_wide().chain(Some(0)).collect();
            let dest: Vec<u16> = self
                .root
                .join(name)
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
            let mut delay = 5;
            loop {
                if unsafe {
                    MoveFileExW(
                        src.as_ptr(),
                        dest.as_ptr(),
                        MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                    )
                } != 0
                {
                    break;
                }
                let error = std::io::Error::last_os_error();
                let transient = matches!(
                    error.raw_os_error().map(|n| n as u32),
                    Some(ERROR_ACCESS_DENIED | ERROR_LOCK_VIOLATION | ERROR_SHARING_VIOLATION)
                );
                if !transient || std::time::Instant::now() >= deadline {
                    let _ = fs::remove_file(&tmp);
                    return Err(Error::from(error).context("atomic.replace"));
                }
                std::thread::sleep(std::time::Duration::from_millis(delay));
                delay = (delay * 2).min(50);
            }
        }
        #[cfg(not(windows))]
        fs::rename(tmp, self.root.join(name))?;
        #[cfg(unix)]
        File::open(&self.root)?.sync_all()?;
        Ok(())
    }
    pub fn instance_lock(&self) -> Result<File> {
        let f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.root.join("agent.lock"))?;
        f.try_lock_exclusive()
            .map_err(|_| Error::new("already_running"))?;
        Ok(f)
    }
}
