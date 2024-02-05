mod aabb;
mod encoder;
mod ffi;
mod mesh;
mod parser;
mod picture;
mod rasterbackend;
mod zbuffer;

use anyhow::Result;
use encoder::*;
use mesh::LazyMesh;
use mesh::{Triangle, Vec3};
use parser::Parser;
use picture::Picture;
use rasterbackend::RasterBackend;

use clap::{App, Arg};
use std::time::{Duration, Instant};

struct Settings {
    verbose: bool,
    lazy: bool,
    recalculate_normals: bool,
    turntable: bool,
    size_hint: bool,
    grid: bool,
    cam_elevation: f32,
    cam_azimuth: f32,
    timeout: Option<Duration>,
}

fn main() -> Result<()> {
    let matches = App::new("stl2thumbnail")
        .version(clap::crate_version!())
        .about("Generates thumbnails from STL files")
        .arg(
            Arg::with_name("INPUT")
                .short("i")
                .index(1)
                .long("input")
                .help("Input filename")
                .required(true),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .short("o")
                .index(2)
                .long("output")
                .help("Output filename")
                .required(true),
        )
        .arg(
            Arg::with_name("TURNTABLE")
                .short("t")
                .long("turntable")
                .help("Enables turntable mode"),
        )
        .arg(Arg::with_name("VERBOSE").short("v").long("verbose").help("Be verbose"))
        .arg(
            Arg::with_name("LAZY")
                .short("l")
                .long("lazy")
                .help("Enables low memory usage mode"),
        )
        .arg(
            Arg::with_name("RECALC_NORMALS")
                .short("n")
                .long("normals")
                .help("Always recalculate normals"),
        )
        .arg(
            Arg::with_name("WIDTH")
                .short("w")
                .long("width")
                .takes_value(true)
                .help("Width of the generated image (defaults to 256)"),
        )
        .arg(
            Arg::with_name("HEIGHT")
                .short("h")
                .long("height")
                .takes_value(true)
                .help("Height of the generated image (defaults to 256)"),
        )
        .arg(
            Arg::with_name("SIZE_HINT")
                .short("d")
                .long("dimensions")
                .help("Draws the dimensions underneath the model (requires height of at least 256 pixels)"),
        )
        .arg(
            Arg::with_name("CAM_ELEVATION")
                .long("cam-elevation")
                .takes_value(true)
                .help("The camera's elevation"),
        )
        .arg(
            Arg::with_name("CAM_AZIMUTH")
                .long("cam-azimuth")
                .takes_value(true)
                .help("The camera's azimuth"),
        )
        .arg(
            Arg::with_name("GRID_VISIBLE")
                .short("g")
                .long("grid")
                .takes_value(true)
                .help("Show or hide the grid"),
        )
        .arg(
            Arg::with_name("TIMEOUT")
                .long("timeout")
                .takes_value(true)
                .help("Sets the time budget for the rendering process"),
        )
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let output = matches.value_of("OUTPUT").unwrap();

    let width = matches
        .value_of("WIDTH")
        .unwrap_or_default()
        .parse::<u32>()
        .unwrap_or(256);
    let height = matches
        .value_of("HEIGHT")
        .unwrap_or_default()
        .parse::<u32>()
        .unwrap_or(256);

    let settings = Settings {
        verbose: matches.is_present("VERBOSE"),
        lazy: matches.is_present("LAZY"),
        recalculate_normals: matches.is_present("RECALC_NORMALS"),
        size_hint: matches.is_present("SIZE_HINT") && height >= 256,
        turntable: matches.is_present("TURNTABLE"),
        grid: matches
            .value_of("GRID_VISIBLE")
            .unwrap_or_default()
            .parse::<bool>()
            .unwrap_or(true),
        cam_elevation: matches
            .value_of("CAM_ELEVATION")
            .unwrap_or_default()
            .parse::<f32>()
            .unwrap_or(25.0),
        cam_azimuth: matches
            .value_of("CAM_AZIMUTH")
            .unwrap_or_default()
            .parse::<f32>()
            .unwrap_or(45.0),
        timeout: matches
            .value_of("TIMEOUT")
            .unwrap_or_default()
            .parse::<u64>()
            .ok()
            .map_or(None, |v| Some(Duration::from_millis(v))),
    };

    if settings.verbose {
        println!("Size                  '{}x{}'", width, height);
        println!("Input                 '{}'", input);
        println!("Output                '{}'", output);
        println!("Recalculate normals   '{}'", settings.recalculate_normals);
        println!("Low memory usage mode '{}'", settings.lazy);
        println!("Draw dimensions       '{}'", settings.size_hint);
        println!("Grid visible          '{}'", settings.grid);
        println!("Cam elevation         {}°", settings.cam_elevation);
        println!("Cam azimuth           {}°", settings.cam_azimuth);
        println!("Timeout               {:?}", settings.timeout);
    }

    let start_time = Instant::now();
    let mut parser = Parser::from_file(&input, settings.recalculate_normals)?;

    if settings.lazy {
        let parsed_mesh = LazyMesh::new(&mut parser);
        create(width, height, &parsed_mesh, &output, &settings)?;
    } else {
        let parsed_mesh = parser.read_all()?;
        create(width, height, &parsed_mesh, &output, &settings)?;
    }

    if settings.verbose {
        println!(
            "Saved as '{}' (took {}s)",
            output,
            (Instant::now() - start_time).as_secs_f32()
        );
    }

    Ok(())
}

fn create(
    width: u32,
    height: u32,
    mesh: impl IntoIterator<Item = Triangle> + Copy,
    path: &str,
    settings: &Settings,
) -> Result<()> {
    if settings.turntable {
        create_turntable_animation(width, height, mesh, path, settings)
    } else {
        create_still(width, height, mesh, path, settings)
    }
}

fn create_still(
    width: u32,
    height: u32,
    mesh: impl IntoIterator<Item = Triangle> + Copy,
    path: &str,
    settings: &Settings,
) -> Result<()> {
    let elevation_angle = settings.cam_elevation * std::f32::consts::PI / 180.0;
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

fn create_turntable_animation(
    width: u32,
    height: u32,
    mesh: impl IntoIterator<Item = Triangle> + Copy,
    path: &str,
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
