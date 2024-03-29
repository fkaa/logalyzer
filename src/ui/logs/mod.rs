use crossterm::event::{self};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::{prelude::*, widgets::*};

use tui_textarea::{CursorMove, Input, TextArea};

use super::cheat_sheet::{CheatSheet, Key, KeyBinding};
use super::columns::{ColumnList, ColumnSetting};
use super::KeyBindings;
use crate::db::{DbApi, DbLogRow, DbResponse, DbRowValue};
use crate::logalang::FilterRule;
use crate::parse::{ColumnDefinition, ColumnType};

#[derive(Default)]
pub struct LogRows {
    offset: usize,
    rows: Vec<DbLogRow>,
}

enum Mode {
    Normal,
    FilterSelection,
    FilterInput,
    Columns,
}

pub struct LogFile {
    file: String,
    db: DbApi,
    total_rows: usize,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    should_quit: bool,
    loading: bool,
    rows: LogRows,
    renderable_rows: u16,
    mode: Mode,
    bindings: KeyBindings,
    max_id_row_width: u32,
    show_preview: bool,

    filter_values: Vec<String>,
    filter_active_value_idx: usize,
    filter_text_area: TextArea<'static>,

    // columns
    columns: ColumnList,
}

impl LogFile {
    pub fn new(
        columns: Vec<ColumnDefinition>,
        bindings: KeyBindings,
        file: String,
        mut db: DbApi,
        total_rows: usize,
    ) -> Self {
        db.get_rows(0, 300, vec![]);

        let mut column_settings = Vec::new();
        column_settings.push(ColumnSetting {
            index: 0,
            name: "Id".into(),
            visible: true,
            width: Constraint::Length(8),
            enumerations: vec![],
        });

        for (idx, column) in columns.iter().enumerate() {
            column_settings.push(ColumnSetting {
                index: idx + 1,
                name: column.nice_name.clone(),
                visible: true,
                width: column.column_width,
                enumerations: if let ColumnType::Enumeration(enums) = &column.column_type {
                    enums.clone()
                } else {
                    vec![]
                },
            })
        }

        let columns_count = columns.len();
        let columns = ColumnList::new(column_settings, &bindings);

        LogFile {
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
            columns,
            max_id_row_width: 0,
            bindings,
            show_preview: false,
            renderable_rows: 0,
            filter_values: vec!["".to_string(); columns_count + 1],
            filter_active_value_idx: 0,
        }
    }

