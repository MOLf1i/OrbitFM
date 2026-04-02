use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::App,
    fs_utils::{format_size, home_dir, shortcuts_list},
    types::PanelFocus,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // path bar
            Constraint::Min(0),    // 3 colums
            Constraint::Length(3), // info bar
        ])
        .split(area);

    draw_path_bar(f, app, main[0]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(18), // Shortcuts
            Constraint::Percentage(40), // files
            Constraint::Percentage(42), // preview
        ])
        .split(main[1]);

    draw_shortcuts(f, app, cols[0]);
    draw_file_list(f, app, cols[1]);
    draw_preview(f, app, cols[2]);
    draw_info_bar(f, app, main[2]);
}

fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

fn draw_path_bar(f: &mut Frame, app: &App, area: Rect) {
    let text = if app.search_mode {
        format!("🔍 Пошук: {}_", app.search_query)
    } else {
        format!(" {}", app.current_path.display())
    };
    let p = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 󰀘 OrbitFM ")
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    f.render_widget(p, area);
}

fn draw_shortcuts(f: &mut Frame, app: &mut App, area: Rect) {
    let home = home_dir();
    let focused = app.focus == PanelFocus::Shortcuts;

    let mut items: Vec<ListItem> = app
        .shortcuts
        .iter()
        .map(|p| {
            let icon = shortcut_icon(p, &home);
            let name = if p == &home {
                "Home"
            } else {
                p.file_name().and_then(|n| n.to_str()).unwrap_or("?")
            };
            ListItem::new(format!("{} {}", icon, name))
        })
        .collect();

    if !app.disks.is_empty() {
        items.push(
            ListItem::new("─────────────")
                .style(Style::default().fg(Color::DarkGray)),
        );
    }

    for disk in &app.disks {
        items.push(ListItem::new(disk.label.clone()));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Shortcuts ")
                .border_style(border_style(focused)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut app.shortcut_state);
}

fn draw_file_list(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == PanelFocus::FileList;

    let items: Vec<ListItem> = app
        .entries
        .iter()
        .map(|e| {
            let icon = file_icon(e.is_dir, &e.extension);
            let style = if e.is_dir {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let size_str = if e.is_dir {
                "       ".into()
            } else {
                format!("{:>8}", format_size(e.size))
            };
            let line = Line::from(vec![
                Span::raw(format!("{} ", icon)),
                Span::styled(e.name.clone(), style),
                Span::styled(
                    format!("  {}", size_str),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let title = if app.search_mode && !app.search_query.is_empty() {
        format!("  \"{}\" ({}) ", app.search_query, app.entries.len())
    } else {
        format!(" Files ({}) ", app.entries.len())
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style(focused)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_preview(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == PanelFocus::Preview;

    let title = if let Some(e) = app.entries.get(app.selected) {
        format!(" {} {} ", file_icon(e.is_dir, &e.extension), e.name)
    } else {
        " Перегляд ".into()
    };

    let p = Paragraph::new(Text::raw(app.preview_content.clone()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style(focused)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0))
        .style(Style::default().fg(Color::Gray));

    f.render_widget(p, area);
}

fn draw_info_bar(f: &mut Frame, app: &App, area: Rect) {
    let text = if let Some(e) = app.entries.get(app.selected) {
        let detail = if e.is_dir {
            "Catalogue".into()
        } else {
            format_size(e.size)
        };
        format!("  {} {}  │  {}  │  {}", file_icon(e.is_dir, &e.extension), e.name, detail, app.status_msg)
    } else {
        app.status_msg.clone()
    };

    let p = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Інфо ")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(p, area);
}

fn file_icon(is_dir: bool, ext: &str) -> &'static str {
    if is_dir {
        return "";
    }
    match ext {
        "rs" => "",
        "py" => "",
        "js" | "ts" => "",
        "html" | "htm" => "",
        "css" => "",
        "json" | "toml" | "yaml" | "yml" => "",
        "md" | "txt" | "rst" => "",
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" => "󰋩",
        "mp4" | "mkv" | "avi" | "mov" | "webm" => "󰕧",
        "mp3" | "flac" | "ogg" | "wav" => "󰎄",
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => "󰀼",
        "pdf" => "󰈦",
        "exe" | "sh" | "bat" | "bin" => "",
        "c" | "cpp" | "h" | "hpp" => "",
        "go" => "󰟓",
        "java" | "class" => "",
        "rb" => "",
        "php" => "",
        _ => "",
    }
}

pub fn shortcut_icon<'a>(path: &std::path::Path, home: &std::path::Path) -> &'static str {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if path == home {
        ""
    } else {
        match name {
            "Desktop" => "󰟀",
            "Documents" => "󰈙",
            "Downloads" => "󰉍",
            "Pictures" => "",
            "Music" => "󰁧",
            "Videos" => "󰈫",
            _ => "",
        }
    }
}
