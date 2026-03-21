use anyhow::Result;
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use tar::Archive;
use tracing::info;

pub fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    info!("Extracting {:?} to {:?}", archive_path, dest_dir);

    fs::create_dir_all(dest_dir)?;

    let file = File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    let entries = archive.entries()?;

    for entry in entries {
        let mut entry = entry?;
        entry.unpack_in(dest_dir)?;
    }

    info!("Extraction complete");
    Ok(())
}

#[allow(dead_code)]
pub fn extract_tar_xz(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    info!("Extracting {:?} to {:?}", archive_path, dest_dir);

    fs::create_dir_all(dest_dir)?;

    let file = File::open(archive_path)?;
    let decoder = xz2::read::XzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    let entries = archive.entries()?;

    for entry in entries {
        let mut entry: tar::Entry<xz2::read::XzDecoder<File>> = entry?;
        entry.unpack_in(dest_dir)?;
    }

    info!("Extraction complete");
    Ok(())
}

#[allow(dead_code)]
pub fn create_tar_xz(source_dir: &Path, output_path: &Path) -> Result<()> {
    info!(
        "Creating tar.xz archive from {:?} to {:?}",
        source_dir, output_path
    );

    let file = File::create(output_path)?;
    let encoder = xz2::write::XzEncoder::new(file, 9);
    let mut builder = tar::Builder::new(encoder);

    add_dir_contents(&mut builder, source_dir, source_dir)?;

    builder.finish()?;

    info!("Archive created: {:?}", output_path);
    Ok(())
}

fn add_dir_contents(
    builder: &mut tar::Builder<xz2::write::XzEncoder<File>>,
    dir: &Path,
    base: &Path,
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path_buf = entry.path();
        let name = path_buf
            .strip_prefix(base)
            .unwrap_or(path_buf.as_path())
            .to_path_buf();

        if path_buf.is_file() {
            builder.append_path_with_name(&path_buf, &name)?;
        } else if path_buf.is_dir() {
            builder.append_dir(&name, &path_buf)?;
            add_dir_contents(builder, &path_buf, base)?;
        }
    }
    Ok(())
}
