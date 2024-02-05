use std::io;
use std::time::Duration;

use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};
use tui_textarea::TextArea;

use crate::db::{DbApi, DbLogRow};
use crate::logalang::FilterRule;
use crate::system_report::SystemReport;
use crate::ui::cheat_sheet::CheatSheet;
use crate::ui::columns::ColumnSetting;

mod cheat_sheet;
mod columns;
mod log_file;

pub use log_file::LogFileState;

pub struct AppState {
    system_report: Option<SystemReport>,
    log_files: Vec<LogFileState>,

    tabs: Tabs<'static>,
    selected_tab: usize,

    should_quit: bool,
}

impl Widget for &AppState {
    fn render(self, area: Rect, buf: &mut Buffer) {}
}

impl AppState {
    pub fn new(system_report: Option<SystemReport>, log_files: Vec<LogFileState>) -> Self {
        let mut tab_names = Vec::new();
        if system_report.is_some() {
            tab_names.push("System report".to_string());
        }
        for l in &log_files {
            tab_names.push(l.file_name().to_string());
        }

        let tabs = Tabs::new(tab_names)
            .block(Block::default().title("Tabs").borders(Borders::ALL))
            .style(Style::default().white())
            .highlight_style(Style::default().yellow())
            .divider(symbols::DOT)
            .padding("->", "<-");

        AppState {
            system_report,
            log_files,
            tabs,
            selected_tab: 0,

            should_quit: false,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    fn render_tabs(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(self.tabs.clone(), area);
    }
    fn render_content(&self, frame: &mut Frame, area: Rect) {
        self.render_tabs(frame, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let p = Paragraph::new(Text::from(Line::raw("◄ ► to change tab | Press q to quit")));
        frame.render_widget(p, area);
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.size();

        use Constraint::*;
        let vertical = Layout::new(Direction::Vertical, [Length(1), Min(0), Length(1)]);
        let [header_area, inner_area, footer_area]: [Rect; 3] =
            (*vertical.split(area)).try_into().unwrap();

        self.render_tabs(frame, header_area);

        self.render_content(frame, inner_area);

        self.render_footer(frame, footer_area);
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(50))? {
            let event = event::read()?;

            if self.system_report.is_some() {
                match self.selected_tab {
                    0 => {}
                    idx @ _ => {
                        let log_file = &mut self.log_files[idx - 1];
                        log_file.handle_event(event)?;
                    }
                }
            } else {
                let log_file = &mut self.log_files[self.selected_tab];
                log_file.handle_event(event)?;
            }
        }

        Ok(())
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
