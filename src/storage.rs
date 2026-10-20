use crate::protocol::{Answer, Command, CommandError};
use anyhow::Result;
use chrono::Local;
use std::{
    collections::HashMap,
    fs::File,
    io::{
        BufRead, BufReader, Seek,
        SeekFrom::{End, Start},
        Write,
    },
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

    pub fn log_error(&mut self, raw_command: &String, error: &CommandError) -> Result<()> {
        let date = Local::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let level = "[ERROR]";
        let log_line = format!("{} {} <{}> -> {}", date, level, raw_command, error);
        writeln!(self.log_file, "{}", log_line)?;
        Ok(())
    }
}

pub struct WalManager {
    wal_file: File,
}

impl WalManager {
    pub fn new(path: String) -> Result<Self> {
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        Ok(WalManager { wal_file: file })
    }

    pub fn wal(&mut self, command: &Command) -> Result<()> {
        self.wal_file.seek(End(0))?;
        match command {
            Command::Set { .. } | Command::Del { .. } => {
                writeln!(self.wal_file, "{}", command.format())?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn build_live_memory(&mut self) -> Result<HashMap<String, String>> {
        let mut res: HashMap<String, String> = HashMap::new();
        self.wal_file.seek(Start(0))?;
        let buf_reader = BufReader::new(&self.wal_file);

        for result_line in buf_reader.lines() {
            let line = result_line?;
            let associated_command = Command::parse(line)?;

            match associated_command {
                Command::Set { key, value } => {
                    res.insert(key, value);
                }
                Command::Del { key } => {
                    res.remove(&key);
                }
                _ => {}
            }
        }

        Ok(res)
    }
}
