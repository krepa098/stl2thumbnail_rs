use std::io::{Read, Seek};

use anyhow::Result;
use image::DynamicImage;

pub fn extract_preview_from_file(filename: &str) -> Result<DynamicImage> {
    let file = std::fs::File::open(filename)?;
    extract_preview(file)
}

pub fn extract_preview<R>(r: R) -> Result<DynamicImage>
where
    R: Read + Seek,
{
    let mut archive = zip::ZipArchive::new(r)?;

    let mut thumbnail = archive.by_name("Metadata/thumbnail.png")?;
    let mut buffer = vec![];
    thumbnail.read_to_end(&mut buffer)?;
    let image = image::load_from_memory(&buffer)?;

    Ok(image)
}

#[cfg(test)]
mod test {
    use super::*;

    static TEST_FILE: &[u8] = include_bytes!("test_models/test_cube.3mf");

    #[test]
    pub fn test_extract_preview() {
        let cursor = std::io::Cursor::new(TEST_FILE);
        let preview = extract_preview(cursor);

        assert!(preview.is_ok());
    }
}