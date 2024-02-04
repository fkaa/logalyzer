use std::env::args;
use std::io::{self, stdout};
use std::panic;
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
mod logalang;
mod parse;
mod system_report;
mod ui;

const BATCH_SIZE: usize = 64;

fn main() -> io::Result<()> {
    let Some(file) = args().nth(1) else {
        eprintln!(
            "Usage: {} [path to .log or .zip file]",
            args().nth(0).unwrap()
        );
        return Ok(());
    };

    if file.ends_with(".log") {
        let now = Instant::now();
        if let Err(e) = std::fs::remove_file("threaded_batched.db") {}

        let (send, recv) = mpsc::sync_channel(16);

        let handle = thread::spawn(move || {
            db::consumer(recv, BATCH_SIZE);
        });
        parse::producer(send, &file, BATCH_SIZE);

        handle.join().unwrap();

        let rows = db::get_row_count();
        println!("Program done in {:.2?} ({rows} rows)", now.elapsed());

        // Start TUI
        tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
        tui_logger::set_default_level(log::LevelFilter::Trace);

        let db = DbApi::new();

        let result = panic::catch_unwind(|| {
            run_ui(&file, db, rows).unwrap();
        });

        if let Err(e) = result {
            disable_raw_mode()?;
            stdout().execute(LeaveAlternateScreen)?;

            eprintln!("{:?}", e);
        }
    } else if file.ends_with(".zip") {
        let system_report = system_report::open(&file).unwrap();
        dbg!(system_report);
    }

    Ok(())
}

fn run_ui(file: &String, db: DbApi, rows: usize) -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app_state = AppState::new(file.clone(), db, rows);

    while !app_state.should_quit() {
        terminal.draw(|f| app_state.draw(f))?;
        app_state.handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
