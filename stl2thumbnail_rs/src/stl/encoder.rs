use anyhow::Result;
use image::Delay;
use std::{path::Path, time::Duration};

use crate::picture::Picture;

pub fn encode_gif<P: AsRef<Path>>(path: P, pictures: &[Picture]) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut encoder = image::codecs::gif::GifEncoder::new(file);
    encoder.set_repeat(image::codecs::gif::Repeat::Infinite)?;

    let animation_frames: Vec<_> = pictures
        .iter()
        .map(|pic| {
            image::Frame::from_parts(
                pic.img_buf().clone(),
                0,
                0,
                Delay::from_saturating_duration(Duration::from_millis(6)),
            )
        })
        .collect();

    encoder.encode_frames(animation_frames.into_iter())?;

    Ok(())
}
