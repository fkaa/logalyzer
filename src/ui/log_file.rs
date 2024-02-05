use std::io;
use std::time::Duration;

use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};
use tui_textarea::TextArea;

use crate::db::{DbApi, DbLogRow};
use crate::logalang::FilterRule;
use crate::ui::centered_rect;
use crate::ui::cheat_sheet::CheatSheet;
use crate::ui::columns;
use crate::ui::columns::ColumnSetting;

#[derive(Default)]
pub struct LogRows {
    offset: usize,
    rows: Vec<DbLogRow>,
}

enum Mode {
    Normal,
    Filter,
    Columns,
}

pub struct LogFileState {
    file: String,
    db: DbApi,
    total_rows: usize,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    should_quit: bool,
    loading: bool,
    rows: LogRows,
    mode: Mode,
    filter_text_area: TextArea<'static>,

    // columns
    columns: columns::ColumnList,
}

impl Widget for LogFileState {
    fn render(self, area: Rect, buf: &mut Buffer) {}
}

impl LogFileState {
    pub fn new(file: String, mut db: DbApi, total_rows: usize) -> Self {
        db.get_rows(0, 1000, vec![]);
        LogFileState {
            file,
            db,
            total_rows,
            table_state: TableState::new().with_selected(Some(1)),
            scrollbar_state: ScrollbarState::new(total_rows),
            should_quit: false,
            loading: false,
            rows: Default::default(),
            mode: Mode::Normal,
            filter_text_area: TextArea::default(),
            columns: columns::ColumnList::new(vec![
                ColumnSetting {
                    index: 0,
                    name: "Id".into(),
                    visible: true,
                    width: Constraint::Length(4),
                },
                ColumnSetting {
                    index: 1,
                    name: "Time".into(),
                    visible: true,
                    width: Constraint::Length(23),
                },
                ColumnSetting {
                    index: 2,
                    name: "Level".into(),
                    visible: true,
                    width: Constraint::Length(5),
                },
                ColumnSetting {
                    index: 3,
                    name: "Context".into(),
                    visible: true,
                    width: Constraint::Length(10),
                },
                ColumnSetting {
                    index: 4,
                    name: "Thread".into(),
                    visible: true,
                    width: Constraint::Length(5),
                },
                ColumnSetting {
                    index: 5,
                    name: "File".into(),
                    visible: true,
                    width: Constraint::Length(30),
                },
                ColumnSetting {
                    index: 6,
                    name: "Method".into(),
                    visible: true,
                    width: Constraint::Length(10),
                },
                ColumnSetting {
                    index: 7,
                    name: "Object".into(),
                    visible: true,
                    width: Constraint::Length(5),
                },
                ColumnSetting {
                    index: 8,
                    name: "Message".into(),
                    visible: true,
                    width: Constraint::Percentage(100),
                },
            ]),
        }
    }

