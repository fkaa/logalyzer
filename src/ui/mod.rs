use std::io;
use std::sync::{atomic::Ordering, Arc};
use std::time::Duration;

use bytesize::ByteSize;
use crossterm::event;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::palette::tailwind::GREEN;
use ratatui::{prelude::*, widgets::*};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

use crate::db::DbApi;
use crate::LoadingProgress;

mod cheat_sheet;
mod columns;
mod logs;

use cheat_sheet::{Key, KeyBinding};

use crate::parse::ColumnDefinition;
use logs::LogFile;

#[derive(Clone)]
pub struct KeyBindings {
    pub up: KeyBinding,
    pub down: KeyBinding,
    pub pg_up: KeyBinding,
    pub pg_down: KeyBinding,
    pub top: KeyBinding,
    pub bot: KeyBinding,
    pub filter: KeyBinding,
    pub apply_filter: KeyBinding,
    pub close_filter: KeyBinding,
    pub columns: KeyBinding,
    pub quit: KeyBinding,
    pub console: KeyBinding,
    pub preview: KeyBinding,
}

impl Default for KeyBindings {
    fn default() -> Self {
        use KeyCode::*;

        KeyBindings {
            up: KeyBinding::new("Up".into(), vec![Key(None, Char('k')), Key(None, Up)]),
            down: KeyBinding::new("Down".into(), vec![Key(None, Char('j')), Key(None, Down)]),
            pg_up: KeyBinding::new("Page Up".into(), vec![Key(None, PageUp)]),
            pg_down: KeyBinding::new("Page Down".into(), vec![Key(None, PageDown)]),
            top: KeyBinding::new("Top".into(), vec![Key(None, Char('g')), Key(None, Home)]),
            bot: KeyBinding::new("Bot".into(), vec![Key(None, Char('G')), Key(None, End)]),
            filter: KeyBinding::new("Filter".into(), vec![Key(None, Char('f'))]),
            close_filter: KeyBinding::new("Close".into(), vec![Key(None, Esc)]),
            apply_filter: KeyBinding::new(
                "Apply filter".into(),
                vec![Key(Some(KeyModifiers::CONTROL), Char('f'))],
            ),
            columns: KeyBinding::new("Columns".into(), vec![Key(None, Char('c'))]),
            quit: KeyBinding::new("Quit".into(), vec![Key(None, Char('q'))]),
            console: KeyBinding::new(
                "Console".into(),
                vec![Key(Some(KeyModifiers::CONTROL), Char('c'))],
            ),
            preview: KeyBinding::new("Preview".into(), vec![Key(None, Char('p'))]),
        }
    }
}

pub struct AppState {
    log: Option<LogFile>,
    columns: Vec<ColumnDefinition>,
    file: String,
    db: Option<DbApi>,
    progress: Arc<LoadingProgress>,
    show_console: bool,
    should_quit: bool,
    bindings: KeyBindings,
}

impl AppState {
    pub fn new(
        columns: Vec<ColumnDefinition>,
        file: String,
        db: DbApi,
        progress: Arc<LoadingProgress>,
    ) -> Self {
        let bindings = KeyBindings::default();

        AppState {
            log: None,
            columns,
            db: Some(db),
            file,
            progress,
            show_console: false,
            should_quit: false,
            bindings,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        if self.log.is_none() {
            let total_bytes = self.progress.total_bytes.load(Ordering::SeqCst);
            let parsed_bytes = self.progress.parsed_bytes.load(Ordering::SeqCst);
            let rows_parsed = self.progress.rows_parsed.load(Ordering::SeqCst);
            let rows_inserted = self.progress.rows_inserted.load(Ordering::SeqCst);

            if total_bytes != 0 && total_bytes == parsed_bytes && rows_parsed == rows_inserted {
                self.log = Some(LogFile::new(
                    self.columns.clone(),
                    self.bindings.clone(),
                    self.file.clone(),
                    self.db.take().unwrap(),
                    rows_inserted as _,
                ))
            }
        }

        let tui_w: TuiLoggerWidget = TuiLoggerWidget::default()
            .block(
                Block::default()
                    .title("stdout")
                    .border_style(Style::default().fg(Color::White).bg(Color::Black))
                    .borders(Borders::ALL),
            )
            .output_separator('|')
            .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Long))
            .output_target(false)
            .output_file(false)
            .output_line(false)
            .style(Style::default().fg(Color::White).bg(Color::Black));

        let area = frame.size();

        match &mut self.log {
            Some(log) => {
                let mut constraints = vec![Constraint::Percentage(100)];
                if self.show_console {
                    constraints.push(Constraint::Min(15));
                }
                let layout = Layout::new(Direction::Vertical, constraints).split(area);

                log.draw(layout[0], frame);
                if self.show_console {
                    frame.render_widget(tui_w, layout[1]);
                }
            }
            None => {
                let total_bytes = self.progress.total_bytes.load(Ordering::SeqCst);
                let parsed_bytes = self.progress.parsed_bytes.load(Ordering::SeqCst);
                let rows_parsed = self.progress.rows_parsed.load(Ordering::SeqCst);
                let rows_inserted = self.progress.rows_inserted.load(Ordering::SeqCst);

                let area = centered_rect2(60, 10, area);

                let outer_block = Block::default()
                    .padding(Padding::horizontal(1))
                    .borders(Borders::ALL)
                    .title("Loading...")
                    .title_alignment(Alignment::Left);

                let inner = outer_block.inner(area);
                let layout = Layout::new(
                    Direction::Vertical,
                    vec![Constraint::Length(4), Constraint::Length(4)],
                )
                .split(inner);

                let parse_block = Block::default()
                    //.borders(Borders::ALL)
                    .title("Parsing log file...")
                    .title_alignment(Alignment::Center);
                let parse_gauge = Gauge::default()
                    .block(parse_block)
                    .use_unicode(true)
                    .ratio((parsed_bytes as f64 / total_bytes as f64).clamp(0.0, 1.0))
                    .gauge_style(GREEN.c600)
                    .label(format!(
                        "{}/{}",
                        ByteSize::b(parsed_bytes),
                        ByteSize::b(total_bytes)
                    ));

                let db_block = Block::default()
                    //.borders(Borders::ALL)
                    .title("Inserting in database...")
                    .title_alignment(Alignment::Center);
                let db_gauge = Gauge::default()
                    .block(db_block)
                    .use_unicode(true)
                    .gauge_style(GREEN.c800)
                    .ratio(if rows_parsed > 0 {
                        (rows_inserted as f64 / rows_parsed as f64).clamp(0.0, 1.0)
                    } else {
                        0.0
                    })
                    .label(format!("{}/{}", rows_inserted, rows_parsed));

                frame.render_widget(Clear, area);
                frame.render_widget(outer_block, area);
                frame.render_widget(parse_gauge, layout[0]);
                frame.render_widget(db_gauge, layout[1]);
            }
        }
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(50))? {
            let event = event::read()?;

            if self.bindings.console.is_pressed(&event) {
                self.show_console = !self.show_console;
            } else if self.bindings.quit.is_pressed(&event) {
                self.should_quit = true;
            } else if let Some(log) = &mut self.log {
                log.input(&event);
            }
        }

        Ok(())
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}
fn centered_rect2(percent_x: u16, height_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Percentage(50),
            Constraint::Min(height_y),
            Constraint::Percentage(50),
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
