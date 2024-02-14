use std::{
    io::{Read, Seek},
    path::Path,
};

use anyhow::{bail, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use byteorder::{LittleEndian, ReadBytesExt};

use crate::picture::Picture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GCodeType {
    Ascii,
    Binary,
}

enum BlockType {
    FileMetadata = 0,
    GCode = 1,
    SlicerMetadata = 2,
    PrinterMetadata = 3,
    PrintMetadata = 4,
    Thumbnail = 5,
}

enum CompressionType {
    None = 0,
    Deflate = 1,
    Heatshrink11_4 = 2,
    Heatshrink12_4 = 3,
}

// ref: https://github.com/prusa3d/libbgcode/blob/main/doc/specifications.md
struct FileHeader {
    magic_number: u32,
    _version: u32,
    checksum_type: u16,
}

impl FileHeader {
    fn is_valid(&self) -> bool {
        self.magic_number.to_ne_bytes() == [b'G', b'C', b'D', b'E']
    }
}

struct Block {
    kind: BlockType,
    compression: CompressionType,
    data: Vec<u8>,
}

impl Block {
    fn uncompressed_data(&self) -> Result<Vec<u8>> {
        match self.compression {
            CompressionType::None => Ok(self.data.clone()),
            CompressionType::Deflate => {
                let mut decompress = flate2::Decompress::new(false);
                let mut decompressed_data = vec![];
                decompress.decompress_vec(&self.data, &mut decompressed_data, flate2::FlushDecompress::None)?;
                Ok(decompressed_data)
            }
            _ => bail!("unimplemented"),
        }
    }
}

fn detect_format(data: &[u8]) -> Result<GCodeType> {
    let mut cursor = std::io::Cursor::new(data);

    let header = read_header(&mut cursor);

    if let Ok(header) = header {
        if header.is_valid() {
            return Ok(GCodeType::Binary);
        }
    }

    Ok(GCodeType::Ascii)
}

pub fn extract_previews_from_file<P: AsRef<Path>>(filename: P) -> Result<Vec<Picture>> {
    let data = std::fs::read(filename)?;

    extract_previews_from_data(&data)
}

pub fn extract_previews_from_data(data: &[u8]) -> Result<Vec<Picture>> {
    match detect_format(data) {
        Ok(GCodeType::Ascii) => extract_previews_ascii(data),
        Ok(GCodeType::Binary) => extract_previews_binary(data),
        _ => bail!("Cannot detect gcode format"),
    }
}

pub fn extract_previews_binary(data: &[u8]) -> Result<Vec<Picture>> {
    let mut pictures = vec![];

    let mut cursor = std::io::Cursor::new(data);

    let header = read_header(&mut cursor)?;

    // skip header
    cursor.seek(std::io::SeekFrom::Start(10))?;

    while let Ok(block) = try_read_thumbnail_block(&header, &mut cursor) {
        if let Some(block) = block {
            if let BlockType::Thumbnail = block.kind {
                let block_data = block.uncompressed_data()?;
                let mut block_reader = std::io::Cursor::new(block_data);

                let mut img_data = vec![];
                block_reader.read_to_end(&mut img_data)?;
                let img = image::load_from_memory(&img_data)?;

                pictures.push(Picture::from_img_buffer(img.to_rgba8()));
            } else {
                println!("not thumbnail");
            }
        }
    }

    Ok(pictures)
}

fn read_header<R>(reader: &mut R) -> Result<FileHeader>
where
    R: Read + Seek,
{
    reader.rewind()?;

    let magic_number = reader.read_u32::<LittleEndian>()?;
    let version = reader.read_u32::<LittleEndian>()?;
    let checksum_type = reader.read_u16::<LittleEndian>()?;

    Ok(FileHeader {
        magic_number,
        _version: version,
        checksum_type,
    })
}

fn try_read_thumbnail_block<R>(file_header: &FileHeader, reader: &mut R) -> Result<Option<Block>>
where
    R: Read + Seek,
{
    // The block section has the following format:
    //
    // Block Header (8 or 12 bytes, depending on compression)
    // Block Parameters (size varies with block type)
    // Block Data (compressed or uncompressed, size given in header)
    // Optional CRC (present if indicated in file header)

    let block_type = reader.read_u16::<LittleEndian>()?;
    let compression = reader.read_u16::<LittleEndian>()?;
    let uncompressed_size = reader.read_u32::<LittleEndian>()?;

    let block_size = if compression == 0 {
        uncompressed_size
    } else {
        reader.read_u32::<LittleEndian>()?
    };

    let compression_type = match compression {
        0 => CompressionType::None,
        1 => CompressionType::Deflate,
        2 => CompressionType::Heatshrink11_4,
        3 => CompressionType::Heatshrink12_4,
        _ => unimplemented!(),
    };

    let block_kind = match block_type {
        0 => BlockType::FileMetadata,
        1 => BlockType::GCode,
        2 => BlockType::SlicerMetadata,
        3 => BlockType::PrinterMetadata,
        4 => BlockType::PrintMetadata,
        5 => BlockType::Thumbnail,
        _ => unimplemented!("{block_type}"),
    };

    // skip parameter sections
    match block_kind {
        BlockType::FileMetadata => {
            reader.read_u16::<LittleEndian>()?;
        }
        BlockType::GCode => {
            reader.read_u16::<LittleEndian>()?;
        }
        BlockType::SlicerMetadata => {
            reader.read_u16::<LittleEndian>()?;
        }
        BlockType::PrinterMetadata => {
            reader.read_u16::<LittleEndian>()?;
        }
        BlockType::PrintMetadata => {
            reader.read_u16::<LittleEndian>()?;
        }
        BlockType::Thumbnail => {
            reader.read_u16::<LittleEndian>()?;
            reader.read_u16::<LittleEndian>()?;
            reader.read_u16::<LittleEndian>()?;
        }
    }

    // read block data section
    let mut buf = vec![0u8; block_size as usize];
    reader.read_exact(&mut buf)?;

    // skip crc if present
    if file_header.checksum_type == 1 {
        // crc32
        reader.read_u32::<LittleEndian>()?;
    }

    // let mut block = None;

    if let BlockType::Thumbnail = block_kind {
        return Ok(Some(Block {
            kind: block_kind,
            compression: compression_type,
            data: buf,
        }));
    }

    Ok(None)
}

pub fn extract_previews_ascii(data: &[u8]) -> Result<Vec<Picture>> {
    let content = String::from_utf8(data.to_vec())?;

    // gcode format
    // ...
    // ; thumbnail begin <width>x<height> <?>
    // ; <base64>
    // ; ...
    // ; thumbnail end
    //
    // the encoded image can be a 'png', 'jpeg' or 'qoi'

    let mut base64_images = vec![];
    let mut base64_image = String::new();

    let mut in_thumbnail_section = false;

    for (i, line) in content.lines().enumerate() {
        let trimmed_line = line.trim();

        if in_thumbnail_section && (trimmed_line.starts_with("; thumbnail end") || !trimmed_line.starts_with(';')) {
            in_thumbnail_section = false;

            if !base64_image.is_empty() {
                base64_images.push(base64_image);
                base64_image = String::new();
            }
        }

        if in_thumbnail_section && trimmed_line.starts_with(';') {
            let (_, base64) = trimmed_line.split_at(1);
            base64_image.push_str(base64.trim());
        }

        if trimmed_line.starts_with("; thumbnail begin") {
            in_thumbnail_section = true;
        }

        // gcode files can be huge we thus avoid scanning the whole file
        // the thumbnail should be at the start of the file
        if i > 2000 {
            break;
        }
    }

    let mut pcitures = vec![];

    for base64_image in base64_images {
        let image_bytes = STANDARD.decode(base64_image)?;

        // try to decode the image (possible formats are 'png', 'jpeg', 'qoi')
        if let Ok(image) = image::load_from_memory(&image_bytes) {
            pcitures.push(Picture::from_img_buffer(image.to_rgba8()));
        }
    }

    // sort by size (ascending order)
    pcitures.sort_by_key(|a| a.width() * a.height());

    Ok(pcitures)
}

#[cfg(test)]
mod test {
    use super::*;

    static GCODE_ASCII: &[u8] = include_bytes!("test_models/test_cube.gcode");
    static GCODE_BIN: &[u8] = include_bytes!("test_models/test_cube.bgcode");

    #[test]
    fn test_parser_ascii() {
        let images = extract_previews_ascii(GCODE_ASCII).unwrap();

        assert_eq!(images.len(), 2);
        assert_eq!(images[0].width(), 32);
        assert_eq!(images[1].width(), 400);
    }

    #[test]
    fn test_parser_binary() {
        let images = extract_previews_binary(GCODE_BIN).unwrap();

        assert_eq!(images.len(), 2);
        assert_eq!(images[0].width(), 32);
        assert_eq!(images[1].width(), 400);
    }

    #[test]
    fn test_detect_format() {
        assert_eq!(detect_format(GCODE_BIN).unwrap(), GCodeType::Binary);
        assert_eq!(detect_format(GCODE_ASCII).unwrap(), GCodeType::Ascii);
    }
}
