use std::io::{self, stdout};
use std::io::{BufRead, Read};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use crate::db::DbApi;
use crate::ui::AppState;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

mod db;
mod parse;
mod ui;

const BATCH_SIZE: usize = 64;

fn main() -> io::Result<()> {
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

    let (send, recv) = mpsc::sync_channel(16);

    let handle = thread::spawn(move || {
        db::consumer(recv, BATCH_SIZE);
    });
    parse::producer(send, &file, BATCH_SIZE);

    handle.join().unwrap();

    let rows = db::get_row_count();
    println!("Program done in {:.2?} ({rows} rows)", now.elapsed());

    if first != "parse" {
        let db = DbApi::new();

        run_ui(db, rows)?;
    }

    Ok(())
}

fn run_ui(db: DbApi, rows: usize) -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app_state = AppState::new(db, rows);

    while !app_state.should_quit() {
        terminal.draw(|f| app_state.draw(f))?;
        app_state.handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
