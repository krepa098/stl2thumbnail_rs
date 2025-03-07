use picture::Color;
use std::time::Duration;

pub mod ffi;
pub mod gcode;
pub mod picture;
pub mod stl;
pub mod threemf;

pub struct Settings {
    pub verbose: bool,
    pub lazy: bool,
    pub recalculate_normals: bool,
    pub turntable: bool,
    pub size_hint: bool,
    pub grid: bool,
    pub cam_elevation: f32,
    pub cam_azimuth: f32,
    pub timeout: Option<Duration>,
    pub background_color: Color,
}
