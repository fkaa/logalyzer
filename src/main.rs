use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::ops::Range;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use chrono::{DateTime, NaiveDate};
use rusqlite::{Connection, ToSql};
use std::io::{self, stdout};
use unicode_bom::Bom;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};

const TRACE: i8 = 0;
const INFO: i8 = 1;
const DEBUG: i8 = 2;
const WARN: i8 = 3;
const ERROR: i8 = 4;
const FATAL: i8 = 5;

#[derive(Debug, Default)]
struct LogRow {
    line: String,
    time: u16,
    time_unixtime: i64,
    level: i8,
    context: Range<u16>,
    thread: Range<u16>,
    file: Range<u16>,
    method: Range<u16>,
    object: Range<u16>,
    message: u16,
}

impl LogRow {
    pub fn time(&self) -> &str {
        &self.line[..self.time as usize]
    }

    /*pub fn level(&self) -> &str {
        &self.line[self.level.start as usize..self.level.end as usize]
    }*/

    pub fn context(&self) -> &str {
        &self.line[self.context.start as usize..self.context.end as usize]
    }

    pub fn thread(&self) -> &str {
        &self.line[self.thread.start as usize..self.thread.end as usize]
    }

    pub fn file(&self) -> &str {
        &self.line[self.file.start as usize..self.file.end as usize]
    }

    pub fn method(&self) -> &str {
        &self.line[self.method.start as usize..self.method.end as usize]
    }

    pub fn object(&self) -> &str {
        &self.line[self.object.start as usize..self.object.end as usize]
    }

    pub fn message(&self) -> &str {
        &self.line[self.message as usize..]
    }
}

fn parse_line(line: String) -> Option<LogRow> {
    let rest = &line;
    let level_start = 25;
    let level_end = level_start + rest[level_start..].find(' ')?;

    let level = match &line[level_start..level_end] {
        "TRACE" => TRACE,
        "INFO" => INFO,
        "DEBUG" => DEBUG,
        "WARN" => WARN,
        "ERROR" => ERROR,
        "FATAL" => FATAL,
        _ => -1,
    };

    let context_start = level_end + 2;
    let context_end = context_start + rest[context_start..].find("] ")?;

    let thread_start = context_end + 3;
    let thread_end = thread_start + rest[thread_start..].find("] ")?;

    let file_start = thread_end + 2;
    let file_end = file_start + rest[file_start..].find(",  ")?;

    let method_start = file_end + 3;
    let method_end = method_start + rest[method_start..].find(" <")?;

    let object_start = method_end + 2;
    let object_end = object_start + rest[object_start..].find("> - ")?;

    let message_start = object_end + 4;

    let timestr = &line[..23];

    let (y, rest) = timestr.split_once("-")?;
    let (m, rest) = rest.split_once("-")?;
    let (d, rest) = rest.split_once(" ")?;
    let (h, rest) = rest.split_once(":")?;
    let (min, rest) = rest.split_once(":")?;
    let (s, ms) = rest.split_once(",")?;

    let y = y.parse::<i32>().ok()?;
    let m = m.parse::<u32>().ok()?;
    let d = d.parse::<u32>().ok()?;
    let h = h.parse::<u32>().ok()?;
    let min = min.parse::<u32>().ok()?;
    let s = s.parse::<u32>().ok()?;
    let ms = ms.parse::<u32>().ok()?;

    let time_unixtime = NaiveDate::from_ymd(y, m, d).and_hms_milli(h, min, s, ms);
    let time_unixtime = time_unixtime.timestamp_millis();

    Some(LogRow {
        line,
        time: 23,
        time_unixtime,
        level,
        //level: level_start as u16..level_end as u16,
        context: context_start as u16..context_end as u16,
        thread: thread_start as u16..thread_end as u16,
        file: file_start as u16..file_end as u16,
        method: method_start as u16..method_end as u16,
        object: object_start as u16..object_end as u16,
        message: message_start as u16,
    })
}

const BATCH_SIZE: usize = 64;

fn producer(send: mpsc::SyncSender<Vec<LogRow>>, path: &str) {
    let bom = getbom(path);
    let mut reader = BufReader::new(File::open(path).unwrap());

    if bom != Bom::Null {
        let mut x = [0; 3];
        let _y = reader.read_exact(&mut x);
    }

    let mut batch = Vec::new();

    let now = Instant::now();
    let mut i = 0;
    for line in reader.lines() {
        let line = line.unwrap();

        let row = parse_line(line).unwrap();
        batch.push(row);

        if batch.len() >= BATCH_SIZE {
            let old_vec = std::mem::replace(&mut batch, Vec::new());
            send.send(old_vec).unwrap();
        }
        i += 1;

        // dbg!(row);
    }
    println!("Reading {i} lines took {:.2?}", now.elapsed());
}

