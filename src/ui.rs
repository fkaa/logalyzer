use std::io;
use std::time::Duration;

use crossterm::event;
use crossterm::event::{Event, KeyCode};
use ratatui::{prelude::*, widgets::*};

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

pub struct AppState {
    db: DbApi,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    should_quit: bool,
    loading: bool,
    rows: LogRows,
}

impl AppState {
    pub fn new(mut db: DbApi, total_rows: usize) -> Self {
        db.get_rows(0, 1000);

        AppState {
            db,
            table_state: TableState::new().with_selected(Some(1)),
            scrollbar_state: ScrollbarState::new(total_rows),
            should_quit: false,
            loading: false,
            rows: Default::default(),
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

        frame.render_stateful_widget(
            table,
            area.inner(&Margin {
                vertical: 0,
                horizontal: 2,
            }),
            &mut self.table_state,
        );
        frame.render_stateful_widget(
            scrollbar,
            area.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            }), // using a inner vertical margin of 1 unit makes the scrollbar inside the block
            &mut self.scrollbar_state,
        );
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        while let Some(resp) = self.db.get_response() {
            self.rows.offset = resp.offset;
            self.rows.rows = resp.rows;
            self.loading = false;
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
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
            }
        }

        Ok(())
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
            self.db.get_rows(self.rows.offset - 100, 300);
            self.table_state.select(Some(selection + 100));
            *self.table_state.offset_mut() += 100;
        }

        if selection > 200 {
            self.db.get_rows(self.rows.offset + 100, 300);
            self.table_state.select(Some(selection - 99));
            *self.table_state.offset_mut() -= 100;
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
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
