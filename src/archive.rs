use std::io::{Read, Seek, Write};
use anyhow::{Context};
use zip::{result::ZipError, write::SimpleFileOptions, CompressionMethod, ZipArchive};

use std::fs::File;
use std::path::{Path};
use walkdir::{DirEntry, WalkDir};

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

pub fn zip(src_dir: &str, dst_file: &str) -> anyhow::Result<()> {
    match doit(Path::new(src_dir), Path::new(dst_file), CompressionMethod::Deflated) {
        Ok(_) => println!("done: {src_dir:?} written to {dst_file:?}"),
        Err(e) => eprintln!("Error: {e:?}"),
    }

    Ok(())
}

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &Path,
    writer: T,
    method: zip::CompressionMethod,
) -> anyhow::Result<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let prefix = Path::new(prefix);
    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(prefix).unwrap();
        let path_as_string = name
            .to_str()
            .map(str::to_owned)
            .with_context(|| format!("{name:?} Is a Non UTF-8 Path"))?;

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {path:?} as {name:?} ...");
            zip.start_file(path_as_string, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            println!("adding dir {path_as_string:?} as {name:?} ...");
            zip.add_directory(path_as_string, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

fn doit(src_dir: &Path, dst_file: &Path, method: zip::CompressionMethod) -> anyhow::Result<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound.into());
    }

    let path = Path::new(dst_file);
    let file = File::create(path).unwrap();

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;

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
    use crate::archive::{unzip, zip};

    #[test]
    fn test_unzip() {
        unzip("C:\\program1\\Data\\Saves\\Navezgane\\MyGame.zip", "C:\\program1\\Data\\Saves\\Navezgane\\MyGame").unwrap();
    }

    #[test]
    fn test_zip() {
        zip("C:\\program1\\Data\\Saves\\Navezgane\\MyGame", "C:\\program1\\Data\\Saves\\Navezgane\\MyGame.zip").unwrap();
    }
}
