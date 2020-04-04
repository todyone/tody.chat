// TODO: Convert all that to a static constant (not necessary to close for server), or use `Arc`
// TODO: Move to a separate crate with macro `assets!`
use anyhow::Error;
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::Read;
use tar::Archive;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("wrong assets format")]
    WrongFormat,
}

const ASSETS: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ui.tar.gz"));

pub type Assets = HashMap<String, Vec<u8>>;

pub fn read_assets() -> Result<Assets, Error> {
    let tar = GzDecoder::new(ASSETS);
    let mut archive = Archive::new(tar);
    let mut files = Assets::new();
    for entry in archive.entries()? {
        let mut entry = entry?;
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        if data.len() > 0 {
            let name = entry
                .path()?
                .to_str()
                .ok_or(AssetError::WrongFormat)?
                .to_owned();
            log::trace!("Register asset file: {}", name);
            files.insert(name, data);
        }
    }
    Ok(files)
}
