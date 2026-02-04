use std::path::PathBuf;
use std::env;
use std::io;
use std::io::Write;
use std::fs;
use serde_json::json;
use std::path::Path;

const NAME: &str = "yomitan_api";

#[derive(Clone, Copy)]
enum Browser {
    Firefox,
    Chrome,
    #[cfg(not(target_os = "windows"))]
    Chromium,
    Brave,
    #[cfg(target_os = "windows")]
    Edge,
}

impl Browser {
    const VALUES: &[Self] = &[
        Self::Firefox,
        Self::Chrome,
        #[cfg(not(target_os = "windows"))]
        Self::Chromium,
        Self::Brave,
        #[cfg(target_os = "windows")]
        Self::Edge,
    ];

    fn name(&self) -> &'static str {
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

    fn extension_id(&self) -> &'static str {
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
}

fn prompt(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn install_client_binary(browser: Browser) -> io::Result<PathBuf> {
    let dir = env::current_exe()?.parent().unwrap().to_path_buf();

    #[cfg(target_os = "windows")]
    let client_path = dir.join("client.exe");
    #[cfg(not(target_os = "windows"))]
    let client_path = dir.join("client");

    let install_path = browser.install_path();
    fs::create_dir_all(&install_path)?;
    let binary_path = install_path.join("yomitan-api-rs-client");
    fs::copy(&client_path, &binary_path)?;

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

// TODO: support multiple extension ids
// TODO: should open in terminal on macos?
// TODO: on windows, make sure window stays open after error
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("yomitan-api-rs installer {}", env!("CARGO_PKG_VERSION"));

    println!("Select browser:");
    for (i, browser) in Browser::VALUES.iter().enumerate() {
        println!("\t{}: {}", i + 1, browser.name());
    }
    println!("");
    println!("Don't see your browser or the installation doesn't appear to work? Open an issue at https://github.com/aura-ix/yomitan-api-rs");
    println!("");
    
    let browser = loop {
        if let Ok(choice) = prompt("Choice (enter corresponding number): ")?.parse::<usize>() && choice >= 1 && choice <= Browser::VALUES.len() {
            break Browser::VALUES[choice - 1];
        } else {
            println!("Invalid input, must be a number 1 - {}.", Browser::VALUES.len());
            continue
        }
    };

    println!("");
    println!("If you are using a development version of yomitan, enter your extension ID now. Otherwise, hit enter to skip this step.");
    println!("");

    let choice = prompt("Extension ID (enter to use default): ")?;
    let extension_id = if choice.len() == 0 {
        browser.extension_id()
    } else {
        &choice
    };

    let client_path = install_client_binary(browser)?;
    let _manifest_path = install_manifest(browser, &client_path, &[extension_id])?;
    #[cfg(target_os = "windows")]
    install_registry(browser, &_manifest_path)?;


    let _ = prompt("Installation complete. Hit enter to exit.");
    Ok(())
}