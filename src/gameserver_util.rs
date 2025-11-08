use tokio::process::Command;

pub fn start_game_server() -> Result<tokio::process::Child, std::io::Error> {
    let child = Command::new("/root/7DaysToDieServer/startserver.sh")
        .arg("-configfile=serverconfig.xml").spawn()?;
    Ok(child)
}

pub fn start_folk_game_server() -> Result<tokio::process::Child, std::io::Error> {
    let child = Command::new("C:\\Users\\89396\\projects\\game_master\\target\\debug\\folk_server.exe")
        .arg("-configfile=serverconfig.xml").spawn()?;
    Ok(child)
}