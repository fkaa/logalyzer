use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct LogFormatConfiguration {
    pub title: String,
    pub syntax: Vec<LogFormatInstruction>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum LogFormatInstruction {
    EmitDate {
        name: String,
        width: i32,
    },
    EmitString {
        name: String,
        width: i32,
    },
    EmitEnumeration {
        name: String,
        width: i32,
        enumerations: Vec<String>,
    },
    EmitRemainder {
        name: String,
        width: i32,
    },
    Begin,
    Skip(u16),
    SkipUntilChar(char),
    SkipUntilString(String),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_config() {
        use LogFormatInstruction::*;

        let cfg = LogFormatConfiguration {
            title: "Log4Net (AXIS)".into(),
            syntax: vec![
                Begin,
                Skip(23),
                // Date
                EmitDate {
                    name: "Date".into(),
                    width: 23,
                },
                Skip(2),
                Begin,
                SkipUntilChar(' '),
                // Level
                EmitEnumeration {
                    name: "Level".into(),
                    width: 5,
                    enumerations: vec![
                        "TRACE".into(),
                        "DEBUG".into(),
                        "INFO".into(),
                        "WARN".into(),
                        "ERROR".into(),
                        "FATAL".into(),
                    ],
                },
                SkipUntilChar('['),
                Skip(1),
                Begin,
                SkipUntilChar(']'),
                // Context
                EmitString {
                    name: "Context".into(),
                    width: 5,
                },
                SkipUntilChar('['),
                Skip(1),
                Begin,
                SkipUntilChar(']'),
                // Thread
                EmitString {
                    name: "Thread".into(),
                    width: 5,
                },
                Skip(2),
                Begin,
                SkipUntilChar(','),
                // File
                EmitString {
                    name: "File".into(),
                    width: 5,
                },
                Skip(3),
                Begin,
                SkipUntilString(" <".into()),
                // Method
                EmitString {
                    name: "Method".into(),
                    width: 5,
                },
                Skip(2),
                Begin,
                SkipUntilChar('>'),
                // Object
                EmitString {
                    name: "Object".into(),
                    width: 5,
                },
                SkipUntilChar('-'),
                Skip(2),
                Begin,
                // Message
                EmitRemainder {
                    name: "Message".into(),
                    width: 5,
                },
            ],
        };

        println!("{}", toml::to_string(&cfg).unwrap());
        todo!()
    }
}
