use tokio::process::Command;

pub fn start_game_server() -> Result<tokio::process::Child, std::io::Error> {
    let child = Command::new("/root/7DaysToDieServer/startserver.sh")
        .arg("-configfile=serverconfig.xml").spawn()?;
    Ok(child)
}