fn consumer(recv: mpsc::Receiver<Vec<LogRow>>) {
    let mut conn = Connection::open("threaded_batched.db").unwrap();
    conn.execute_batch(
        "PRAGMA journal_mode = OFF;
              PRAGMA synchronous = 0;
              PRAGMA cache_size = 1000000;
              PRAGMA locking_mode = EXCLUSIVE;",
    )
    .expect("PRAGMA");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS row (
                id INTEGER not null primary key,
                time INTEGER not null,
                level INTEGER not null,
                context TEXT not null,
                thread TEXT not null,
                file TEXT not null,
                method TEXT not null,
                object TEXT not null,
                message TEXT not null)",
        [],
    )
    .unwrap();
    conn.execute("CREATE INDEX idx_log_time ON row (time)", [])
        .unwrap();

    let now = Instant::now();
    let mut bump = bumpalo::Bump::new();

    let conn = conn.transaction().unwrap();

    {
        let mut sql_values = "(NULL, ?, ?, ?, ?, ?, ?, ?, ?),".repeat(BATCH_SIZE);
        sql_values.pop();
        let query = format!("INSERT INTO row VALUES {}", sql_values);
        let mut stmt = conn.prepare_cached(&query).unwrap();

        for rows in recv {
            let mut row_values: Vec<&dyn ToSql> = Vec::with_capacity(BATCH_SIZE * 8);

            /*let mut row_values: Vec<&str> = Vec::new();
            for row in rows.iter() {
                row_values.push(row.time());
                row_values.push(row.level());
                row_values.push(row.context());
                row_values.push(row.thread());
                row_values.push(row.file());
                row_values.push(row.method());
                row_values.push(row.object());
                row_values.push(row.message());
            }*/
            for row in rows.iter() {
                row_values.push(bump.alloc(row.time_unixtime));
                row_values.push(bump.alloc(row.level));
                row_values.push(bump.alloc(row.context()));
                row_values.push(bump.alloc(row.thread()));
                row_values.push(bump.alloc(row.file()));
                row_values.push(bump.alloc(row.method()));
                row_values.push(bump.alloc(row.object()));
                row_values.push(bump.alloc(row.message()));
            }

            stmt.execute(rusqlite::params_from_iter(row_values))
                .unwrap();

            bump.reset();
        }
    }
    conn.commit().unwrap();
    println!("Inserting took {:.2?}", now.elapsed());
}

fn main() -> io::Result<()> {
    let now = Instant::now();
    if let Err(e) = std::fs::remove_file("threaded_batched.db") {
        eprintln!("{e}");
    }
    let file = std::env::args().nth(1).unwrap();

    let (send, recv) = mpsc::sync_channel(16);

    let handle = thread::spawn(move || {
        consumer(recv);
    });
    producer(send, &file);

    handle.join().unwrap();

    println!("Program done in {:.2?}", now.elapsed());

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(ui)?;
        should_quit = handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn ui(frame: &mut Frame) {
    let mut table_state = TableState::default();
    table_state.select(Some(2));
    let rows = [
        Row::new(vec![
            "2023-12-04 01:12:30,690",
            "DEBUG",
            "",
            "24",
            "CA.Core\\WebProxy\\TcpConnection\\TcpConnection.cs(73)",
            "Open",
            "",
            "Setting up secure connection [73fc :: 95bf]",
        ]),
        Row::new(vec![
            "2023-12-04 01:12:30,690",
            "DEBUG",
            "",
            "24",
            "CA.Core\\WebProxy\\TcpConnection\\TcpConnection.cs(73)",
            "Open",
            "",
            "Setting up secure connection [73fc :: 95bf]",
        ]),
        Row::new(vec![
            "2023-12-04 01:12:30,690",
            "DEBUG",
            "",
            "24",
            "CA.Core\\WebProxy\\TcpConnection\\TcpConnection.cs(73)",
            "Open",
            "",
            "Setting up secure connection [73fc :: 95bf]",
        ]),
    ];
    let widths = [
        Constraint::Length(23),
        Constraint::Length(5),
        Constraint::Length(10),
        Constraint::Length(5),
        Constraint::Length(30),
        Constraint::Length(10),
        Constraint::Length(5),
        Constraint::Percentage(100),
    ];
    let table = Table::new(rows, widths)
        .header(
            Row::new(vec![
                "Time", "Level", "Context", "Thread", "File", "Method", "Object", "Message",
            ])
            .style(Style::new().bold())
            // To add space between the header and the rest of the rows, specify the margin
            .bottom_margin(1),
        )
        .block(Block::default().title("Table"))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">>");

    frame.render_stateful_widget(table, frame.size(), &mut table_state);
}

fn getbom(path: &str) -> Bom {
    let mut file = File::open(path).unwrap();
    Bom::from(&mut file)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_line() {
        let line = parse_line("2023-12-04 01:12:30,690  DEBUG [] [  24] CA.Core\\WebProxy\\TcpConnection\\TcpConnection.cs(73),  Open <> - Setting up secure connection [73fc :: 95bf]".into()).unwrap();

        assert_eq!(line.time(), "2023-12-04 01:12:30,690");
        assert_eq!(line.level(), "DEBUG");
        assert_eq!(line.context(), "");
        assert_eq!(line.thread(), "  24");
        assert_eq!(
            line.file(),
            "CA.Core\\WebProxy\\TcpConnection\\TcpConnection.cs(73)"
        );
        assert_eq!(line.method(), "Open");
        assert_eq!(line.object(), "");
        assert_eq!(
            line.message(),
            "Setting up secure connection [73fc :: 95bf]"
        );
    }
}
