use std::path::PathBuf;
use std::env;
use std::io;
use std::fs;
use serde_json::json;
use std::path::Path;

// TODO: should probably put the native messaging host manifest in it's own folder

pub const CLIENT_BINARY: &[u8] = include_bytes!(env!("CLIENT_BINARY_PATH"));

const NAME: &str = "yomitan_api";

#[derive(Clone, Copy)]
pub enum Browser {
    Firefox,
    Chrome,
    #[cfg(not(target_os = "windows"))]
    Chromium,
    Brave,
    #[cfg(target_os = "windows")]
    Edge,
}

impl Browser {
    pub const VALUES: &[Self] = &[
        Self::Firefox,
        Self::Chrome,
        #[cfg(not(target_os = "windows"))]
        Self::Chromium,
        Self::Brave,
        #[cfg(target_os = "windows")]
        Self::Edge,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Firefox => "firefox",
            Self::Chrome => "chrome",
            #[cfg(not(target_os = "windows"))]
            Self::Chromium => "chromium",
            Self::Brave => "brave",
            #[cfg(target_os = "windows")]
            Self::Edge => "edge",
        }
    }

    fn manifest_key(&self) -> &'static str {
        match self {
            Self::Firefox => "allowed_extensions",
            _ => "allowed_origins",
        }
    }

    pub fn extension_id(&self) -> &'static str {
        match self {
            Self::Firefox => "{6b733b82-9261-47ee-a595-2dda294a4d08}",
            _ => "chrome-extension://likgccmbimhjbgkjambclfkhldnlhbnn/",
        }
    }

    #[cfg(target_os = "linux")]
    fn install_path(&self) -> PathBuf {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .expect("Could not find home directory");

        home.join(match self {
            Self::Firefox => ".mozilla/native-messaging-hosts/",
            Self::Chrome => ".config/google-chrome/NativeMessagingHosts/",
            Self::Chromium => ".config/chromium/NativeMessagingHosts/",
            Self::Brave => ".config/BraveSoftware/Brave-Browser/NativeMessagingHosts/",
        })
    }

    #[cfg(target_os = "macos")]
    fn install_path(&self) -> PathBuf {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .expect("Could not find home directory");

        home.join(match self {
            Self::Firefox => "Library/Application Support/Mozilla/NativeMessagingHosts/",
            Self::Chrome => "Library/Application Support/Google/Chrome/NativeMessagingHosts/",
            Self::Chromium => "Library/Application Support/Chromium/NativeMessagingHosts/",
            Self::Brave => "Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts/",
        })
    }

    #[cfg(target_os = "windows")]
    fn install_path(&self) -> PathBuf {
        let local_app_data = std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .expect("Could not find LOCALAPPDATA directory");
        
        local_app_data.join("yomitan-api-rs")
    }

    #[cfg(target_os = "windows")]
    fn registry_path(&self) -> &'static str {
        match self {
            Self::Firefox => "SOFTWARE\\Mozilla\\NativeMessagingHosts\\yomitan_api",
            Self::Chrome => "SOFTWARE\\Google\\Chrome\\NativeMessagingHosts\\yomitan_api",
            Self::Brave => "SOFTWARE\\BraveSoftware\\Brave-Browser\\NativeMessagingHosts\\yomitan_api",
            Self::Edge => "SOFTWARE\\Microsoft\\Edge\\NativeMessagingHosts\\yomitan_api",
        }
    }

    pub fn install_api(self, extension_ids: &[&str]) -> io::Result<()> {
        let client_path = install_client_binary(self)?;
        let _manifest_path = install_manifest(self, &client_path, extension_ids)?;
        #[cfg(target_os = "windows")]
        install_registry(self, &_manifest_path)?;

        Ok(())
    }
}

fn install_client_binary(browser: Browser) -> io::Result<PathBuf> {
    let install_path = browser.install_path();
    fs::create_dir_all(&install_path)?;
    let binary_path = install_path.join("yomitan-api-rs-client");
    fs::write(&binary_path, CLIENT_BINARY)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)?;
    }

    Ok(binary_path)
}

#[cfg(target_os = "windows")]
fn install_registry(browser: Browser, manifest_path: &Path) -> io::Result<()> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    let (key, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(browser.registry_path())?;
    key.set_value("", &manifest_path.to_str().unwrap())?;

    Ok(())
}

fn install_manifest(browser: Browser, client_path: &Path, extension_ids: &[&str]) -> io::Result<PathBuf> {
    let manifest = serde_json::to_string_pretty(&json!({
        "name": NAME,
        "description": "Yomitan API (RS)",
        "type": "stdio",
        "path": client_path.to_str().unwrap(),
        browser.manifest_key(): extension_ids,
    })).unwrap();

    let install_path = browser.install_path();
    fs::create_dir_all(&install_path)?;
    let manifest_path = install_path.join(format!("{}.json", NAME));
    fs::write(&manifest_path, manifest)?;

    Ok(manifest_path)
}