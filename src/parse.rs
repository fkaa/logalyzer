use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::ops::Range;
use std::sync::mpsc;
use std::time::Instant;

use crate::db::DbLogRow;
use chrono::NaiveDate;
use log::warn;
use ratatui::layout::Constraint;
use unicode_bom::Bom;

pub const TRACE: i8 = 0;
pub const INFO: i8 = 1;
pub const DEBUG: i8 = 2;
pub const WARN: i8 = 3;
pub const ERROR: i8 = 4;
pub const FATAL: i8 = 5;

// {Datetime:DATE} {Level:ENUM(TRACE,INFO,DEBUG,WARN,ERROR,FATAL)} {Context:WORD} {Thread:WORD}
// {File:WORD} {Method:WORD} {Object:WORD} {Message:REST}

#[derive(Clone)]
pub enum ColumnType {
    String,
    Date,
    Enumeration(Vec<String>),
}

#[derive(Clone)]
pub struct ColumnDefinition {
    pub nice_name: String,
    pub column_type: ColumnType,
    pub column_width: Constraint,
}

impl ColumnDefinition {
    pub fn string(nice_name: String, column_width: Constraint) -> Self {
        ColumnDefinition {
            nice_name,
            column_type: ColumnType::String,
            column_width,
        }
    }

    pub fn date(nice_name: String, column_width: Constraint) -> Self {
        ColumnDefinition {
            nice_name,
            column_type: ColumnType::Date,
            column_width,
        }
    }

    pub fn enumeration(
        nice_name: String,
        column_width: Constraint,
        enumerations: Vec<String>,
    ) -> Self {
        ColumnDefinition {
            nice_name,
            column_type: ColumnType::Enumeration(enumerations),
            column_width,
        }
    }
}

#[derive(Clone, Debug)]
pub enum RowValue {
    String(String),
    Date(i64),
    Integer(i64),
}

pub struct Parser {
    instructions: Vec<ParserInstruction>,
    pub columns: Vec<ColumnDefinition>,
}

impl Parser {
    pub fn new(instructions: Vec<ParserInstruction>, columns: Vec<ColumnDefinition>) -> Self {
        Parser {
            instructions,
            columns,
        }
    }

    pub fn parse_line(&self, line: &str) -> Result<Vec<RowValue>, String> {
        use ParserInstruction::*;

        let mut values = Vec::new();

        let mut index = 0usize;
        let mut begin_index = 0;

        for i in &self.instructions {
            match i {
                EmitDate => {
                    let date_str = &line[begin_index..index];
                    let date =
                        parse_datetime(date_str).ok_or(format!("Invalid datetime {date_str}"))?;

                    values.push(RowValue::Date(date));
                }
                EmitString => {
                    let date = &line[begin_index..index];
                    values.push(RowValue::String(date.to_string()));
                }
                EmitEnumeration(enums) => {
                    let value = &line[begin_index..index];
                    let idx = enums
                        .iter()
                        .position(|e| e == value)
                        .ok_or(format!("Unknown enum {value}"))?;
                    values.push(RowValue::Integer(idx as _));
                }
                EmitRemainder => {
                    let date = &line[begin_index..];
                    values.push(RowValue::String(date.to_string()));
                }
                Begin => begin_index = index,
                Skip(amount) => index += *amount as usize,
                SkipUntilChar(ch) => index += line[index..].find(*ch).unwrap(),
                SkipUntilString(text) => index += line[index..].find(&*text).unwrap(),
            }
        }

        Ok(values)
    }
}

pub enum ParserInstruction {
    EmitDate,
    EmitString,
    EmitEnumeration(Vec<String>),
    EmitRemainder,
    Begin,
    Skip(u16),
    SkipUntilChar(char),
    SkipUntilString(String),
}

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

pub fn producer(
    send: mpsc::SyncSender<Vec<DbLogRow>>,
    path: &str,
    parser: Parser,
    batch_size: usize,
) {
    let bom = getbom(path);
    let mut reader = BufReader::new(File::open(path).unwrap());

    if bom != Bom::Null {
        let mut x = [0; 3];
        let _y = reader.read_exact(&mut x);
    }

    let mut batch = Vec::new();

    let now = Instant::now();
    let mut i = 0;
    let mut latest_parsed_row = None;

    for line in reader.lines() {
        let line = line.unwrap();
        match parser.parse_line(&line) {
            Ok(values) => {
                if let Some(row) = latest_parsed_row.take() {
                    batch.push(row);
                    if batch.len() >= batch_size {
                        let old_vec = std::mem::replace(&mut batch, Vec::new());
                        send.send(old_vec).unwrap();
                    }
                }
                latest_parsed_row = Some(values);

                i += 1;
            }
            Err(e) => {
                warn!("Error while parsing line: {e}");
                if let Some(mut row) = latest_parsed_row.take() {
                    if let Some(last) = row.last_mut() {
                        if let RowValue::String(ref mut s) = last {
                            *s += &line;
                        }
                    }
                }
            }
        };

        //if let Some(row) = parse_line(line) {

        //}
    }

    if let Some(mut row) = latest_parsed_row.take() {
        batch.push(row);
    }
    send.send(batch).unwrap();

    println!("Reading {i} lines took {:.2?}", now.elapsed());
}

fn parse_datetime(date: &str) -> Option<i64> {
    let (y, rest) = date.split_once("-")?;
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

    let time_unixtime = NaiveDate::from_ymd_opt(y, m, d)?.and_hms_milli_opt(h, min, s, ms)?;
    let time_unixtime = time_unixtime.timestamp_millis();

    Some(time_unixtime)
}

fn getbom(path: &str) -> Bom {
    let mut file = File::open(path).unwrap();
    Bom::from(&mut file)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_line_2() {
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
            vec![],
        );

        let a = parser.parse_line("2023-12-04 01:12:30,690  DEBUG [] [  24] CA.Core\\WebProxy\\TcpConnection\\TcpConnection.cs(73),  Open <> - Setting up secure connection [73fc :: 95bf]");
        let a = parser.parse_line("2024-01-11 02:40:52,860  DEBUG [0.1.Facades-56.WindowsClient-0.1686.] [  25] Axis.MediaStreaming\\BufferingDataSpanReader.cs(80),  Start <6518> - Start");

        dbg!(a);
        parser.parse_line("2023-12-08 09:45:01,199  INFO  [0.1.Facades-38.WindowsClient-0.5969.32.] [ 489] Server\\Common\\DatabaseHandling\\Private\\FirebirdService.cs(481),  DoGbakBackup <> - gbak:    writing privilege for user SYSDBA");

        todo!();
    }

    #[test]
    fn test_parse_line() {
        let line = parse_line("2023-12-04 01:12:30,690  DEBUG [] [  24] CA.Core\\WebProxy\\TcpConnection\\TcpConnection.cs(73),  Open <> - Setting up secure connection [73fc :: 95bf]".into()).unwrap();

        // assert_eq!(line.time(), "2023-12-04 01:12:30,690");
        assert_eq!(line.level, 2);
        assert_eq!(line.context(), "");
        assert_eq!(line.thread(), "  24");
        assert_eq!(line.file(), "TcpConnection.cs(73)");
        assert_eq!(line.method(), "Open");
        assert_eq!(line.object(), "");
        assert_eq!(
            line.message(),
            "Setting up secure connection [73fc :: 95bf]"
        );
    }
}
