use anyhow::Result;
use image::{Pixel, RgbaImage};
use std::convert::From;
use std::i32;

use crate::mesh::{Vec2, Vec4};
use std::ops::{Add, Mul};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn alpha(&self, a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: (self.a as f32 * a) as u8,
        }
    }

    pub fn over(&self, b: Color) -> Self {
        // draw self over b
        // Porter-Duff algorithm
        let alpha_a = self.a as f32 / 255.0;
        let alpha_b = b.a as f32 / 255.0;
        let alpha_c = alpha_a + (1.0 - alpha_a) * alpha_b;

        let mut new_p = self * (alpha_a / alpha_c) + b * (((1.0 - alpha_a) * alpha_b) / alpha_c);
        new_p.a = (alpha_c * 255.0) as u8;
        new_p
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            r: (self.r as f32 * rhs) as u8,
            g: (self.g as f32 * rhs) as u8,
            b: (self.b as f32 * rhs) as u8,
            a: self.a,
        }
    }
}

impl Mul<f32> for &Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        *self * rhs
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            r: (self.r as i32 + rhs.r as i32) as u8,
            g: (self.g as i32 + rhs.g as i32) as u8,
            b: (self.b as i32 + rhs.b as i32) as u8,
            a: (self.a as i32 + rhs.a as i32) as u8,
        }
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(rgba: (u8, u8, u8, u8)) -> Self {
        Self {
            r: rgba.0,
            g: rgba.1,
            b: rgba.2,
            a: rgba.3,
        }
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    fn from(rgba: (f32, f32, f32, f32)) -> Self {
        Self {
            r: (rgba.0.clamp(0.0, 1.0) * 255.0) as u8,
            g: (rgba.1.clamp(0.0, 1.0) * 255.0) as u8,
            b: (rgba.2.clamp(0.0, 1.0) * 255.0) as u8,
            a: (rgba.3.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }
}

impl From<&str> for Color {
    fn from(rgba: &str) -> Self {
        assert_eq!(rgba.len(), 8, "expected format: 'RRGGBBAA'");

        Self {
            r: i32::from_str_radix(&rgba[0..2], 16).unwrap() as u8,
            g: i32::from_str_radix(&rgba[2..4], 16).unwrap() as u8,
            b: i32::from_str_radix(&rgba[4..6], 16).unwrap() as u8,
            a: i32::from_str_radix(&rgba[6..8], 16).unwrap() as u8,
        }
    }
}

impl From<&Vec4> for Color {
    fn from(vec: &Vec4) -> Self {
        Self {
            r: (vec.x.clamp(0.0, 1.0) * 255.0) as u8,
            g: (vec.y.clamp(0.0, 1.0) * 255.0) as u8,
            b: (vec.z.clamp(0.0, 1.0) * 255.0) as u8,
            a: (vec.w.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }
}

#[derive(Debug)]
pub struct Picture {
    inner: RgbaImage,
}

impl Picture {
    pub fn new(width: u32, height: u32) -> Self {
        let mut pic = Picture {
            inner: RgbaImage::new(width, height),
        };

        pic.fill(&(0, 0, 0, 255).into());
        pic
    }

    pub fn from_img_buffer(img_buf: RgbaImage) -> Self {
        Picture { inner: img_buf }
    }

    pub fn stride(&self) -> u32 {
        self.inner.width() * self.depth()
    }

    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    pub fn height(&self) -> u32 {
        self.inner.height()
    }

    pub fn depth(&self) -> u32 {
        4
    }

    pub fn data(&self) -> &[u8] {
        &self.inner
    }

    pub fn to_bgra(&self) -> Vec<u8> {
        let mut bgra_img = RgbaImage::new(self.width(), self.height());

        for (rgba, bgra) in self.inner.pixels().zip(bgra_img.pixels_mut()) {
            bgra.channels_mut()[0] = rgba.channels()[2];
            bgra.channels_mut()[1] = rgba.channels()[1];
            bgra.channels_mut()[2] = rgba.channels()[0];
        }

        bgra_img.to_vec()
    }

    pub fn data_as_boxed_slice(&self) -> Box<[u8]> {
        self.data().to_vec().into_boxed_slice()
    }

    pub fn fill(&mut self, rgba: &Color) {
        for pixel in self.inner.pixels_mut() {
            pixel.channels_mut()[0] = rgba.r;
            pixel.channels_mut()[1] = rgba.g;
            pixel.channels_mut()[2] = rgba.b;
            pixel.channels_mut()[3] = rgba.a;
        }
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, rgba: &Color) {
        // Bresenham's line algorithm
        let mut x = x0;
        let mut y = y0;

        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.set(x as u32, y as u32, rgba);
            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn thick_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, rgba: &Color, width: f32) {
        // Anti-aliased thick line
        // Ref: http://members.chello.at/~easyfilter/bresenham.html
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        let ed = if dx + dy == 0 {
            1.0
        } else {
            ((dx * dx + dy * dy) as f32).sqrt()
        };
        let mut x2;
        let mut y2;
        let mut e2;

        let wd = (width + 1.0) / 2.0;
        loop {
            let a = 1.0 - ((err - dx + dy).abs() as f32 / ed - wd).max(0.0);
            self.alpha_blend(x0 as u32, y0 as u32, rgba.alpha(a));
            e2 = err;
            x2 = x0;
            if 2 * e2 >= -dx {
                e2 += dy;
                y2 = y0;
                while (e2 as f32) < ed * wd && (y1 != y2 || dx > dy) {
                    e2 += dx;
                    y2 += sy;
                    let a = 1.0 - (e2.abs() as f32 / ed - wd).max(0.0);
                    self.alpha_blend(x0 as u32, y2 as u32, rgba.alpha(a));
                }

                if x0 == x1 {
                    break;
                }

                e2 = err;
                err -= dy;
                x0 += sx;
            }

            if 2 * e2 <= dy {
                e2 = dx - e2;
                while (e2 as f32) < ed * wd && (x1 != x2 || dx < dy) {
                    e2 += dy;
                    x2 += sx;
                    let a = 1.0 - (e2.abs() as f32 / ed - wd).max(0.0);
                    self.alpha_blend(x2 as u32, y0 as u32, rgba.alpha(a));
                }

                if y0 == y1 {
                    break;
                }

                err += dx;
                y0 += sy;
            }
        }
    }

    pub fn set(&mut self, x: u32, y: u32, rgba: &Color) {
        if x >= self.width() || y >= self.height() {
            return;
        }

        self.inner
            .put_pixel(x, y, image::Rgba([rgba.r, rgba.g, rgba.b, rgba.a]));
    }

    pub fn img_buf(&self) -> &RgbaImage {
        &self.inner
    }

    pub fn alpha_blend(&mut self, x: u32, y: u32, rgba: Color) {
        if x >= self.width() || y >= self.height() {
            return;
        }

        // draw a over b
        let b = self.get(x, y);
        let a = rgba;

        self.set(x, y, &a.over(b));
    }

    pub fn get(&self, x: u32, y: u32) -> Color {
        let pixel = self.inner.get_pixel(x, y);

        Color {
            r: pixel.channels()[0],
            g: pixel.channels()[1],
            b: pixel.channels()[2],
            a: pixel.channels()[3],
        }
    }

    pub fn save(&self, path: &str) -> Result<()> {
        self.inner.save_with_format(path, image::ImageFormat::Png)?;

        Ok(())
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width() as f32 / self.height() as f32
    }

    pub fn resize(&mut self, width: u32, height: u32) -> &mut Self {
        let image = image::imageops::resize(&self.inner, width, height, image::imageops::FilterType::Triangle);

        self.inner = image;
        self
    }

    /// Resizes the image to fit within a rectangle of width and height, keeping the aspect ratio
    pub fn resize_keep_aspect_ratio(&mut self, width: u32, height: u32) -> &mut Self {
        let aspect_ratio = self.aspect_ratio();
        let aspect_ratio_inv = 1.0 / aspect_ratio;

        let new_size = if self.width() >= self.height() {
            (width, (width as f32 * aspect_ratio_inv) as u32)
        } else {
            ((height as f32 * aspect_ratio) as u32, height)
        };

        self.resize(new_size.0, new_size.1)
    }

    pub fn stroke_string(&mut self, x: u32, y: u32, s: &str, char_size: f32, rgba: &Color) {
        for (i, c) in s.chars().enumerate() {
            self.stroke_letter(x + i as u32 * (char_size * 0.7 + 6.0) as u32, y, c, char_size, rgba);
        }
    }

    pub fn stroke_letter(&mut self, x: u32, y: u32, c: char, char_size: f32, rgba: &Color) {
        let points = match c {
            '0' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 0.0),
            ],

            '1' => vec![Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0)],

            '2' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 0.5),
                Vec2::new(1.0, 0.5),
                Vec2::new(0.0, 0.5),
                Vec2::new(0.0, 0.5),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 1.0),
            ],

            '3' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 0.5),
                Vec2::new(0.0, 0.5),
            ],

            '4' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 0.5),
                Vec2::new(0.0, 0.5),
                Vec2::new(1.0, 0.5),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
            ],

            '5' => vec![
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 0.5),
                Vec2::new(1.0, 0.5),
                Vec2::new(0.0, 0.5),
                Vec2::new(0.0, 0.5),
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
            ],

            '6' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 0.5),
                Vec2::new(1.0, 0.5),
                Vec2::new(0.0, 0.5),
            ],

            '7' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
            ],

            '8' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 0.5),
                Vec2::new(1.0, 0.5),
            ],

            '9' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 0.5),
                Vec2::new(0.0, 0.5),
                Vec2::new(1.0, 0.5),
            ],

            'x' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],

            'm' => vec![
                Vec2::new(0.0, 0.5),
                Vec2::new(1.0, 0.5),
                Vec2::new(0.0, 0.5),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.5, 0.5),
                Vec2::new(0.5, 1.0),
                Vec2::new(1.0, 0.5),
                Vec2::new(1.0, 1.0),
            ],

            _ => vec![],
        };

        for p in points.chunks(2) {
            let x0 = p[0].x * char_size * 0.7 + x as f32;
            let y0 = p[0].y * char_size + y as f32;
            let x1 = p[1].x * char_size * 0.7 + x as f32;
            let y1 = p[1].y * char_size + y as f32;
            self.thick_line(x0 as i32, y0 as i32, x1 as i32, y1 as i32, rgba, 3.0);
        }
    }

    pub fn fill_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, rgba: &Color) {
        for x in x0.max(0)..=x1.min(self.width() as i32 - 1) {
            for y in y0.max(0)..=y1.min(self.height() as i32 - 1) {
                self.set(x as u32, y as u32, rgba);
            }
        }
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn test_rgba() {
        let rgba: Color = (1.0, 0.5, -1.0, 2.0).into();
        assert_eq!(rgba, (255, 127, 0, 255).into());

        let rgba: Color = "FF00FF00".into();
        assert_eq!(rgba, (255, 0, 255, 0).into());
    }

    #[test]
    fn test_line() {
        let mut pic = Picture::new(512, 512);
        pic.fill(&(1.0, 1.0, 1.0, 1.0).into());

        pic.thick_line(0, 0, 512, 512, &(1.0, 0.0, 0.0, 1.0).into(), 4.0);
        pic.thick_line(0, 0, 256, 512, &(1.0, 0.0, 0.0, 1.0).into(), 4.0);
        pic.thick_line(0, 256, 512, 256, &(1.0, 0.0, 0.0, 1.0).into(), 4.0);
        pic.thick_line(512, 0, 0, 512, &(1.0, 0.0, 0.0, 1.0).into(), 1.0);

        pic.thick_line(0, 256, 512, 256, &(1.0, 0.0, 0.0, 1.0).into(), 1.0);
        pic.thick_line(256, 0, 256, 512, &(1.0, 0.0, 0.0, 1.0).into(), 1.0);

        // plot chars
        for (i, c) in ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'x', 'm']
            .iter()
            .enumerate()
        {
            pic.stroke_letter(100 + i as u32 * 14, 100, *c, 10.0, &"000000FF".into());
        }

        pic.stroke_string(100, 200, "12x55mm", 10.0, &"E6E6E6FF".into());

        pic.save("test.png").unwrap();
    }
}
