use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::DynamicImage;

pub fn extract_previews_from_file(filename: &str) -> Result<Vec<DynamicImage>> {
    let content = std::fs::read_to_string(filename)?;

    extract_previews(&content)
}

pub fn extract_previews(content: &str) -> Result<Vec<DynamicImage>> {
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

    for line in content.lines() {
        let trimmed_line = line.trim();

        if in_thumbnail_section && (trimmed_line.starts_with("; thumbnail end") || !trimmed_line.starts_with(';')) {
            in_thumbnail_section = false;

            if !base64_image.is_empty() {
                base64_images.push(base64_image);
                base64_image = String::new();
            }
        }

        if in_thumbnail_section {
            let (_, base64) = trimmed_line.split_at(1);
            base64_image.push_str(base64.trim());
        }

        if trimmed_line.starts_with("; thumbnail begin") {
            let args = trimmed_line.split_ascii_whitespace();

            in_thumbnail_section = true;

            for a in args {
                dbg!(a);
            }
        }
    }

    let mut images = vec![];

    for base64_image in base64_images {
        let image_bytes = STANDARD.decode(base64_image)?;

        // try to decode the image (possible formats are 'png', 'jpeg', 'qoi')
        if let Ok(image) = image::load_from_memory(&image_bytes) {
            dbg!(image.width(), image.height());

            images.push(image);
        }
    }

    // sort by size (ascending order)
    images.sort_by_key(|a| a.width() * a.height());

    Ok(images)
}

#[cfg(test)]
mod test {
    use super::*;

    static GCODE: &str = include_str!("test_models/test_cube.gcode");

    #[test]
    fn test_parser() {
        let images = extract_previews(GCODE).unwrap();

        assert_eq!(images.len(), 2);
        assert_eq!(images[0].width(), 32);
        assert_eq!(images[1].width(), 400);
    }
}
