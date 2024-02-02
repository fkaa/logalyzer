use std::io;
use std::time::Duration;

use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use tui_textarea::TextArea;

use crate::db::{DbApi, DbLogRow};

pub struct LogRowState {
    total_rows: usize,
    total_filtered_rows: usize,
}

#[derive(Default)]
pub struct LogRows {
    offset: usize,
    rows: Vec<DbLogRow>,
}

enum Mode {
    Normal,
    Filter,
}

pub struct AppState {
    db: DbApi,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    should_quit: bool,
    loading: bool,
    rows: LogRows,
    mode: Mode,
    filter_text_area: TextArea<'static>,
    cursor_position: usize,
}

impl AppState {
    pub fn new(mut db: DbApi, total_rows: usize) -> Self {
        db.get_rows(0, 1000, None);
        AppState {
            db,
            table_state: TableState::new().with_selected(Some(1)),
            scrollbar_state: ScrollbarState::new(total_rows),
            should_quit: false,
            loading: false,
            rows: Default::default(),
            mode: Mode::Normal,
            filter_text_area: TextArea::default(),
            cursor_position: 0,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let widths = [
            Constraint::Length(4),
            Constraint::Length(23),
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Length(5),
            Constraint::Length(30),
            Constraint::Length(10),
            Constraint::Length(5),
            Constraint::Percentage(100),
        ];

        let rows = self
            .rows
            .rows
            .iter()
            .map(db_row_to_ui_row)
            .collect::<Vec<_>>();

        let table = Table::new(rows, widths)
            .header(
                Row::new(vec![
                    "Id", "Time", "Level", "Context", "Thread", "File", "Method", "Object",
                    "Message",
                ])
                .style(Style::new().bold())
                // To add space between the header and the rest of the rows, specify the margin
                .bottom_margin(1),
            )
            .block(Block::default().title("Table"))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">>");

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalLeft);
        let area = frame.size();

        let layout = Layout::new(
            Direction::Vertical,
            vec![Constraint::Percentage(100), Constraint::Min(1)],
        )
        .split(area);

        frame.render_stateful_widget(table, layout[0], &mut self.table_state);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            }), // using a inner vertical margin of 1 unit makes the scrollbar inside the block
            &mut self.scrollbar_state,
        );

        if let Mode::Filter = self.mode {
            self.filter_text_area.set_block(Block::default().title("Edit filter(s)").borders(Borders::ALL));

            let area = centered_rect(60, 60, area);
            frame.render_widget(Clear, area); //this clears out the background
            frame.render_widget(self.filter_text_area.widget(), area);
        }
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        while let Some(resp) = self.db.get_response() {
            self.rows.offset = resp.offset;
            self.rows.rows = resp.rows;
            self.loading = false;
        }

        if event::poll(Duration::from_millis(50))? {
            let event = event::read()?;

            match self.mode {
                Mode::Normal => {
                    if let Event::Key(key) = &event {
                        if key.kind == event::KeyEventKind::Press {
                            self.handle_normal_input(key);
                        }
                    }
                }
                Mode::Filter => {
                    if let Event::Key(key) = &event {
                        if key.kind == event::KeyEventKind::Press {
                            self.handle_filter_input(key);
                        }
                    }

                    self.filter_text_area.input(event);
                }
            }
        }

        Ok(())
    }

    fn handle_filter_input(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.apply_filter();
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn handle_normal_input(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Char('f') => {
                self.mode = Mode::Filter;
            }
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('j') => {
                self.move_selection(1);
            }
            KeyCode::Char('k') => {
                self.move_selection(-1);
            }
            _ => {}
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.loading {
            return;
        }

        let selection = self.table_state.selected().unwrap();

        if delta < 0 {
            if delta.abs() as usize > selection {
                self.table_state.select(Some(0));
            } else {
                self.table_state
                    .select(Some(selection - delta.abs() as usize));
            }
        } else {
            self.table_state.select(Some(selection + delta as usize));
        }

        if selection < 50 && self.rows.offset >= 50 {
            self.db.get_rows(
                self.rows.offset - 100,
                300,
                if self.filter_text_area.lines().first().unwrap().to_string().is_empty() {
                    None
                } else {
                    Some(self.filter_text_area.lines().first().unwrap().to_string().clone())
                },
            );
            self.table_state.select(Some(selection + 100));
            *self.table_state.offset_mut() += 100;
        }

        if selection > 200 {
            self.db.get_rows(
                self.rows.offset + 100,
                300,
                if self.filter_text_area.lines().first().unwrap().to_string().is_empty() {
                    None
                } else {
                    Some(self.filter_text_area.lines().first().unwrap().to_string().clone())
                },
            );
            self.table_state.select(Some(selection - 99));
            *self.table_state.offset_mut() -= 100;
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    fn apply_filter(&mut self) {
        self.db.get_rows(0, 300, Some(self.filter_text_area.lines().first().unwrap().to_string()));
        self.loading = true;
        *self.table_state.offset_mut() = 0;
        self.table_state.select(Some(0));

        self.mode = Mode::Normal;
    }
}

fn db_row_to_ui_row(row: &DbLogRow) -> Row {
    let time = chrono::DateTime::UNIX_EPOCH + chrono::Duration::milliseconds(row.time);

    Row::new([
        Cell::new(format!("{}", row.id)),
        Cell::new(format!("{}", time.format("%y-%m-%d %T%.3f"))),
        level_to_cell(row.level),
        Cell::new(row.context.clone()),
        Cell::new(row.thread.clone()),
        Cell::new(Line::from(row.file.as_str()).alignment(Alignment::Right)),
        Cell::new(row.method.clone()),
        Cell::new(row.object.clone()),
        Cell::new(row.message.clone()),
    ])
}

fn level_to_cell(level: i8) -> Cell<'static> {
    match level {
        crate::parse::TRACE => Cell::new("TRACE").style(Style::new().fg(Color::Gray)),
        crate::parse::INFO => Cell::new("INFO"),
        crate::parse::DEBUG => {
            Cell::new("DEBUG").style(Style::new().bg(Color::White).fg(Color::Gray))
        }
        crate::parse::WARN => Cell::new("WARN").style(Style::new().fg(Color::Yellow)),
        crate::parse::ERROR => Cell::new("ERROR").style(Style::new().fg(Color::Red)),
        crate::parse::FATAL => Cell::new("FATAL").style(Style::new().fg(Color::Red)),
        _ => Cell::new("UNKNWN"),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ],
    )
    .split(r);

    Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ],
    )
    .split(popup_layout[1])[1]
}
