use crate::protocol::{Answer, Command};
use chrono::Local;
use std::{
    fs::File,
    io::{Result, Write},
};

pub struct LogManager {
    log_file: File,
}

impl LogManager {
    pub fn new(path: String) -> Result<Self> {
        let file = File::options().append(true).create(true).open(path)?;
        Ok(LogManager { log_file: file })
    }

    pub fn log(&mut self, command: &Command, answer: &Answer) -> Result<()> {
        let date = Local::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let level = match answer {
            Answer::Ok | Answer::Found { value: _ } => "[SUCCESS]",
            Answer::NotFound => "[WARN]",
        };
        let log_line = format!(
            "{} {} {:?} -> {:?}: {}",
            date,
            level,
            command,
            answer,
            answer.format()
        );
        writeln!(self.log_file, "{}", log_line)?;
        Ok(())
    }
}

pub struct WalManager {
    wal_file: File,
}

impl WalManager {
    pub fn new(path: String) -> Result<Self> {
        let file = File::options().append(true).create(true).open(path)?;
        Ok(WalManager { wal_file: file })
    }

    pub fn wal(&mut self, command: &Command) -> Result<()> {
        match command {
            Command::Set { .. } | Command::Del { .. } => {
                writeln!(self.wal_file, "{}", command.format())?;
            }
            _ => {}
        }
        Ok(())
    }
}
