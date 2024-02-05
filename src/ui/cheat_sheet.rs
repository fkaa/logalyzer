use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::Alignment;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub struct CheatSheet {
    pub items: Vec<KeyBinding>,
}

impl CheatSheet {
    pub fn to_widget(&self) -> Paragraph {
        let keybinding_style = Style::new().bg(Color::Green).fg(Color::White);
        let key_style = keybinding_style.clone().bold();

        let mut spans = Vec::new();

        for bind in &self.items {
            spans.push(Span::styled(format!("{} [", bind.name), keybinding_style));
            for (idx, key) in bind.keys.iter().enumerate() {
                spans.push(Span::styled(format!("{key}"), key_style));
                if idx < bind.keys.len() - 1 {
                    spans.push(Span::styled("/", keybinding_style));
                }
            }
            spans.push(Span::styled("]", keybinding_style));
            spans.push(Span::raw(" "));
        }

        let keybindings = Line::from(spans);
        Paragraph::new(keybindings).alignment(Alignment::Left)
    }
}

#[derive(Clone)]
pub struct KeyBinding {
    name: String,
    keys: Vec<Key>,
}

impl KeyBinding {
    pub fn new(name: String, keys: Vec<Key>) -> Self {
        KeyBinding { name, keys }
    }

    pub fn is_pressed(&self, event: &Event) -> bool {
        self.keys.iter().any(|k| k.is_pressed(event))
    }
}

#[derive(Clone)]
pub struct Key(pub Option<KeyModifiers>, pub KeyCode);

impl Key {
    pub fn is_pressed(&self, event: &Event) -> bool {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                if let Some(modifier) = self.0 {
                    if modifier != key.modifiers {
                        return false;
                    }
                }
                if key.code == self.1 {
                    return true;
                }
            }
        }

        false
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(modifiers) = self.0 {
            if modifiers.contains(KeyModifiers::CONTROL) {
                write!(fmt, "C-")?;
            }
        }

        match self.1 {
            KeyCode::Char(c) => write!(fmt, "{c}")?,
            _ => write!(fmt, "TODO")?,
        }

        Ok(())
    }
}
