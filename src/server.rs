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

    // TODO: clean unwraps
    fn command_analysis(
        &self,
        command: Result<Command, CommandError>,
        raw_command: &String,
    ) -> Result<Answer, CommandError> {
        match command {
            Err(e) => {
                let log_manager_access = self.log_manager.clone();
                let mut log_manager = log_manager_access.lock().unwrap();
                log_manager.log_error(raw_command, &e).unwrap();
                Err(e)
            }
            Ok(val) => Ok(self.execute_command(val)),
        }
    }

    // TODO: clean Arc cloning + action order for del case + clean unwraps
    fn execute_command(&self, command: Command) -> Answer {
        match &command {
            Command::Set { key, value } => {
                // wal
                let wal_manager_access = self.wal_manager.clone();
                let mut wal_manager = wal_manager_access.lock().unwrap();
                wal_manager.wal(&command).unwrap();

                // memory
                let memory_access = self.memory.clone();
                let mut memory = memory_access.lock().unwrap();
                memory.insert(key.clone(), value.clone());
                let answer = Answer::Ok;

                // log
                let log_manager_access = self.log_manager.clone();
                let mut log_manager = log_manager_access.lock().unwrap();
                log_manager.log(&command, &answer).unwrap();

                answer
            }
            Command::Del { key } => {
                // wal
                let wal_manager_access = self.wal_manager.clone();
                let mut wal_manager = wal_manager_access.lock().unwrap();
                wal_manager.wal(&command).unwrap();

                // memory
                let memory_access = self.memory.clone();
                let mut memory = memory_access.lock().unwrap();
                let res = memory.remove(key);
                let answer = match res {
                    Some(_) => Answer::Ok,
                    None => Answer::NotFound,
                };

                // log
                let log_manager_access = self.log_manager.clone();
                let mut log_manager = log_manager_access.lock().unwrap();
                log_manager.log(&command, &answer).unwrap();

                answer
            }
            Command::Get { key } => {
                // memory
                let memory_access = self.memory.clone();
                let memory = memory_access.lock().unwrap();
                let res = memory.get(key);
                let answer = match res {
                    Some(val) => Answer::Found { value: val.clone() },
                    None => Answer::NotFound,
                };

                // log
                let log_manager_access = self.log_manager.clone();
                let mut log_manager = log_manager_access.lock().unwrap();
                log_manager.log(&command, &answer).unwrap();

                answer
            }
        }
    }
}
