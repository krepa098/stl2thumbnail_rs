pub mod aabb;
pub mod encoder;
pub mod mesh;
pub mod parser;
pub mod rasterbackend;
pub mod zbuffer;

use std::path::Path;

use self::{
    encoder::encode_gif,
    mesh::{Triangle, Vec3},
    rasterbackend::RasterBackend,
};
use crate::{picture::Picture, Settings};
use anyhow::Result;

pub fn render_stl<P: AsRef<Path>>(
    width: u32,
    height: u32,
    mesh: impl IntoIterator<Item = Triangle> + Copy,
    path: P,
    settings: &Settings,
) -> Result<()> {
    if settings.turntable {
        render_stl_turntable_animation(width, height, mesh, path, settings)
    } else {
        render_stl_still(width, height, mesh, path, settings)
    }
}

pub fn render_stl_still<P: AsRef<Path>>(
    width: u32,
    height: u32,
    mesh: impl IntoIterator<Item = Triangle> + Copy,
    path: P,
    settings: &Settings,
) -> Result<()> {
    let mut backend = RasterBackend::new(width, height);
    backend.render_options.grid_visible = settings.grid;

    backend.render_options.view_pos = Vec3::new(
        settings.cam_azimuth.to_radians().cos(),
        settings.cam_azimuth.to_radians().sin(),
        -settings.cam_elevation.to_radians().tan(),
    );

    let (aabb, scale) = backend.fit_mesh_scale(mesh);
    backend.render_options.zoom = 1.05;
    backend.render_options.draw_size_hint = settings.size_hint;

    backend.render(mesh, scale, &aabb, settings.timeout).save(path)?;

    Ok(())
}

pub fn render_stl_turntable_animation<P: AsRef<Path>>(
    width: u32,
    height: u32,
    mesh: impl IntoIterator<Item = Triangle> + Copy,
    path: P,
    settings: &Settings,
) -> Result<()> {
    let mut backend = RasterBackend::new(width, height);
    backend.render_options.grid_visible = settings.grid;
    let mut pictures: Vec<Picture> = Vec::new();

    backend.render_options.view_pos = Vec3::new(1.0, 1.0, -settings.cam_elevation.to_radians().tan());
    let (aabb, scale) = backend.fit_mesh_scale(mesh);
    backend.render_options.zoom = 1.05;
    backend.render_options.draw_size_hint = settings.size_hint;

    for i in 0..45 {
        let angle = (8.0 * i as f32).to_radians();
        backend.render_options.view_pos =
            Vec3::new(angle.cos(), angle.sin(), -settings.cam_elevation.to_radians().tan());
        pictures.push(backend.render(mesh, scale, &aabb, settings.timeout));
    }

    encode_gif(path, pictures.as_slice())?;

    Ok(())
}
