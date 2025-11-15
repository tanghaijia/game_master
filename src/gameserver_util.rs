use tokio::process::Command;
use crate::const_value::{SEVENDAYS_EXE_PATH, SEVENDAYS_LOG_PATH, SEVENDAYS_SERVER_PATH};

pub fn start_game_server() -> Result<tokio::process::Child, std::io::Error> {
    let child = Command::new(SEVENDAYS_EXE_PATH)
        .arg("-logfile")
        .arg(SEVENDAYS_LOG_PATH)
        .arg("-quit")
        .arg("-batchmode")
        .arg("-nographics")
        .arg("-dedicated")
        .arg("-configfile=serverconfig.xml")
        .env("LD_LIBRARY_PATH", SEVENDAYS_SERVER_PATH)
        .current_dir(SEVENDAYS_SERVER_PATH)
        .spawn()?;

    Ok(child)
}

pub fn start_folk_game_server() -> Result<tokio::process::Child, std::io::Error> {
    let child = Command::new("C:\\Users\\89396\\projects\\game_master\\target\\debug\\folk_server.exe")
        .arg("-configfile=serverconfig.xml").spawn()?;
    Ok(child)
}