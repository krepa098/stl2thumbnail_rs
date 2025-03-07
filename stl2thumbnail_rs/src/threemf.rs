use anyhow::{bail, Result};
use std::{
    io::{Read, Seek},
    path::Path,
};

use crate::picture::Picture;

pub fn extract_preview_from_file<P: AsRef<Path>>(filename: P) -> Result<Picture> {
    let file = std::fs::File::open(filename)?;
    extract_preview(file)
}

pub fn extract_preview<R>(r: R) -> Result<Picture>
where
    R: Read + Seek,
{
    let mut archive = zip::ZipArchive::new(r)?;

    // depending on the slicer, the thumbnail gets stored in different locations
    //
    // Prusa, Cura: 'Metadata/thumbnail.png'
    // Bambu, Orca: 'Metadata/plate_<plate-number>.png'
    //
    // Prusa slicer generates a single thumbnail for all build plates
    // Bambu slicer generates one thumbnail per build plate (we show the thumbnail of the first build plate)

    for filename in ["Metadata/thumbnail.png", "Metadata/plate_1.png"] {
        if let Ok(mut thumbnail) = archive.by_name(filename) {
            let mut buffer = vec![];
            thumbnail.read_to_end(&mut buffer)?;
            let image = image::load_from_memory(&buffer)?;

            return Ok(Picture::from_img_buffer(image.to_rgba8()));
        }
    }

    bail!("Cannot find thumbnail in 3mf")
}

#[cfg(test)]
mod test {
    use super::*;

    static PRUSA_TEST_FILE: &[u8] = include_bytes!("../test_models/prusa_test.3mf");
    static BAMBU_TEST_FILE: &[u8] = include_bytes!("../test_models/bambu_test.3mf");

    #[test]
    pub fn test_extract_preview() {
        let cursor = std::io::Cursor::new(PRUSA_TEST_FILE);
        let preview = extract_preview(cursor);

        assert!(preview.is_ok());

        let cursor = std::io::Cursor::new(BAMBU_TEST_FILE);
        let preview = extract_preview(cursor);

        assert!(preview.is_ok());
    }
}