    fn on_rows_received(&mut self, offset: usize, rows: Vec<DbLogRow>) {
        self.rows.offset = offset;
        self.rows.rows = rows;

        self.max_id_row_width = self
            .rows
            .rows
            .iter()
            .map(|r| {
                if let DbRowValue::Integer(val) = r[0] {
                    val
                } else {
                    0
                }
            })
            .max()
            .map(|id| id.ilog10() + 1)
            .unwrap_or(4);
        self.columns.items[0].width = Constraint::Length(self.max_id_row_width as u16);
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        while let Some(resp) = self.db.get_response() {
            match resp {
                DbResponse::FilterApplied {
                    id: _,
                    total_filtered_rows: _,
                } => {}
                DbResponse::RowsFetched {
                    id: _,
                    offset,
                    limit: _,
                    rows,
                } => {
                    self.on_rows_received(offset, rows);
                    self.loading = false;
                }
            }
        }

        let widths = self.columns.to_column_constraints();

        let rows = self
            .rows
            .rows
            .iter()
            .map(|r| db_row_to_ui_row(r, &self.columns.get_settings()))
            .collect::<Vec<_>>();

        let header = if let Mode::FilterSelection = self.mode {
            self.columns.get_header_row_numbered()
        } else {
            self.columns.get_header_row()
        };
        let table = Table::new(rows, widths)
            .header(
                header
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

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalLeft);

        let cheat_sheet = CheatSheet {
            items: vec![
                self.bindings.quit.clone(),
                self.bindings.console.clone(),
                self.bindings.columns.clone(),
                self.bindings.filter.clone(),
                self.bindings.up.clone(),
                self.bindings.down.clone(),
                self.bindings.top.clone(),
                self.bindings.bot.clone(),
                self.bindings.preview.clone(),
            ],
        };

        let mut text = String::new();
        if let Some(selected_row) = &self.rows.rows.get(self.table_state.selected().unwrap()) {
            if let DbRowValue::String(msg) = selected_row.last().unwrap() {
                text = msg.clone().replace('↵', "\n");
            }
        }
        let preview_window = Paragraph::new(text)
            .block(Block::new().borders(Borders::ALL).title("Preview"))
            .wrap(Wrap { trim: false });

        let mut constraints = Vec::new();
        constraints.push(Constraint::Percentage(100));
        if self.show_preview {
            constraints.push(Constraint::Min(15));
        }
        constraints.push(Constraint::Min(1));

        let layout = Layout::new(Direction::Vertical, constraints).split(area);

        self.renderable_rows = layout[0].height - 2; // -1 column header, -1 spacing
        frame.render_stateful_widget(table, layout[0], &mut self.table_state);

        if self.show_preview {
            frame.render_widget(preview_window, layout[1]);
            frame.render_widget(cheat_sheet.to_widget(), layout[2]);
        } else {
            frame.render_widget(cheat_sheet.to_widget(), layout[1]);
        }

        frame.render_stateful_widget(
            scrollbar,
            layout[0].inner(&Margin {
                vertical: 0,
                horizontal: 0,
            }), // using a inner vertical margin of 1 unit makes the scrollbar inside the block
            &mut self.scrollbar_state,
        );

        if let Mode::FilterInput = self.mode {
            self.filter_text_area.set_block(
                Block::default()
                    .title("Edit filter(s)")
                    .borders(Borders::ALL),
            );

            let area = super::centered_rect(60, 60, area);

            let layout = Layout::new(
                Direction::Vertical,
                vec![Constraint::Percentage(100), Constraint::Min(1)],
            )
            .split(area);

            let cheat_sheet = CheatSheet {
                items: vec![
                    self.bindings.apply_filter.clone(),
                    self.bindings.close_filter.clone(),
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

    fn get_filters(&self) -> Vec<FilterRule> {
        let mut filters = Vec::new();
        for (idx, line) in self.filter_values.iter().enumerate() {
            match crate::logalang::parse_line(line) {
                Ok(filter) => filters.push(FilterRule { column_name: format!("Column{idx}"), rules: filter }),
                Err(e) => log::warn!("invalid filter: {e}"),
            }
        }
        filters
    }

    fn apply_filter(&mut self) {
        self.db.get_rows(0, 300, self.get_filters());
        self.loading = true;
        *self.table_state.offset_mut() = 0;
        self.table_state.select(Some(0));

        self.mode = Mode::Normal;
    }

    pub fn input(&mut self, event: &Event) {
        match self.mode {
            Mode::Normal => {
                self.handle_normal_input(&event);
            }
            Mode::FilterSelection => {
                self.handle_filter_selection(event);
            }
            Mode::FilterInput => {
                if let Event::Key(key) = &event {
                    if key.kind == event::KeyEventKind::Press {
                        self.handle_filter_input(key);
                    }
                }

                self.filter_text_area.input(event.clone());
            }
            Mode::Columns => {
                self.handle_column_input(&event);
            }
        }
    }

    fn handle_column_input(&mut self, event: &Event) {
        if self.columns.input(event) {
            self.mode = Mode::Normal;
        }
    }

    fn handle_filter_input(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Char('f') | KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.filter_values[self.filter_active_value_idx] = self.filter_text_area.lines()[0].to_string();
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

    fn handle_filter_selection(&mut self, event: &Event) {
        for (idx, _col_item) in self.columns.items.iter().enumerate() {
            let bind = KeyBinding::new(
                "".into(),
                vec![Key(
                    None,
                    KeyCode::Char(char::from_digit((idx + 1) as u32, 10).unwrap()),
                )],
            );

            if bind.is_pressed(event) {
                self.filter_active_value_idx = idx;
                self.filter_text_area = TextArea::new(vec![self.filter_values[self.filter_active_value_idx].to_string()]);
                self.filter_text_area.move_cursor(CursorMove::End);
                self.mode = Mode::FilterInput;
                break;
            }
        }
    }

    fn handle_normal_input(&mut self, event: &Event) {
        if self.bindings.filter.is_pressed(event) {
            self.mode = Mode::FilterSelection;
            return;
        }

        if self.bindings.columns.is_pressed(event) {
            self.mode = Mode::Columns;
            return;
        }

        if self.bindings.quit.is_pressed(event) {
            self.should_quit = true;
            return;
        }

        if self.bindings.up.is_pressed(event) || is_scroll_up(event) {
            self.move_selection_relative(-1);
            return;
        }

        if self.bindings.down.is_pressed(event) || is_scroll_down(event) {
            self.move_selection_relative(1);
            return;
        }

        if self.bindings.pg_up.is_pressed(event) {
            self.move_selection_relative(-(self.renderable_rows as isize));
            return;
        }

        if self.bindings.pg_down.is_pressed(event) {
            self.move_selection_relative(self.renderable_rows as _);
            return;
        }

        if self.bindings.top.is_pressed(event) {
            self.move_selection_fixed(0usize);
            return;
        }

        if self.bindings.bot.is_pressed(event) {
            self.move_selection_fixed(self.total_rows);
            return;
        }

        if self.bindings.preview.is_pressed(event) {
            self.show_preview = !self.show_preview;
            return;
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
        } else if position > (self.total_rows - min_items_to_read) {
            let start_pos = self.total_rows - min_items_to_read;
            self.db
                .get_rows(start_pos, min_items_to_read, self.get_filters());
            self.table_state.select(Some(299)); // Select the last item
            *self.table_state.offset_mut() = (300 - self.renderable_rows) as usize;
        // Offset the visible items to show the last item at bottom
        } else {
            self.db
                .get_rows(position, min_items_to_read, self.get_filters());
            self.table_state.select(Some(149)); // Select middle item
            *self.table_state.offset_mut() = (149 - self.renderable_rows / 2) as usize;
        }
    }
}

fn row_value_to_cell(row: DbRowValue) -> Cell<'static> {
    match row {
        DbRowValue::String(val) => Cell::new(val),
        DbRowValue::Date(time) => {
            let time = chrono::DateTime::UNIX_EPOCH + chrono::Duration::milliseconds(time);

            Cell::new(format!("{}", time.format("%y-%m-%d %T%.3f")))
        }
        DbRowValue::Integer(val) => Cell::new(format!("{val}")),
    }
}

fn db_row_to_ui_row<'a, 'b>(rows: &'a DbLogRow, settings: &'b [ColumnSetting]) -> Row<'a> {
    let mut cells = Vec::new();

    for (setting, row) in settings.iter().zip(rows) {
        if !setting.visible {
            continue;
        }

        let cell = if setting.enumerations.len() > 0 {
            let DbRowValue::Integer(v) = row else {
                panic!("hmm");
            };
            level_to_cell(*v as i8, &setting.enumerations)
        } else {
            row_value_to_cell(row.clone())
        };

        cells.push(cell);
    }

    Row::new(cells)
}

fn level_to_cell(level: i8, enumerations: &[String]) -> Cell<'static> {
    let colors = [
        Some(Color::Gray),
        None,
        Some(Color::Gray),
        Some(Color::Yellow),
        Some(Color::Red),
        Some(Color::Red),
    ];

    let mut cell = Cell::new(enumerations[level as usize].clone());

    if let Some(Some(col)) = colors.get(level as usize) {
        cell = cell.style(Style::new().fg(*col));
    }

    cell
}

fn is_scroll_up(event: &Event) -> bool {
    if let Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        ..
    }) = event
    {
        return true;
    }

    false
}

fn is_scroll_down(event: &Event) -> bool {
    if let Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        ..
    }) = event
    {
        return true;
    }

    false
}
