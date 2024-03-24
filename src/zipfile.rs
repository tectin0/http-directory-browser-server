use std::{
    fs::{self, File},
    io::{Read, Seek, Write},
};

use std::path::Path;

use zip::{result::ZipError, write::FileOptions};

use walkdir::WalkDir;

use walkdir::DirEntry;

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {path:?} as {name:?} ...");
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            println!("adding dir {path:?} as {name:?} ...");
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}

pub fn zip(source: &str, destination: &str) -> anyhow::Result<()> {
    log::debug!("Zipping folder: {} to {}", source, destination);

    if !Path::new(source).is_dir() {
        return Err(ZipError::FileNotFound.into());
    }

    let path = Path::new(destination);

    if path.exists() {
        std::fs::remove_file(path)?;
    }

    fs::create_dir_all(path.parent().unwrap())?;

    let file = File::create(path).unwrap();

    let walkdir = WalkDir::new(source);
    let it = walkdir.into_iter();

    zip_dir(
        &mut it.filter_map(|e| e.ok()),
        source,
        file,
        zip::CompressionMethod::Bzip2,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_zip_folder() {
        let source = "test_files";
        let destination = "archive.zip";
        let result = zip(source, destination);
        assert!(result.is_ok());

        fs::remove_file(destination).unwrap();
    }
}
