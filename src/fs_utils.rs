use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use crate::types::{DiskInfo, FileEntry};


pub fn home_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string()))
    }
    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".to_string()))
    }
}


pub fn shortcuts_list(home: &Path) -> Vec<PathBuf> {
    let candidates = vec![
        home.to_path_buf(),
        home.join("Desktop"),
        home.join("Documents"),
        home.join("Downloads"),
        home.join("Pictures"),
        home.join("Music"),
        home.join("Videos"),
    ];
    candidates.into_iter().filter(|p| p.exists()).collect()
}


pub fn read_dir_entries(path: &Path, show_hidden: bool, query: &str) -> Vec<FileEntry> {
    let mut entries: Vec<FileEntry> = fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if !show_hidden && name.starts_with('.') {
                return None;
            }
            if !query.is_empty() && !name.to_lowercase().contains(&query.to_lowercase()) {
                return None;
            }
            let meta = e.metadata().ok()?;
            let is_dir = meta.is_dir();
            let size = if is_dir { 0 } else { meta.len() };
            let extension = if is_dir {
                String::new()
            } else {
                Path::new(&name)
                    .extension()
                    .and_then(|x| x.to_str())
                    .unwrap_or("")
                    .to_lowercase()
            };
            Some(FileEntry { name, is_dir, size, extension })
        })
        .collect();


    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    entries
}

pub fn detect_disks() -> Vec<DiskInfo> {
    let mut disks = vec![];

    #[cfg(target_os = "windows")]
    {
        for letter in b'A'..=b'Z' {
            let path = format!("{}:\\", letter as char);
            if Path::new(&path).exists() {
                disks.push(DiskInfo {
                    name: path.clone(),
                    label: format!(" {}", path),
                });
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        disks.push(DiskInfo {
            name: "/".into(),
            label: " / (root)".into(),
        });
        for base in ["/mnt", "/media"] {
            if let Ok(rd) = fs::read_dir(base) {
                for entry in rd.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        let label = format!(
                            " {}",
                            path.file_name().unwrap_or_default().to_string_lossy()
                        );
                        disks.push(DiskInfo {
                            name: path.to_string_lossy().into(),
                            label,
                        });
                    }
                }
            }
        }
    }

    disks
}

pub fn dir_preview(path: &Path) -> String {
    let mut lines = vec![format!(" {}\n", path.display())];
    let entries: Vec<_> = fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .take(30)
        .collect();
    let total = entries.len();
    for e in &entries {
        let name = e.file_name().to_string_lossy().to_string();
        let is_dir = e.metadata().map(|m| m.is_dir()).unwrap_or(false);
        let icon = if is_dir { "" } else { "" };
        lines.push(format!("{} {}", icon, name));
    }
    if total == 30 {
        lines.push("... (more files)".into());
    }
    lines.push(format!("\nTotal: {} elements", total));
    lines.join("\n")
}

pub fn file_preview(path: &Path, ext: &str) -> String {
    let image_exts = ["png", "jpg", "jpeg", "gif", "bmp", "svg", "webp", "ico"];
    if image_exts.contains(&ext) {
        return format!("󰋩  Images\n\n{}", path.display());
    }
    let binary_exts = [
        "exe", "bin", "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "pdf",
        "mp3", "mp4", "mkv", "avi", "flac", "ogg", "wav", "class",
    ];
    if binary_exts.contains(&ext) {
        let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        return format!(" bin file\n\nsize: {}", format_size(size));
    }
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => return format!("❌ Error:\n{}", e),
    };
    let mut buf = vec![0u8; 8192];
    let n = file.read(&mut buf).unwrap_or(0);
    let slice = &buf[..n];
    if slice.iter().any(|&b| b == 0) {
        return " Bin file".into();
    }
    String::from_utf8_lossy(slice).into_owned()
}

pub fn open_with_system(path: &Path, status: &mut String) {
    let result = {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", &path.to_string_lossy()])
                .spawn()
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            std::process::Command::new("xdg-open").arg(path).spawn()
        }
    };
    match result {
        Ok(_) => {
            *status = format!(
                "Open: {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            )
        }
        Err(e) => *status = format!("False opened: {}", e),
    }
}

pub fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / 1024.0 / 1024.0)
    } else {
        format!("{:.1} GB", size as f64 / 1024.0 / 1024.0 / 1024.0)
    }
}
