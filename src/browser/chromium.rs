use crate::browser::core::{BrowserEngine, BrowserError, PageContent};
use std::path::PathBuf;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process::Command;

pub struct ChromiumEngine {
    executable_path: Option<PathBuf>,
}

impl ChromiumEngine {
    pub fn new() -> Self {
        let mut engine = Self { executable_path: None };
        engine.executable_path = engine.detect_chromium_path();
        engine
    }

    pub fn is_available(&self) -> bool {
        #[cfg(target_os = "android")]
        {
            return false;
        }
        self.executable_path.is_some()
    }

    pub fn get_executable_path(&self) -> Option<PathBuf> {
        self.executable_path.clone()
    }

    pub fn set_manual_path(&mut self, path: PathBuf) {
        if path.exists() {
            self.executable_path = Some(path);
        }
    }

    pub fn get_cache_directory(&self) -> Option<PathBuf> {
        dirs_next::cache_dir().map(|mut d| {
            d.push("isearch");
            d.push("chromium");
            d
        })
    }

    pub fn get_guided_install_instructions(&self) -> String {
        let os = env::consts::OS;
        match os {
            "linux" => {
                "Guided installation for Linux:\n\
                 - Ubuntu/Debian: sudo apt update && sudo apt install -y chromium-browser\n\
                 - Fedora: sudo dnf install -y chromium\n\
                 - Arch Linux: sudo pacman -S chromium\n\
                 - Flatpak: flatpak install flathub org.chromium.Chromium\n\
                 - Snap: sudo snap install chromium".to_string()
            }
            "macos" => {
                "Guided installation for macOS:\n\
                 - Using Homebrew: brew install --cask google-chrome\n\
                 - Or download official Google Chrome dmg package from google.com/chrome".to_string()
            }
            "windows" => {
                "Guided installation for Windows:\n\
                 - Using winget: winget install Google.Chrome\n\
                 - Or download and run ChromeStandaloneSetup64.exe from google.com/chrome".to_string()
            }
            "android" => {
                "Android (Termux) detected:\n\
                 - Chromium Headless is not supported on Android Termux.\n\
                 - Please use the Native Rendering Engine (default) which is fully optimized!".to_string()
            }
            _ => "Please install Google Chrome or Chromium using your system's package manager.".to_string(),
        }
    }

    pub fn download_chromium<F>(&mut self, on_progress: F) -> Result<PathBuf, BrowserError>
    where
        F: Fn(f32, &str),
    {
        #[cfg(target_os = "android")]
        {
            return Err(BrowserError::UnsupportedPlatform("Chromium is not supported on Android Termux.".to_string()));
        }

        let os = env::consts::OS;
        let arch = env::consts::ARCH;

        // Determine URL for Google Chrome for Testing (125.0.6422.141)
        let download_url = match (os, arch) {
            ("linux", "x86_64") => "https://storage.googleapis.com/chrome-for-testing-public/125.0.6422.141/linux64/chrome-linux64.zip",
            ("macos", "x86_64") => "https://storage.googleapis.com/chrome-for-testing-public/125.0.6422.141/mac-x64/chrome-mac.zip",
            ("macos", "aarch64") => "https://storage.googleapis.com/chrome-for-testing-public/125.0.6422.141/mac-arm64/chrome-mac.zip",
            ("windows", "x86_64") => "https://storage.googleapis.com/chrome-for-testing-public/125.0.6422.141/win64/chrome-win64.zip",
            _ => return Err(BrowserError::UnsupportedPlatform(format!("No automated pre-built Chromium binary for {} {}", os, arch))),
        };

        let cache_dir = self.get_cache_directory().ok_or_else(|| BrowserError::IoError("Failed to get cache directory".to_string()))?;
        fs::create_dir_all(&cache_dir).map_err(|e| BrowserError::IoError(e.to_string()))?;

        let zip_path = cache_dir.join("chrome.zip");
        on_progress(0.0, "Initiating download...");

        // Download via ureq
        let agent = ureq::Agent::new_with_defaults();
        let response = agent.get(download_url).call()
            .map_err(|e| BrowserError::DownloadError(format!("Network download failed: {:?}", e)))?;

        let total_size = response.headers().get("content-length")
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(150_000_000); // approx size

        let mut body_obj = response.into_body();
        let mut reader = body_obj.as_reader();
        let mut file = fs::File::create(&zip_path).map_err(|e| BrowserError::IoError(e.to_string()))?;

        let mut buffer = vec![0; 16384];
        let mut downloaded: u64 = 0;

        loop {
            let bytes_read = reader.read(&mut buffer).map_err(|e| BrowserError::DownloadError(e.to_string()))?;
            if bytes_read == 0 {
                break;
            }
            file.write_all(&buffer[..bytes_read]).map_err(|e| BrowserError::IoError(e.to_string()))?;
            downloaded += bytes_read as u64;
            let progress = (downloaded as f32 / total_size as f32).min(1.0);
            on_progress(progress * 0.7, &format!("Downloading Chromium... {:.1}%", progress * 100.0));
        }

        on_progress(0.7, "Extracting zip archive...");
        // Extract ZIP
        let file = fs::File::open(&zip_path).map_err(|e| BrowserError::IoError(e.to_string()))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| BrowserError::DownloadError(format!("Failed to parse zip: {}", e)))?;
        let total_files = archive.len();

