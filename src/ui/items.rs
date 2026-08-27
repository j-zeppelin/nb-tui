use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::{
    config::{self, Config},
    nb::{FolderNav, NbItemKind},
    ui::{self, state::items::ItemState},
};

pub fn render(
    f: &mut Frame,
    area: Rect,
    state: &mut ItemState,
    nav: &FolderNav,
    config: &Config,
    focused: bool,
) {
    let border_style = ui::border_style(focused);

    let block = Block::new()
        .borders(Borders::ALL)
        .title("[n] Notes")
        .border_style(border_style)
        .border_type(BorderType::Rounded);

    let mut items: Vec<ListItem> = state
        .visible()
        .map(|i| {
            let mut label = String::new();

            if i.pinned {
                label.push_str(config.indicators.pinned());
                label.push(' ');
            }

            if i.encrypted {
                label.push_str(config.indicators.encrypted());
                label.push(' ');
            }

            label.push_str(config.indicators.for_kind(&i.kind));
            if !matches!(i.kind, NbItemKind::Note) {
                label.push(' ');
            }

            label.push_str(&i.title);

            let line = Line::from(vec![
                Span::styled(format!("[{}] ", i.id), Style::default().green()),
                Span::styled(
                    label,
                    if i.kind == NbItemKind::Folder {
                        Style::new().bold()
                    } else {
                        Style::new()
                    },
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    items.insert(
        0,
        ListItem::new(
            Line::from(format!("./{}", nav.breadcrumbs().join("/")))
                .style(Style::default().bold().fg(Color::Blue).dim()),
        ),
    );

    let list = List::new(items)
        .highlight_style(if focused {
            Style::new().bold().bg(Color::DarkGray)
        } else {
            Style::default()
        })
        .highlight_symbol("> ")
        .block(block);

    f.render_stateful_widget(list, area, &mut state.list_state);
}
