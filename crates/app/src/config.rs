// SPDX-License-Identifier: AGPL-3.0-only
//! Private atomic configuration. Never format credentials or untrusted upstream errors.
use base64::Engine;
use psremote::rp_session::SessionCreds;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::Ipv4Addr,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub type Result<T> = std::result::Result<T, &'static str>;
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Preferences {
    pub version: u32,
    pub host: String,
    pub codec: u8,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32,
    pub deadzone: i16,
    pub jitter_ms: u64,
    pub reconnect: bool,
    pub wake: bool,
    pub usb_effects: bool,
    pub usb_audio: bool,
    pub haptics_device: String,
    pub haptics_strength: u8,
    pub trigger_strength: u8,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            version: 1,
            host: String::new(),
            codec: 0,
            height: 1080,
            fps: 60,
            bitrate: 15000,
            deadzone: 2000,
            jitter_ms: 20,
            reconnect: true,
            wake: true,
            usb_effects: false,
            usb_audio: false,
            haptics_device: String::new(),
            haptics_strength: 1,
            trigger_strength: 1,
        }
    }
}
impl Preferences {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || self.codec > 2
            || ![720, 1080].contains(&self.height)
            || ![30, 60].contains(&self.fps)
            || !(2000..=50000).contains(&self.bitrate)
            || !(0..=16000).contains(&self.deadzone)
            || !(10..=80).contains(&self.jitter_ms)
            || self.haptics_strength > 3
            || self.trigger_strength > 3
            || self.haptics_device.len() > 256
        {
            return Err("Invalid preferences");
        }
        if !self.host.is_empty() {
            validate_host(&self.host)?;
        }
        Ok(())
    }
}
pub fn validate_host(host: &str) -> Result<()> {
    let ip: Ipv4Addr = host.parse().map_err(|_| "Enter a LAN IPv4 address")?;
    if !(ip.is_private() || ip.is_link_local()) || ip.is_broadcast() {
        return Err("Use a private LAN address");
    }
    Ok(())
}
pub fn validate_account(account: &str) -> Result<()> {
    let b = base64::engine::general_purpose::STANDARD
        .decode(account.trim())
        .map_err(|_| "Account ID must be base64 of 8 bytes")?;
    if b.len() != 8 {
        return Err("Account ID must be base64 of 8 bytes");
    }
    Ok(())
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pairing {
    pub version: u32,
    pub host: String,
    pub regist_key: String,
    pub rp_key: String,
}
impl Pairing {
    pub fn creds(&self) -> Result<SessionCreds> {
        validate_host(&self.host)?;
        if self.version != 1
            || [&self.regist_key, &self.rp_key]
                .iter()
                .any(|s| s.len() != 32 || !s.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err("Invalid saved pairing");
        }
        SessionCreds::from_hex(&self.regist_key, &self.rp_key).map_err(|_| "Invalid saved pairing")
    }
}
pub struct Store {
    dir: PathBuf,
}
impl Store {
    pub fn open(dir: PathBuf) -> Result<Self> {
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|_| "Cannot create private data directory")?;
        }
        let m = fs::symlink_metadata(&dir).map_err(|_| "Cannot inspect data directory")?;
        if !m.is_dir() || m.file_type().is_symlink() || m.uid() != unsafe { libc::geteuid() } {
            eprintln!(
                "Storage rejected: directory={} symlink={} owner_uid={} app_uid={}",
                m.is_dir(),
                m.file_type().is_symlink(),
                m.uid(),
                unsafe { libc::geteuid() }
            );
            return Err("Unsafe data directory");
        }
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))
            .map_err(|_| "Cannot protect data directory")?;
        Ok(Self { dir })
    }
    fn read<T: serde::de::DeserializeOwned>(&self, name: &str) -> Result<Option<T>> {
        let path = self.dir.join(name);
        let f = match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&path)
        {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err("Cannot read private configuration"),
        };
        let m = f.metadata().map_err(|_| "Cannot inspect configuration")?;
        if !m.is_file()
            || m.uid() != unsafe { libc::geteuid() }
            || m.mode() & 0o077 != 0
            || m.len() > 16384
        {
            return Err("Unsafe configuration permissions or size");
        }
        let mut bytes = Vec::new();
        f.take(16385)
            .read_to_end(&mut bytes)
            .map_err(|_| "Cannot read configuration")?;
        if bytes.len() > 16384 {
            return Err("Configuration too large");
        }
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|_| "Invalid private configuration")
    }
    fn write<T: Serialize>(&self, name: &str, value: &T) -> Result<()> {
        let bytes = serde_json::to_vec(value).map_err(|_| "Cannot encode configuration")?;
        let path = self.dir.join(format!(
            ".{}-{}-{}",
            name,
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let result = (|| {
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
                .open(&path)
                .map_err(|_| "Cannot create private configuration")?;
            f.write_all(&bytes)
                .and_then(|_| f.sync_all())
                .map_err(|_| "Cannot write configuration")?;
            fs::rename(&path, self.dir.join(name))
                .map_err(|_| "Cannot atomically save configuration")?;
            File::open(&self.dir)
                .and_then(|f| f.sync_all())
                .map_err(|_| "Cannot sync data directory")?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(path);
        }
        result
    }
    pub fn preferences(&self) -> Result<Preferences> {
        let p = self
            .read::<Preferences>("settings.json")?
            .unwrap_or_default();
        p.validate()?;
        Ok(p)
    }
    pub fn pairing(&self) -> Result<Option<Pairing>> {
        let p = self.read::<Pairing>("pairing.json")?;
        if let Some(p) = &p {
            p.creds()?;
        }
        Ok(p)
    }
    pub fn save_preferences(&self, p: &Preferences) -> Result<()> {
        p.validate()?;
        self.write("settings.json", p)
    }
    pub fn save_pairing(&self, p: &Pairing) -> Result<()> {
        p.creds()?;
        self.write("pairing.json", p)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "psremote-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }
    #[test]
    fn blocks_external_hosts_and_bad_account() {
        for h in [
            "8.8.8.8",
            "sony.com",
            "127.0.0.1",
            "192.168.0.1:80",
            "192.168.0.1\r\n",
        ] {
            assert!(validate_host(h).is_err());
        }
        assert!(validate_host("192.168.1.20").is_ok());
        assert!(validate_account("AQIDBAUGBwg=").is_ok());
        assert!(validate_account("secret").is_err());
    }
    #[test]
    fn atomic_private_round_trip_and_bad_update_preserves_pairing() {
        let d = dir();
        let s = Store::open(d.clone()).unwrap();
        let p = Pairing {
            version: 1,
            host: "192.168.1.20".into(),
            regist_key: "00".repeat(16),
            rp_key: "01".repeat(16),
        };
        s.save_pairing(&p).unwrap();
        assert_eq!(
            fs::metadata(d.join("pairing.json")).unwrap().mode() & 0o777,
            0o600
        );
        let mut bad = p.clone();
        bad.rp_key = "é".repeat(16);
        assert!(s.save_pairing(&bad).is_err());
        assert_eq!(s.pairing().unwrap().unwrap().rp_key, p.rp_key);
        fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn rejects_symlink() {
        let d = dir();
        let s = Store::open(d.clone()).unwrap();
        std::os::unix::fs::symlink("/etc/passwd", d.join("pairing.json")).unwrap();
        assert!(s.pairing().is_err());
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn private_child_preserves_installation_directory_permissions() {
        let base = dir();
        fs::create_dir(&base).unwrap();
        fs::set_permissions(&base, fs::Permissions::from_mode(0o755)).unwrap();
        let private = base.join("private");
        let store = Store::open(private.clone()).unwrap();
        store.save_preferences(&Preferences::default()).unwrap();
        assert_eq!(fs::metadata(&base).unwrap().mode() & 0o777, 0o755);
        assert_eq!(fs::metadata(&private).unwrap().mode() & 0o777, 0o700);
        assert_eq!(
            fs::metadata(private.join("settings.json")).unwrap().mode() & 0o777,
            0o600
        );
        assert!(store.preferences().is_ok());
        fs::remove_dir_all(base).unwrap();
    }
}
