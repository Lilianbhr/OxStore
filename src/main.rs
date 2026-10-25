use anyhow::Result;

mod protocol;
mod server;
mod storage;

fn main() -> Result<()> {
    let serv = server::Server::new(
        "server_data/data.log".to_string(),
        "server_data/data.wal".to_string(),
    )?;
    serv.start()?;

    Ok(())
}
