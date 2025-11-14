use std::fs::File;
use std::path::Path;
use zip::ZipArchive;


pub fn unzip(archive_path: &str, extract_to: &str) -> anyhow::Result<()> {
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = Path::new(extract_to).join(file.name());

        let parent = outpath.parent().expect("no parent");
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

async fn async_unzip(archive_path: String, extract_to: String) -> anyhow::Result<()> {
    let a = archive_path.clone();
    let e = extract_to.clone();
    tokio::task::spawn_blocking(move || {
        let _ = unzip(a.as_str(), e.as_str());
    }).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::archive::unzip;

    #[test]
    fn test_unzip() {
        unzip("C:\\Users\\89396\\Downloads\\MyGame.zip", "C:\\program1\\Data\\Saves\\Navezgane").unwrap();
    }
}
