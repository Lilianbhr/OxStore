use anyhow::Result;
use std::{fs, io, path};

mod protocol;
mod server;
mod storage;

fn main() -> Result<()> {
    let mut data_dir = String::new();
    println!("In which directory should the server data be stored ?");
    io::stdin().read_line(&mut data_dir)?;
    let mut clean_data_dir = data_dir.trim().to_string();
    clean_data_dir += "/";
    println!("{clean_data_dir}");

    let path = path::Path::new(&clean_data_dir);
    fs::create_dir_all(path)?;

    let mut log_path = String::new();
    if let Some(log_path_str) = path.join("data.log").to_str() {
        log_path = log_path_str.to_string();
    }
    let mut wal_path = String::new();
    if let Some(wal_path_str) = path.join("data.wal").to_str() {
        wal_path = wal_path_str.to_string();
    }
    let serv = server::Server::new(log_path, wal_path)?;
    serv.start()?;

    Ok(())
}
