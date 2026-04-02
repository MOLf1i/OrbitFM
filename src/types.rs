#[derive(Debug, Clone, PartialEq)]
pub enum PanelFocus {
    Shortcuts,
    FileList,
    Preview,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub extension: String,
}

#[derive(Debug)]
pub struct DiskInfo {
    pub name: String,
    pub label: String,
}
