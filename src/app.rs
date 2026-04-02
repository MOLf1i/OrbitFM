use std::{fs, path::PathBuf};

use ratatui::widgets::ListState;

use crate::{
    fs_utils::*,
    types::{DiskInfo, FileEntry, PanelFocus},
};

pub struct App {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    pub list_state: ListState,

    pub focus: PanelFocus,

    pub shortcuts: Vec<PathBuf>,
    pub shortcut_selected: usize,
    pub shortcut_state: ListState,

    pub disks: Vec<DiskInfo>,
    pub disk_selected: usize,

    pub preview_content: String,
    pub preview_scroll: u16,

    pub status_msg: String,
    pub search_mode: bool,
    pub search_query: String,
    pub show_hidden: bool,
}

impl App {
    pub fn new() -> Self {
        let home = home_dir();
        let mut app = App {
            current_path: home.clone(),
            entries: vec![],
            selected: 0,
            list_state: ListState::default(),
            focus: PanelFocus::FileList,
            shortcuts: shortcuts_list(&home),
            shortcut_selected: 0,
            shortcut_state: ListState::default(),
            disks: detect_disks(),
            disk_selected: 0,
            preview_content: String::new(),
            preview_scroll: 0,
            status_msg: String::from(
                "q:exit  enter:open  bs:back  tab:panel  .:hiden  /:search  D:delete  r:reload",
            ),
            search_mode: false,
            search_query: String::new(),
            show_hidden: false,
        };
        app.list_state.select(Some(0));
        app.shortcut_state.select(Some(0));
        app.reload_entries();
        app
    }

    pub fn reload_entries(&mut self) {
        self.entries = read_dir_entries(&self.current_path, self.show_hidden, &self.search_query);
        self.selected = self.selected.min(self.entries.len().saturating_sub(1));
        self.list_state
            .select(if self.entries.is_empty() { None } else { Some(self.selected) });
        self.update_preview();
    }

    pub fn update_preview(&mut self) {
        self.preview_scroll = 0;
        if let Some(entry) = self.entries.get(self.selected) {
            let path = self.current_path.join(&entry.name);
            self.preview_content = if entry.is_dir {
                dir_preview(&path)
            } else {
                file_preview(&path, &entry.extension)
            };
        } else {
            self.preview_content = String::new();
        }
    }

    pub fn open_selected(&mut self) {
        if let Some(entry) = self.entries.get(self.selected) {
            let path = self.current_path.join(&entry.name);
            if entry.is_dir {
                self.current_path = path;
                self.selected = 0;
                self.search_query.clear();
                self.reload_entries();
                self.status_msg = format!("📂 {}", self.current_path.display());
            } else {
                open_with_system(&path, &mut self.status_msg);
            }
        }
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.selected = 0;
            self.search_query.clear();
            self.reload_entries();
        }
    }

    pub fn move_down(&mut self) {
        if !self.entries.is_empty() {
            self.selected = (self.selected + 1).min(self.entries.len() - 1);
            self.list_state.select(Some(self.selected));
            self.update_preview();
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.list_state.select(Some(self.selected));
            self.update_preview();
        }
    }

    pub fn shortcut_down(&mut self) {
        let total = self.shortcuts.len() + self.disks.len();
        if total > 0 {
            self.shortcut_selected = (self.shortcut_selected + 1).min(total - 1);
            self.shortcut_state.select(Some(self.shortcut_selected));
        }
    }

    pub fn shortcut_up(&mut self) {
        if self.shortcut_selected > 0 {
            self.shortcut_selected -= 1;
            self.shortcut_state.select(Some(self.shortcut_selected));
        }
    }

    pub fn open_shortcut(&mut self) {
        let idx = self.shortcut_selected;
        let path = if idx < self.shortcuts.len() {
            Some(self.shortcuts[idx].clone())
        } else {
            let disk_idx = idx - self.shortcuts.len();
            self.disks.get(disk_idx).map(|d| PathBuf::from(&d.name))
        };
        if let Some(p) = path {
            if p.exists() {
                self.current_path = p;
                self.selected = 0;
                self.search_query.clear();
                self.reload_entries();
                self.focus = PanelFocus::FileList;
            }
        }
    }

    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.reload_entries();
        self.status_msg = format!(
            "Приховані файли: {}",
            if self.show_hidden { "показано" } else { "сховано" }
        );
    }

    pub fn delete_selected(&mut self) {
        if let Some(entry) = self.entries.get(self.selected) {
            let path = self.current_path.join(&entry.name);
            let result = if entry.is_dir {
                fs::remove_dir_all(&path)
            } else {
                fs::remove_file(&path)
            };
            match result {
                Ok(_) => {
                    self.status_msg = format!(" Deleted: {}", entry.name);
                    self.reload_entries();
                }
                Err(e) => {
                    self.status_msg = format!("❌ Error: {}", e);
                }
            }
        }
    }

    pub fn next_panel(&mut self) {
        self.focus = match self.focus {
            PanelFocus::Shortcuts => PanelFocus::FileList,
            PanelFocus::FileList => PanelFocus::Preview,
            PanelFocus::Preview => PanelFocus::Shortcuts,
        };
    }

    pub fn prev_panel(&mut self) {
        self.focus = match self.focus {
            PanelFocus::Shortcuts => PanelFocus::Preview,
            PanelFocus::FileList => PanelFocus::Shortcuts,
            PanelFocus::Preview => PanelFocus::FileList,
        };
    }
}
