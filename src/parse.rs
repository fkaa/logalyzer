use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::ops::Range;
use std::sync::mpsc;
use std::time::Instant;

use chrono::NaiveDate;
use log::warn;
use ratatui::layout::Constraint;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use unicode_bom::Bom;

use crate::config::{LogFormatConfiguration, LogFormatInstruction};

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
pub struct Row {
    pub line: String,
    pub values: SmallVec<[ParsedRowValue; 10]>,
}

#[derive(Clone, Debug)]
pub enum ParsedRowValue {
    String { start: u32, end: i32 },
    Date(i64),
    Integer(i64),
}

impl From<LogFormatConfiguration> for Parser {
    fn from(val: LogFormatConfiguration) -> Self {
        let mut instructions = Vec::new();
        let mut columns = Vec::new();

        for syn in val.syntax {
            match syn {
                LogFormatInstruction::EmitDate { name, width } => {
                    instructions.push(ParserInstruction::EmitDate);
                    columns.push(ColumnDefinition::date(
                        name,
                        Constraint::Length(width as u16),
                    ));
                }
                LogFormatInstruction::EmitString { name, width } => {
                    instructions.push(ParserInstruction::EmitString);
                    columns.push(ColumnDefinition::string(
                        name,
                        Constraint::Length(width as u16),
                    ));
                }
                LogFormatInstruction::EmitEnumeration {
                    name,
                    width,
                    enumerations,
                } => {
                    instructions.push(ParserInstruction::EmitEnumeration(enumerations.clone()));
                    columns.push(ColumnDefinition::enumeration(
                        name,
                        Constraint::Length(width as u16),
                        enumerations,
                    ));
                }
                LogFormatInstruction::EmitRemainder { name, width } => {
                    instructions.push(ParserInstruction::EmitRemainder);

                    columns.push(ColumnDefinition::string(
                        name,
                        if width < 0 {
                            Constraint::Percentage(100)
                        } else {
                            Constraint::Length(width as u16)
                        },
                    ));
                }
                LogFormatInstruction::Begin => instructions.push(ParserInstruction::Begin),
                LogFormatInstruction::Skip(amt) => instructions.push(ParserInstruction::Skip(amt)),
                LogFormatInstruction::SkipUntilChar(c) => {
                    instructions.push(ParserInstruction::SkipUntilChar(c))
                }
                LogFormatInstruction::SkipUntilString(s) => {
                    instructions.push(ParserInstruction::SkipUntilString(s))
                }
            }
        }

        Parser {
            instructions,
            columns,
        }
    }
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

    pub fn parse_line(&self, line: String) -> Result<Row, (String, String)> {
        use ParserInstruction::*;

        let mut values = SmallVec::new();

        let mut index = 0usize;
        let mut begin_index = 0;

        for i in &self.instructions {
            match i {
                EmitDate => {
                    let date_str = &line[begin_index..index];
                    let date = parse_datetime(date_str)
                        .ok_or_else(|| (line.clone(), format!("Invalid datetime {date_str}")))?;

                    values.push(ParsedRowValue::Date(date));
                }
                EmitString => {
                    values.push(ParsedRowValue::String {
                        start: begin_index as _,
                        end: index as _,
                    });
                }
                EmitEnumeration(enums) => {
                    let value = &line[begin_index..index];
                    let idx = enums
                        .iter()
                        .position(|e| e == value)
                        .ok_or_else(|| (line.clone(), format!("Unknown enum {value}")))?;
                    values.push(ParsedRowValue::Integer(idx as _));
                }
                EmitRemainder => {
                    values.push(ParsedRowValue::String {
                        start: begin_index as _,
                        end: -1,
                    });
                }
                Begin => begin_index = index,
                Skip(amount) => index += *amount as usize,
                SkipUntilChar(ch) => index += line[index..].find(*ch).unwrap(),
                SkipUntilString(text) => index += line[index..].find(&*text).unwrap(),
            }
        }

        Ok(Row { line, values })
    }
}

#[derive(Clone, Serialize, Deserialize)]
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
    send: mpsc::SyncSender<SmallVec<[Row; 16]>>,
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

    let mut batch = SmallVec::new();

    let now = Instant::now();
    let mut i = 0;
    let mut latest_parsed_row = None;

    for line in reader.lines() {
        let line = line.unwrap();
        match parser.parse_line(line) {
            Ok(row) => {
                if let Some(last_row) = latest_parsed_row.take() {
                    batch.push(last_row);
                    if batch.len() >= batch_size {
                        let old_vec = std::mem::replace(&mut batch, SmallVec::new());
                        send.send(old_vec).unwrap();
                    }
                }
                latest_parsed_row = Some(row);

                i += 1;
            }
            Err((line, e)) => {
                warn!("Error while parsing line: {e}");
                if let Some(mut row) = latest_parsed_row.take() {
                    row.line += &line;
                }
            }
        };

        //if let Some(row) = parse_line(line) {

        //}
    }

    if let Some(row) = latest_parsed_row.take() {
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
mod test {}
