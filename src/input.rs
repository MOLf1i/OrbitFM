use crossterm::event::{KeyCode, KeyModifiers};

use crate::{app::App, types::PanelFocus};

pub fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> bool {
    // Режим пошуку: символи йдуть у рядок запиту
    if app.search_mode {
        return handle_search_key(app, code);
    }

    match code {
        // Exit
        KeyCode::Char('q') => return true,
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => return true,

        //switch tabs
        KeyCode::Tab => app.next_panel(),
        KeyCode::BackTab => app.prev_panel(),

        //move down
        KeyCode::Down | KeyCode::Char('j') => match app.focus {
            PanelFocus::FileList => app.move_down(),
            PanelFocus::Shortcuts => app.shortcut_down(),
            PanelFocus::Preview => {
                app.preview_scroll = app.preview_scroll.saturating_add(3)
            }
        },

        // move up
        KeyCode::Up | KeyCode::Char('k') => match app.focus {
            PanelFocus::FileList => app.move_up(),
            PanelFocus::Shortcuts => app.shortcut_up(),
            PanelFocus::Preview => {
                app.preview_scroll = app.preview_scroll.saturating_sub(3)
            }
        },

        // page
        KeyCode::PageDown => match app.focus {
            PanelFocus::Preview => {
                app.preview_scroll = app.preview_scroll.saturating_add(20)
            }
            PanelFocus::FileList => {
                for _ in 0..10 {
                    app.move_down();
                }
            }
            _ => {}
        },
        KeyCode::PageUp => match app.focus {
            PanelFocus::Preview => {
                app.preview_scroll = app.preview_scroll.saturating_sub(20)
            }
            PanelFocus::FileList => {
                for _ in 0..10 {
                    app.move_up();
                }
            }
            _ => {}
        },

        //open
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => match app.focus {
            PanelFocus::FileList => app.open_selected(),
            PanelFocus::Shortcuts => app.open_shortcut(),
            PanelFocus::Preview => app.focus = PanelFocus::FileList,
        },

        // back
        KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') => match app.focus {
            PanelFocus::FileList => app.go_up(),
            _ => app.focus = PanelFocus::FileList,
        },

        // home
        KeyCode::Char('~') | KeyCode::Home => {
            app.current_path = crate::fs_utils::home_dir();
            app.selected = 0;
            app.reload_entries();
        }

        // hidden
        KeyCode::Char('.') => app.toggle_hidden(),

        // search
        KeyCode::Char('/') => {
            app.search_mode = true;
            app.search_query.clear();
            app.focus = PanelFocus::FileList;
        }

        // delete
        KeyCode::Char('d') => {
            app.status_msg =
                "⚠ Shift + D to delete"
                    .into();
        }
        KeyCode::Char('D') => {
            app.delete_selected();
        }

        // reload
        KeyCode::Char('r') | KeyCode::F(5) => {
            app.reload_entries();
            app.status_msg = " Reloaded".into();
        }

        _ => {}
    }

    false
}

fn handle_search_key(app: &mut App, code: KeyCode) -> bool {
    match code {
        KeyCode::Esc => {
            app.search_mode = false;
            app.search_query.clear();
            app.reload_entries();
        }
        KeyCode::Enter => {
            app.search_mode = false;
            app.reload_entries();
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.reload_entries();
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            app.reload_entries();
        }
        _ => {}
    }
    false
}
