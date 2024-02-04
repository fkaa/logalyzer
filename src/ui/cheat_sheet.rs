use ratatui::layout::Alignment;
use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::widgets::Paragraph;

pub struct CheatSheet {
    pub items: Vec<String>,
}

impl CheatSheet {
    pub fn to_widget(&self) -> Paragraph {
        let cheat_sheet_style = Style::new().bg(Color::Blue).fg(Color::White);
        let span_items = self
            .items
            .iter()
            .map(|f| Span::styled(f, cheat_sheet_style))
            .flat_map(|span| [span, Span::raw(" ")])
            .collect::<Vec<Span>>();

        let cheat_sheet_items = Line::from(span_items);
        Paragraph::new(cheat_sheet_items).alignment(Alignment::Left)
    }
}
