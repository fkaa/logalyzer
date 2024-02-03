use std::io;
use std::time::Duration;

use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Stylize;
use ratatui::{prelude::*, widgets::*};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};
use tui_textarea::TextArea;

use crate::db::{DbApi, DbLogRow};
use crate::logalang::FilterRule;

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

pub struct ColumnSetting {
    index: usize,
    name: String,
    visible: bool,
    width: Constraint,
}

pub struct ColumnList {
    state: ListState,
    items: Vec<ColumnSetting>,
}

impl ColumnList {
    fn new(items: Vec<ColumnSetting>) -> Self {
        ColumnList {
            state: ListState::default(),
            items,
        }
    }

    fn to_list_items(&self) -> Vec<ListItem<'static>> {
        self.items
            .iter()
            .map(|c| {
                let line = if c.visible {
                    Line::from(format!("SHOW {}", c.name))
                } else {
                    Line::from(format!("HIDE {}", c.name))
                };

                ListItem::new(line)
            })
            .collect()
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

pub struct AppState {
    file: String,
    db: DbApi,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    should_quit: bool,
    loading: bool,
    rows: LogRows,
    mode: Mode,
    filter_text_area: TextArea<'static>,

    // columns
    columns: ColumnList,
}

impl AppState {
    pub fn new(file: String, mut db: DbApi, total_rows: usize) -> Self {
        db.get_rows(0, 1000, vec![]);
        AppState {
            file,
            db,
            table_state: TableState::new().with_selected(Some(1)),
            scrollbar_state: ScrollbarState::new(total_rows),
            should_quit: false,
            loading: false,
            rows: Default::default(),
            mode: Mode::Normal,
            filter_text_area: TextArea::default(),
            columns: ColumnList::new(vec![
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

    pub fn draw(&mut self, frame: &mut Frame) {
        let widths = self
            .columns
            .items
            .iter()
            .filter_map(|c| if c.visible { Some(c.width) } else { None })
            .collect::<Vec<_>>();

        let rows = self
            .rows
            .rows
            .iter()
            .map(|r| db_row_to_ui_row(r, &self.columns.items))
            .collect::<Vec<_>>();

        let table = Table::new(rows, widths)
            .header(
                Row::new(
                    self.columns
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
                    .style(Style::new())
                    // To add space between the header and the rest of the rows, specify the margin
                    .bottom_margin(1),
            )
            .block(Block::default().title(&*self.file).title_alignment(Alignment::Right).title_style(Style {
                fg: Option::from(Color::DarkGray),
                bg: None,
                underline_color: None,
                add_modifier: Default::default(),
                sub_modifier: Default::default(),
            }))
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

        let cheat_sheet_style = Style::new()
            .bg(Color::Blue)
            .fg(Color::White);
        let cheat_sheet_items =
            Line::from(vec![
                Span::styled("Filter [f]", cheat_sheet_style),
                Span::raw(" "),
                Span::styled("Column Visibility [c]", cheat_sheet_style),
                Span::raw(" "),
                Span::styled("Quit [q]", cheat_sheet_style)]);
        let cheat_sheet = Paragraph::new(cheat_sheet_items).alignment(Alignment::Left);

        let area = frame.size();

        let layout = Layout::new(
            Direction::Vertical,
            vec![Constraint::Percentage(100), Constraint::Min(1), Constraint::Min(15)],
        )
            .split(area);

        frame.render_stateful_widget(table, layout[0], &mut self.table_state);
        frame.render_widget(cheat_sheet, layout[1]);
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
            frame.render_widget(Clear, area); //this clears out the background
            frame.render_widget(self.filter_text_area.widget(), area);
        }

        if let Mode::Columns = self.mode {
            let area = centered_rect(60, 60, area);

            let outer_block = Block::default()
                .borders(Borders::ALL)
                //                .fg(TEXT_COLOR)
                //                .bg(TODO_HEADER_BG)
                .title("Columns")
                .title_alignment(Alignment::Center);

            let inner_area = outer_block.inner(area);

            let items = self.columns.to_list_items();

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
            frame.render_stateful_widget(items, area, &mut self.columns.state);
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
                Mode::Columns => {
                    self.handle_column_input(&event);
                }
            }
        }

        Ok(())
    }

    fn handle_column_input(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => {
                        self.columns.next();
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        self.columns.previous();
                    }
                    KeyCode::Char(' ') => {
                        self.columns.toggle();
                    }
                    KeyCode::Esc | KeyCode::Char('c') => {
                        self.mode = Mode::Normal;
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_filter_input(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.apply_filter();
                self.mode = Mode::Normal;
            }
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
            KeyCode::Char('c') => {
                self.mode = Mode::Columns;
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
        crate::parse::DEBUG => {
            Cell::new("DEBUG").style(Style::new().fg(Color::Gray))
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

/* TODO: stuff for syntax highlighting
use crate::logalang::{LogalangParser, Rule};
use pest::{Parser, Token};
use std::collections::HashMap;

fn try_parse(lines: &[String]) -> Vec<Vec<(usize, usize, Style)>> {
    let mut line_tokens = Vec::new();

    for line in lines {
        let result = LogalangParser::parse(Rule::filter, line);

        let mut styles = HashMap::new();
        styles.insert(Rule::expr, Style::new().fg(Color::LightGreen));
        styles.insert(Rule::column_name, Style::new().fg(Color::Yellow));
        styles.insert(Rule::and, Style::new().fg(Color::LightCyan));
        styles.insert(Rule::or, Style::new().fg(Color::LightCyan));
        styles.insert(Rule::not, Style::new().fg(Color::LightCyan));

        let mut spans = Vec::new();

        let mut state = HashMap::new();
        if let Ok(result) = result {
            for token in result.tokens() {
                log::debug!("token: {:?}", token);
                match token {
                    Token::Start { rule, pos } => {
                        state.insert(rule, pos);
                    }
                    Token::End { rule, pos } => {
                        if let Some(start) = state.remove(&rule) {
                            if let Some(style) = styles.get(&rule) {
                                spans.push((start.pos(), pos.pos(), style.clone()));
                            }
                        }
                    }
                }
            }
            line_tokens.push(spans);
        } else {
            line_tokens.push(vec![]);
        }
    }

    line_tokens
}*/
