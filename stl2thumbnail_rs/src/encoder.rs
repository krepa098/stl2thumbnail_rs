use anyhow::Result;
use image::Delay;
use std::time::Duration;

use crate::picture::Picture;

pub fn encode_gif(path: &str, pictures: &[Picture]) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut encoder = image::codecs::gif::GifEncoder::new(file);
    encoder.set_repeat(image::codecs::gif::Repeat::Infinite)?;

    let animation_frames: Vec<_> = pictures
        .iter()
        .map(Picture::create_image)
        .map(|img| {
            image::Frame::from_parts(
                img.into(),
                0,
                0,
                Delay::from_saturating_duration(Duration::from_millis(6)),
            )
        })
        .collect();

    encoder.encode_frames(animation_frames.into_iter())?;

    Ok(())
}
