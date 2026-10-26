use crate::{
    protocol::{Answer, Command, CommandError},
    storage::{LogManager, WalManager},
};
use anyhow::Result;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

pub struct Server {
    memory: Arc<Mutex<HashMap<String, String>>>,
    log_manager: Arc<Mutex<LogManager>>,
    wal_manager: Arc<Mutex<WalManager>>,
}

impl Server {
    pub fn new(log_path: String, wal_path: String) -> Result<Self> {
        let log_manager = LogManager::new(log_path)?;
        let mut wal_manager = WalManager::new(wal_path)?;
        let memory = wal_manager.build_live_memory()?;
        Ok(Self {
            memory: Arc::new(Mutex::new(memory)),
            log_manager: Arc::new(Mutex::new(log_manager)),
            wal_manager: Arc::new(Mutex::new(wal_manager)),
        })
    }

    pub fn start(&self) -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        println!(
            "server started. It reads and writes data on {:?}.",
            listener.local_addr()?
        );

        thread::scope(|s| {
            loop {
                let (stream, _addr) = listener.accept().unwrap();
                s.spawn(move || {
                    self.client_handle(stream).unwrap();
                });
            }
        });

        Ok(())
    }

    fn client_handle(&self, mut stream: TcpStream) -> Result<()> {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut raw_command = String::new();

        loop {
            let bytes = reader.read_line(&mut raw_command)?;
            if bytes == 0 {
                break;
            }
            let command = Command::parse(&raw_command.trim());
            let server_answer = self.command_analysis(command, &raw_command);
            match server_answer {
                Err(e) => {
                    writeln!(stream, "{}", e)?;
                }
                Ok(val) => {
                    writeln!(stream, "{}", val.format())?;
                }
            }
            raw_command.clear();
        }

        Ok(())
    }

    fn command_analysis(
        &self,
        command: Result<Command, CommandError>,
        raw_command: &String,
    ) -> Result<Answer> {
        match command {
            Err(e) => {
                let mut log_manager = self.log_manager.lock().unwrap();
                log_manager.log_error(raw_command, &e)?;
                Err(e.into())
            }
            Ok(val) => self.execute_command(val),
        }
    }

    fn execute_command(&self, command: Command) -> Result<Answer> {
        match &command {
            Command::Set { key, value } => {
                // wal
                let mut wal_manager = self.wal_manager.lock().unwrap();
                wal_manager.wal(&command)?;

                // memory
                let mut memory = self.memory.lock().unwrap();
                memory.insert(key.clone(), value.clone());
                let answer = Answer::Ok;

                // log
                let mut log_manager = self.log_manager.lock().unwrap();
                log_manager.log(&command, &answer)?;

                Ok(answer)
            }
            Command::Del { key } => {
                // wal
                let mut wal_manager = self.wal_manager.lock().unwrap();

                // memory
                let mut memory = self.memory.lock().unwrap();
                let res = memory.remove(key);
                let answer = match res {
                    Some(_) => {
                        wal_manager.wal(&command)?;
                        Answer::Ok
                    }
                    None => Answer::NotFound,
                };

                // log
                let mut log_manager = self.log_manager.lock().unwrap();
                log_manager.log(&command, &answer)?;

                Ok(answer)
            }
            Command::Get { key } => {
                // memory
                let memory = self.memory.lock().unwrap();
                let res = memory.get(key);
                let answer = match res {
                    Some(val) => Answer::Found { value: val.clone() },
                    None => Answer::NotFound,
                };

                // log
                let mut log_manager = self.log_manager.lock().unwrap();
                log_manager.log(&command, &answer)?;

                Ok(answer)
            }
        }
    }
}
