use std::fs;
use std::io::{self, stdout};
use std::sync::{atomic::AtomicU64, mpsc, Arc};
use std::thread;
use std::time::Instant;

use crate::db::DbApi;

use crate::parse::{ColumnDefinition, Parser};
use crate::ui::AppState;
use crossterm::{
    event::DisableMouseCapture,
    event::EnableMouseCapture,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::backend::CrosstermBackend;

use ratatui::Terminal;

mod config;
mod db;
mod logalang;
mod parse;
mod ui;

#[derive(Default)]
pub struct LoadingProgress {
    pub total_bytes: AtomicU64,
    pub parsed_bytes: AtomicU64,
    pub rows_parsed: AtomicU64,
    pub rows_inserted: AtomicU64,
}

const BATCH_SIZE: usize = 16;

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

    let progress = Arc::new(LoadingProgress::default());

    let columns = parser.columns.clone();
    let column_count = parser.columns.len();

    let db_progress = progress.clone();
    let db_handle = thread::spawn(move || {
        db::consumer(column_count, recv, BATCH_SIZE, db_progress);
    });
    let parse_progress = progress.clone();
    let parse_file = file.to_string();
    let parse_handle = thread::spawn(move || {
        parse::producer(send, parse_file, parser, BATCH_SIZE, parse_progress);
    });

    if first != "parse" {
        run_ui(columns, file, db, progress)?;
    }

    // let rows = db::get_row_count();
    // println!("Program done in {:.2?} ({rows} rows)", now.elapsed());

    db_handle.join().unwrap();
    parse_handle.join().unwrap();

    Ok(())
}

fn get_parser() -> Parser {
    let toml = fs::read_to_string("log4net.toml").unwrap();
    let config = toml::from_str::<config::LogFormatConfiguration>(&toml).unwrap();
    config.into()
}

fn run_ui(
    columns: Vec<ColumnDefinition>,
    file: &String,
    db: DbApi,
    progress: Arc<LoadingProgress>,
) -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;

    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        println!("{:#?}", info.location());
        println!("{:#?}", info)
    }));
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app_state = AppState::new(columns, file.clone(), db, progress);

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
