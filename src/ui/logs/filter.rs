use crossterm::event::Event;
use ratatui::prelude::*;

use crate::ui::KeyBinding;

struct FilterBindings {
    up: KeyBinding,
    down: KeyBinding,
    edit: KeyBinding,
    add: KeyBinding,
    remove: KeyBinding,
}

pub struct Filters {
    available_columns: Vec<String>,
    filters: Vec<Filter>,
    bindings: FilterBindings,
}

impl Filters {
    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {}
    pub fn input(&mut self, event: &Event) -> bool {
        false
    }
}

pub struct Filter {
    column: String,
    ty: FilterType,
    value: String,
}

pub enum FilterType {
    Include,
    Highlight,
}
