use std::io;
use std::time::Duration;

use crossterm::event;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

use crate::db::DbApi;

mod cheat_sheet;
mod columns;
mod logs;

use cheat_sheet::{Key, KeyBinding};

use logs::LogFile;

#[derive(Clone)]
pub struct KeyBindings {
    pub up: KeyBinding,
    pub down: KeyBinding,
    pub top: KeyBinding,
    pub bot: KeyBinding,
    pub filter: KeyBinding,
    pub apply_filter: KeyBinding,
    pub close_filter: KeyBinding,
    pub columns: KeyBinding,
    pub quit: KeyBinding,
    pub console: KeyBinding,
}

impl Default for KeyBindings {
    fn default() -> Self {
        use KeyCode::*;

        KeyBindings {
            up: KeyBinding::new("Up".into(), vec![Key(None, Char('k'))]),
            down: KeyBinding::new("Down".into(), vec![Key(None, Char('j'))]),
            top: KeyBinding::new("Top".into(), vec![Key(None, Char('g'))]),
            bot: KeyBinding::new("Bot".into(), vec![Key(None, Char('G'))]),
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
        }
    }
}

pub struct AppState {
    log: LogFile,
    show_console: bool,
    should_quit: bool,
    bindings: KeyBindings,
}

impl AppState {
    pub fn new(file: String, db: DbApi, total_rows: usize) -> Self {
        let bindings = KeyBindings::default();
        let log = LogFile::new(bindings.clone(), file, db, total_rows);

        AppState {
            log,
            show_console: false,
            should_quit: false,
            bindings,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
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

        let mut constraints = vec![Constraint::Percentage(100)];
        if self.show_console {
            constraints.push(Constraint::Min(15));
        }
        let layout = Layout::new(Direction::Vertical, constraints).split(area);

        self.log.draw(layout[0], frame);
        if self.show_console {
            frame.render_widget(tui_w, layout[1]);
        }
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(50))? {
            let event = event::read()?;

            if self.bindings.quit.is_pressed(&event) {
                self.should_quit = true;
            }

            self.log.input(&event);
        }

        Ok(())
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
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
