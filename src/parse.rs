use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::ops::Range;
use std::sync::mpsc;
use std::time::Instant;

use chrono::NaiveDate;
use unicode_bom::Bom;

pub const TRACE: i8 = 0;
pub const INFO: i8 = 1;
pub const DEBUG: i8 = 2;
pub const WARN: i8 = 3;
pub const ERROR: i8 = 4;
pub const FATAL: i8 = 5;

#[derive(Debug, Default)]
pub struct LogRow {
    pub line: String,
    pub time: u16,
    pub time_unixtime: i64,
    pub level: i8,
    pub context: Range<u16>,
    pub thread: Range<u16>,
    pub file: Range<u16>,
    pub method: Range<u16>,
    pub object: Range<u16>,
    pub message: u16,
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
            .rsplit_once('\\')
            .unwrap()
            .1
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

    let context_start = level_end + rest[level_end..].find('[')? + 1;
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

pub fn producer(send: mpsc::SyncSender<Vec<LogRow>>, path: &str, batch_size: usize) {
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

        if let Some(row) = parse_line(line) {
            batch.push(row);

            if batch.len() >= batch_size {
                let old_vec = std::mem::replace(&mut batch, Vec::new());
                send.send(old_vec).unwrap();
            }

            i += 1;
        }
    }

    println!("Reading {i} lines took {:.2?}", now.elapsed());
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
        assert_eq!(line.level, 2);
        assert_eq!(line.context(), "");
        assert_eq!(line.thread(), "  24");
        assert_eq!(
            line.file(),
            "TcpConnection.cs(73)"
        );
        assert_eq!(line.method(), "Open");
        assert_eq!(line.object(), "");
        assert_eq!(
            line.message(),
            "Setting up secure connection [73fc :: 95bf]"
        );
    }
}