        for i in 0..total_files {
            let mut file = archive.by_index(i).map_err(|e| BrowserError::DownloadError(e.to_string()))?;
            let outpath = match file.enclosed_name() {
                Some(path) => cache_dir.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).map_err(|e| BrowserError::IoError(e.to_string()))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(|e| BrowserError::IoError(e.to_string()))?;
                    }
                }
                let mut outfile = fs::File::create(&outpath).map_err(|e| BrowserError::IoError(e.to_string()))?;
                io::copy(&mut file, &mut outfile).map_err(|e| BrowserError::IoError(e.to_string()))?;
            }

            // Set executable permission on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if outpath.file_name().and_then(|n| n.to_str()) == Some("chrome") || outpath.to_string_lossy().contains("Google Chrome.app/Contents/MacOS/") {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(0o755)).unwrap_or(());
                }
            }

            let progress = 0.7 + (i as f32 / total_files as f32) * 0.3;
            on_progress(progress, "Extracting files...");
        }

        // Clean up ZIP
        let _ = fs::remove_file(&zip_path);

        on_progress(1.0, "Chromium successfully downloaded and cached!");

        // Rediscover the path
        if let Some(found_path) = self.detect_chromium_path() {
            self.executable_path = Some(found_path.clone());
            Ok(found_path)
        } else {
            Err(BrowserError::DownloadError("Failed to locate Chromium after extraction".to_string()))
        }
    }

    pub fn detect_chromium_path(&self) -> Option<PathBuf> {
        // 1. Check env var
        if let Ok(path_str) = env::var("CHROME_PATH") {
            let p = PathBuf::from(path_str);
            if p.exists() {
                return Some(p);
            }
        }
        if let Ok(path_str) = env::var("CHROMIUM_PATH") {
            let p = PathBuf::from(path_str);
            if p.exists() {
                return Some(p);
            }
        }

        // 2. Check cached portable download path (we will specify this under ~/.cache/isearch/chromium/)
        if let Some(mut cache_dir) = dirs_next::cache_dir() {
            cache_dir.push("isearch");
            cache_dir.push("chromium");

            // Under Google Chrome for Testing, macOS output structure is chrome-mac/Google Chrome.app/...
            // Linux structure is chrome-linux64/chrome
            // Windows structure is chrome-win64/chrome.exe
            #[cfg(target_os = "windows")]
            let relative_paths = ["chrome-win64/chrome.exe", "chrome.exe"];
            #[cfg(target_os = "macos")]
            let relative_paths = ["chrome-mac/Google Chrome.app/Contents/MacOS/Google Chrome", "Google Chrome.app/Contents/MacOS/Google Chrome"];
            #[cfg(target_os = "linux")]
            let relative_paths = ["chrome-linux64/chrome", "chrome/chrome", "chrome-linux64/google-chrome"];

            for rel in &relative_paths {
                let test_path = cache_dir.join(rel);
                if test_path.exists() {
                    return Some(test_path);
                }
            }
        }

        // 3. Platform specific standard search paths
        #[cfg(target_os = "linux")]
        {
            let linux_paths = [
                "/usr/bin/google-chrome",
                "/usr/bin/chromium",
                "/usr/bin/chromium-browser",
                "/snap/bin/chromium",
                "/var/lib/flatpak/exports/bin/org.chromium.Chromium",
            ];
            for path in linux_paths {
                let p = PathBuf::from(path);
                if p.exists() {
                    return Some(p);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let mac_paths = [
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                "/Applications/Chromium.app/Contents/MacOS/Chromium",
                "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
            ];
            for path in mac_paths {
                let p = PathBuf::from(path);
                if p.exists() {
                    return Some(p);
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            let program_files = env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
            let program_files_x86 = env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
            let local_app_data = env::var("LocalAppData").unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Local".to_string());

            let win_paths = [
                format!("{}\\Google\\Chrome\\Application\\chrome.exe", program_files),
                format!("{}\\Google\\Chrome\\Application\\chrome.exe", program_files_x86),
                format!("{}\\Google\\Chrome\\Application\\chrome.exe", local_app_data),
                format!("{}\\Chromium\\Application\\chrome.exe", program_files),
                format!("{}\\Chromium\\Application\\chrome.exe", program_files_x86),
            ];
            for path in win_paths {
                let p = PathBuf::from(path);
                if p.exists() {
                    return Some(p);
                }
            }
        }

        None
    }
}

impl BrowserEngine for ChromiumEngine {
    fn navigate(&mut self, url: &str) -> Result<PageContent, BrowserError> {
        let exec_path = self.executable_path.as_ref()
            .ok_or_else(|| BrowserError::ChromiumNotAvailable("No Chrome or Chromium executable found. Please download first.".to_string()))?;

        // Format clean URL
        let mut clean_url = url.to_string();
        if !clean_url.starts_with("http://") && !clean_url.starts_with("https://") {
            clean_url = format!("https://{}", clean_url);
        }

        // Run Chrome Headless to fetch DOM after modern JS execution
        let output = Command::new(exec_path)
            .arg("--headless=new")
            .arg("--disable-gpu")
            .arg("--dump-dom")
            .arg(&clean_url)
            .output()
            .map_err(|e| BrowserError::IoError(format!("Failed to execute Chromium: {}", e)))?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(BrowserError::NetworkError(format!("Chromium failed to render page: {}", err_msg)));
        }

        let body = String::from_utf8_lossy(&output.stdout).to_string();

        let title = if let Ok(dom) = tl::parse(&body, tl::ParserOptions::default()) {
            let mut t_str = clean_url.clone();
            for node in dom.nodes() {
                if let tl::Node::Tag(tag) = node {
                    if tag.name().as_utf8_str() == "title" {
                        t_str = tag.inner_text(dom.parser()).to_string();
                        break;
                    }
                }
            }
            t_str
        } else {
            clean_url.clone()
        };

        let parsed = crate::browser::native::NativeEngine::parse_html(&body);
        Ok(PageContent::Html { title, raw_html: body, parsed_nodes: parsed })
    }

    fn search(&mut self, query: &str) -> Result<PageContent, BrowserError> {
        let url = format!("https://www.google.com/search?q={}", percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC));
        self.navigate(&url)
    }

    fn capture_screenshot(&mut self, url: &str, width: u32, height: u32) -> Result<Vec<u8>, BrowserError> {
        let exec_path = self.executable_path.as_ref()
            .ok_or_else(|| BrowserError::ChromiumNotAvailable("No Chrome or Chromium executable found. Please download first.".to_string()))?;

        // Format clean URL
        let mut clean_url = url.to_string();
        if !clean_url.starts_with("http://") && !clean_url.starts_with("https://") {
            clean_url = format!("https://{}", clean_url);
        }

        let temp_screenshot = env::temp_dir().join(format!("isearch_screenshot_{}.png", rand_string()));

        // Run Chrome to capture screenshot
        let output = Command::new(exec_path)
            .arg("--headless=new")
            .arg("--disable-gpu")
            .arg(format!("--screenshot={}", temp_screenshot.display()))
            .arg(format!("--window-size={},{}", width, height))
            .arg(&clean_url)
            .output()
            .map_err(|e| BrowserError::IoError(format!("Failed to execute Chromium: {}", e)))?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(BrowserError::NetworkError(format!("Chromium screenshot failed: {}", err_msg)));
        }

        if temp_screenshot.exists() {
            let bytes = fs::read(&temp_screenshot).map_err(|e| BrowserError::IoError(e.to_string()))?;
            let _ = fs::remove_file(&temp_screenshot);
            Ok(bytes)
        } else {
            Err(BrowserError::IoError("Screenshot was not generated by Chromium".to_string()))
        }
    }
}

fn rand_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("{:x}", nanos)
}

mod dirs_next {
    use std::path::PathBuf;
    pub fn cache_dir() -> Option<PathBuf> {
        std::env::var("HOME").map(|h| PathBuf::from(h).join(".cache")).ok()
    }
}
