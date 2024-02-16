use std::io::{self, stdout};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use crate::db::DbApi;
use crate::parse::ParserInstruction;
use crate::parse::{ColumnDefinition, Parser};
use crate::ui::AppState;
use crossterm::{
    event::DisableMouseCapture,
    event::EnableMouseCapture,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Constraint;
use ratatui::Terminal;

mod db;
mod logalang;
mod parse;
mod ui;

const BATCH_SIZE: usize = 64;

fn main() -> io::Result<()> {
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);

    let now = Instant::now();
    if let Err(e) = std::fs::remove_file("threaded_batched.db") {
        eprintln!("{e}");
    }
    let first = std::env::args().nth(1).unwrap();
    let second = std::env::args().nth(2);

    let file = if first == "parse" {
        second.as_ref().unwrap()
    } else {
        &first
    };

    let parser = get_parser();
    let db = DbApi::new(parser.columns.clone());

    let (send, recv) = mpsc::sync_channel(16);

    let columns = parser.columns.clone();

    let column_count = parser.columns.len();
    let handle = thread::spawn(move || {
        db::consumer(column_count, recv, BATCH_SIZE);
    });
    parse::producer(send, &file, parser, BATCH_SIZE);

    handle.join().unwrap();

    let rows = db::get_row_count();
    println!("Program done in {:.2?} ({rows} rows)", now.elapsed());

    if first != "parse" {
        run_ui(columns, file, db, rows)?;
    }

    Ok(())
}

fn get_parser() -> Parser {
    use ParserInstruction::*;
    let parser = Parser::new(
        vec![
            Begin,
            Skip(23),
            EmitDate,
            Skip(2),
            Begin,
            SkipUntilChar(' '),
            EmitEnumeration(vec![
                "TRACE".into(),
                "DEBUG".into(),
                "INFO".into(),
                "WARN".into(),
                "ERROR".into(),
                "FATAL".into(),
            ]),
            SkipUntilChar('['),
            Skip(1),
            Begin,
            SkipUntilChar(']'),
            EmitString,
            SkipUntilChar('['),
            Skip(1),
            Begin,
            SkipUntilChar(']'),
            EmitString,
            Skip(2),
            Begin,
            SkipUntilChar(','),
            EmitString,
            Skip(3),
            Begin,
            SkipUntilString(" <".into()),
            EmitString,
            Skip(2),
            Begin,
            SkipUntilChar('>'),
            EmitString,
            SkipUntilChar('-'),
            Skip(2),
            Begin,
            EmitRemainder,
        ],
        vec![
            ColumnDefinition::date("Date".to_string(), Constraint::Length(23)),
            ColumnDefinition::enumeration(
                "Level".to_string(),
                Constraint::Length(5),
                vec![
                    "TRACE".into(),
                    "DEBUG".into(),
                    "INFO".into(),
                    "WARN".into(),
                    "ERROR".into(),
                    "FATAL".into(),
                ],
            ),
            ColumnDefinition::string("Context".to_string(), Constraint::Length(8)),
            ColumnDefinition::string("Thread".to_string(), Constraint::Length(8)),
            ColumnDefinition::string("File".to_string(), Constraint::Length(8)),
            ColumnDefinition::string("Method".to_string(), Constraint::Length(8)),
            ColumnDefinition::string("Object".to_string(), Constraint::Length(8)),
            ColumnDefinition::string("Message".to_string(), Constraint::Percentage(100)),
        ],
    );
    parser
}

fn run_ui(columns: Vec<ColumnDefinition>, file: &String, db: DbApi, rows: usize) -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;

    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        println!("{:?}", info)
    }));
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app_state = AppState::new(columns, file.clone(), db, rows);

    while !app_state.should_quit() {
        terminal.draw(|f| app_state.draw(f))?;
        app_state.handle_events()?;
    }

    restore_terminal()?;

    Ok(())
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    stdout().execute(DisableMouseCapture)?;

    Ok(())
}
