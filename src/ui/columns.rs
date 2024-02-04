use crossterm::event;
use crossterm::event::{Event, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Line, Modifier, Style};
use ratatui::style::Color;
use ratatui::widgets::{Block, Borders, Clear, HighlightSpacing, List, ListItem, ListState, Row};
use crate::ui::{centered_rect};
use crate::ui::cheat_sheet::CheatSheet;

pub struct ColumnSetting {
    pub index: usize,
    pub name: String,
    pub visible: bool,
    pub width: Constraint,
}

pub struct ColumnList {
    state: ListState,
    items: Vec<ColumnSetting>,
}

impl ColumnList {
    pub fn new(items: Vec<ColumnSetting>) -> Self {
        ColumnList {
            state: ListState::default(),
            items,
        }
    }

    pub fn to_column_constraints(&self) -> Vec<Constraint> {
        let widths = self
            .items
            .iter()
            .filter_map(|c| if c.visible { Some(c.width) } else { None })
            .collect::<Vec<_>>();

        widths
    }

    pub(crate) fn get_header_row(&self) -> Row {
        Row::new(
            self
                .items
                .iter()
                .filter_map(|c| {
                    if c.visible {
                        Some(c.name.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
        )
    }

    pub(crate) fn to_list_items(&self) -> Vec<ListItem<'static>> {
        self.items
            .iter()
            .map(|c| {
                let line = if c.visible {
                    let mut l = Line::from(format!("[x] {}", c.name));
                    l.patch_style(Style::new().fg(Color::LightGreen));
                    l
                } else {
                    let mut l = Line::from(format!("[ ] {}", c.name));
                    l.patch_style(Style::new().fg(Color::Gray));
                    l
                };

                ListItem::new(line)
            })
            .collect()
    }


    /// Returns true if the popup should close
    pub(crate) fn input(&mut self, event: &Event) -> bool {
        if let Event::Key(key) = event {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => {
                        self.next();
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        self.previous();
                    }
                    KeyCode::Char(' ') => {
                        self.toggle();
                    }
                    KeyCode::Esc | KeyCode::Char('c') => {
                        return true;
                    }
                    _ => {}
                }
            }
        }

        false
    }

    pub(crate) fn get_settings(&self) -> &[ColumnSetting] {
        &self.items
    }

    pub(crate) fn render(&mut self, frame: &mut Frame) {
        let area = frame.size();

        let cheat_sheet = CheatSheet {
            items: vec![
                "Toggle Visibility [ ]".to_string(),
                "Close [Esc]".to_string(),
            ],
        };

        let area = centered_rect(60, 60, area);
        let layout = Layout::new(
            Direction::Vertical,
            vec![Constraint::Percentage(100), Constraint::Min(1)],
        )
            .split(area);

        let outer_block = Block::default()
            .borders(Borders::ALL)
            //                .fg(TEXT_COLOR)
            //                .bg(TODO_HEADER_BG)
            .title("Columns")
            .title_alignment(Alignment::Center);

        let items = self.to_list_items();

        let items = List::new(items)
            .block(outer_block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_widget(Clear, area);
        frame.render_stateful_widget(items, layout[0], &mut self.state);
        frame.render_widget(cheat_sheet.to_widget(), layout[1]);
    }

    fn toggle(&mut self) {
        if let Some(idx) = self.state.selected() {
            self.items[idx].visible = !self.items[idx].visible;
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
