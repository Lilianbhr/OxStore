use anyhow::Result;

mod protocol;
mod storage;

fn main() -> Result<()> {
    println!("Hello, world!");

    let mut logger = storage::LogManager::new("logs.log".to_string())?;
    logger.log(
        &protocol::Command::Set {
            key: "Lilian".to_string(),
            value: "cool".to_string(),
        },
        &protocol::Answer::Ok,
    )?;
    logger.log(
        &protocol::Command::Get {
            key: "Lilian".to_string(),
        },
        &protocol::Answer::Found {
            value: "cool".to_string(),
        },
    )?;
    logger.log(
        &protocol::Command::Del {
            key: "Lilian".to_string(),
        },
        &protocol::Answer::NotFound,
    )?;
    logger.log_error(
        &"not a command at all".to_string(),
        &protocol::CommandError::InvalidCommandName(
            "valid names are : set, get or del".to_string(),
        ),
    )?;

    let mut wal_manager = storage::WalManager::new("data.wal".to_string())?;
    wal_manager.wal(&protocol::Command::Set {
        key: "nom".to_string(),
        value: "Bouhier".to_string(),
    })?;

    let mem = wal_manager.build_live_memory()?;
    println!("{:?}", mem);

    Ok(())
}