    pub fn file_name(&self) -> &str {
        &self.file
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        let widths = self.columns.to_column_constraints();

        let rows = self
            .rows
            .rows
            .iter()
            .map(|r| db_row_to_ui_row(r, &self.columns.get_settings()))
            .collect::<Vec<_>>();

        let table = Table::new(rows, widths)
            .header(
                self.columns
                    .get_header_row()
                    .style(Style::new())
                    // To add space between the header and the rest of the rows, specify the margin
                    .bottom_margin(1),
            )
            .block(
                Block::default()
                    .title(&*self.file)
                    .title_alignment(Alignment::Right)
                    .title_style(Style {
                        fg: Option::from(Color::DarkGray),
                        bg: None,
                        underline_color: None,
                        add_modifier: Default::default(),
                        sub_modifier: Default::default(),
                    }),
            )
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">>");

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
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalLeft);

        let cheat_sheet = CheatSheet {
            items: vec![
                "Filter [f]".to_string(),
                "Column Visibility [c]".to_string(),
                "Move Top [g]".to_string(),
                "Move Bot [G]".to_string(),
                "Quit [q]".to_string(),
            ],
        };

        let layout = Layout::new(
            Direction::Vertical,
            vec![
                Constraint::Percentage(100),
                Constraint::Min(1),
                Constraint::Min(15),
            ],
        )
        .split(area);

        frame.render_stateful_widget(table, layout[0], &mut self.table_state);
        frame.render_widget(cheat_sheet.to_widget(), layout[1]);
        frame.render_widget(tui_w, layout[2]);

        frame.render_stateful_widget(
            scrollbar,
            layout[0].inner(&Margin {
                vertical: 0,
                horizontal: 0,
            }), // using a inner vertical margin of 1 unit makes the scrollbar inside the block
            &mut self.scrollbar_state,
        );

        if let Mode::Filter = self.mode {
            self.filter_text_area.set_block(
                Block::default()
                    .title("Edit filter(s)")
                    .borders(Borders::ALL),
            );

            let area = centered_rect(60, 60, area);

            let layout = Layout::new(
                Direction::Vertical,
                vec![Constraint::Percentage(100), Constraint::Min(1)],
            )
            .split(area);

            let cheat_sheet = CheatSheet {
                items: vec![
                    "Apply Filter [<C-f>]".to_string(),
                    "Close [Esc]".to_string(),
                ],
            };

            frame.render_widget(Clear, area); //this clears out the background
            frame.render_widget(self.filter_text_area.widget(), layout[0]);
            frame.render_widget(cheat_sheet.to_widget(), layout[1]);
        }

        if let Mode::Columns = self.mode {
            self.columns.render(frame);
        }
    }

    pub fn handle_event(&mut self, event: Event) -> io::Result<()> {
        while let Some(resp) = self.db.get_response() {
            self.rows.offset = resp.offset;
            self.rows.rows = resp.rows;
            self.loading = false;
        }

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
            Mode::Columns => {
                self.handle_column_input(&event);
            }
        }

        Ok(())
    }

    fn handle_column_input(&mut self, event: &Event) {
        if self.columns.input(event) {
            self.mode = Mode::Normal;
        }
    }

    fn handle_filter_input(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.apply_filter();
                self.mode = Mode::Normal;
            }
            KeyCode::Esc => {
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
            KeyCode::Char('c') => {
                self.mode = Mode::Columns;
            }
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('j') => {
                self.move_selection_relative(1);
            }
            KeyCode::Char('k') => {
                self.move_selection_relative(-1);
            }
            KeyCode::Char('g') => {
                self.move_selection_fixed(0usize);
            }
            KeyCode::Char('G') => {
                self.move_selection_fixed(self.total_rows);
            }
            _ => {}
        }
    }

    pub fn move_selection_relative(&mut self, delta: isize) {
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
            self.db
                .get_rows(self.rows.offset - 100, 300, self.get_filters());
            self.table_state.select(Some(selection + 100));
            *self.table_state.offset_mut() += 100;
        }

        if selection > 200 {
            self.db
                .get_rows(self.rows.offset + 100, 300, self.get_filters());
            self.table_state.select(Some(selection - 99));
            *self.table_state.offset_mut() -= 100;
        }
    }

    pub fn move_selection_fixed(&mut self, position: usize) {
        if self.loading {
            return;
        }

        let min_items_to_read = 300;
        if position < 300 {
            self.db
                .get_rows(0usize, min_items_to_read, self.get_filters());
            self.table_state.select(Some(0));
            *self.table_state.offset_mut() = 0;
        } else if position > (self.total_rows - min_items_to_read - 1) {
            let start_pos = (self.total_rows - min_items_to_read - 1);
            self.db
                .get_rows(start_pos, min_items_to_read, self.get_filters());
            self.table_state.select(Some(300)); // Select the last item in the current buffer
            *self.table_state.offset_mut() = 300;
        } else {
            self.db
                .get_rows(position, min_items_to_read, self.get_filters());
            self.table_state.select(Some(150)); // Select middle item
            *self.table_state.offset_mut() = 150;
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    fn apply_filter(&mut self) {
        self.db.get_rows(0, 300, self.get_filters());
        self.loading = true;
        *self.table_state.offset_mut() = 0;
        self.table_state.select(Some(0));

        self.mode = Mode::Normal;
    }

    fn get_filters(&self) -> Vec<FilterRule> {
        let mut filters = Vec::new();
        for line in self.filter_text_area.lines() {
            match crate::logalang::parse_line(line) {
                Ok(filter) => filters.push(filter),
                Err(e) => log::warn!("invalid filter: {e}"),
            }
        }
        filters
    }
}

fn db_row_to_ui_row<'a, 'b>(row: &'a DbLogRow, settings: &'b [ColumnSetting]) -> Row<'a> {
    let time = chrono::DateTime::UNIX_EPOCH + chrono::Duration::milliseconds(row.time);

    let mut cells = Vec::new();
    for setting in settings {
        if !setting.visible {
            continue;
        }

        let cell = match setting.index {
            0 => Cell::new(format!("{}", row.id)),
            1 => Cell::new(format!("{}", time.format("%y-%m-%d %T%.3f"))),
            2 => level_to_cell(row.level),
            3 => Cell::new(row.context.clone()),
            4 => Cell::new(row.thread.clone()),
            5 => Cell::new(Line::from(row.file.as_str()).alignment(Alignment::Right)),
            6 => Cell::new(row.method.clone()),
            7 => Cell::new(row.object.clone()),
            8 => Cell::new(row.message.clone()),
            _ => unreachable!(),
        };

        cells.push(cell);
    }

    Row::new(cells)
}

fn level_to_cell(level: i8) -> Cell<'static> {
    match level {
        crate::parse::TRACE => Cell::new("TRACE").style(Style::new().fg(Color::Gray)),
        crate::parse::INFO => Cell::new("INFO"),
        crate::parse::DEBUG => Cell::new("DEBUG").style(Style::new().fg(Color::Gray)),
        crate::parse::WARN => Cell::new("WARN").style(Style::new().fg(Color::Yellow)),
        crate::parse::ERROR => Cell::new("ERROR").style(Style::new().fg(Color::Red)),
        crate::parse::FATAL => Cell::new("FATAL").style(Style::new().fg(Color::Red)),
        _ => Cell::new("UNKNWN"),
    }
}
