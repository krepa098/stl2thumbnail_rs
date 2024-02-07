use stl2thumbnail::*;

use anyhow::Result;
use encoder::*;
use mesh::LazyMesh;
use mesh::{Triangle, Vec3};
use parser::Parser;
use picture::Picture;
use rasterbackend::RasterBackend;

use clap::{Arg, ArgAction, ArgMatches, Command};
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
    let stl_command = Command::new("stl")
        .about("Renders an image of an stl file")
        .arg(
            Arg::new("INPUT")
                .short('i')
                .index(1)
                .long("input")
                .help("Input filename")
                .required(true),
        )
        .arg(
            Arg::new("OUTPUT")
                .short('o')
                .index(2)
                .long("output")
                .help("Output filename")
                .required(true),
        )
        .arg(
            Arg::new("TURNTABLE")
                .short('t')
                .long("turntable")
                .action(ArgAction::SetTrue)
                .help("Enables turntable mode"),
        )
        .arg(
            Arg::new("VERBOSE")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Be verbose"),
        )
        .arg(
            Arg::new("LAZY")
                .short('l')
                .long("lazy")
                .action(ArgAction::SetTrue)
                .help("Enables low memory usage mode"),
        )
        .arg(
            Arg::new("RECALC_NORMALS")
                .short('n')
                .long("normals")
                .action(ArgAction::SetTrue)
                .help("Always recalculate normals"),
        )
        .arg(
            Arg::new("WIDTH")
                .short('w')
                .long("width")
                .action(ArgAction::Set)
                .default_value("256")
                .help("Width of the generated image"),
        )
        .arg(
            Arg::new("HEIGHT")
                .short('h')
                .long("height")
                .action(ArgAction::Set)
                .default_value("256")
                .help("Height of the generated image"),
        )
        .arg(
            Arg::new("SIZE_HINT")
                .short('d')
                .long("dimensions")
                .action(ArgAction::SetTrue)
                .help("Draws the dimensions underneath the model (requires height of at least 256 pixels)"),
        )
        .arg(
            Arg::new("CAM_ELEVATION")
                .long("cam-elevation")
                .action(ArgAction::Set)
                .default_value("25.0")
                .help("The camera's elevation"),
        )
        .arg(
            Arg::new("CAM_AZIMUTH")
                .long("cam-azimuth")
                .action(ArgAction::Set)
                .default_value("45.0")
                .help("The camera's azimuth"),
        )
        .arg(
            Arg::new("GRID_VISIBLE")
                .short('g')
                .long("grid")
                .action(ArgAction::SetTrue)
                .help("Show or hide the grid"),
        )
        .arg(
            Arg::new("TIMEOUT")
                .long("timeout")
                .action(ArgAction::Set)
                .help("Sets the time budget for the rendering process"),
        )
        .arg(
            Arg::new("HELP")
                .long("help")
                .action(ArgAction::HelpLong)
                .help("Prints this"),
        );

    let gcode_command = Command::new("gcode")
        .about("Extracts a thumbnail embedded in a gcode file")
        .arg(
            Arg::new("INPUT")
                .short('i')
                .index(1)
                .long("input")
                .help("Input filename")
                .required(true),
        )
        .arg(
            Arg::new("OUTPUT")
                .short('o')
                .index(2)
                .long("output")
                .help("Output filename")
                .required(true),
        )
        .arg(
            Arg::new("HELP")
                .long("help")
                .action(ArgAction::HelpLong)
                .help("Prints this"),
        );

    let threemf_command = Command::new("3mf")
        .about("Extracts a thumbnail embedded in a gcode file")
        .arg(
            Arg::new("INPUT")
                .short('i')
                .index(1)
                .long("input")
                .help("Input filename")
                .required(true),
        )
        .arg(
            Arg::new("OUTPUT")
                .short('o')
                .index(2)
                .long("output")
                .help("Output filename")
                .required(true),
        )
        .arg(
            Arg::new("HELP")
                .long("help")
                .action(ArgAction::HelpLong)
                .help("Prints this"),
        );

    let matches = Command::new("stl2thumbnail")
        .version(clap::crate_version!())
        .arg_required_else_help(true)
        .disable_help_flag(true)
        .about("STL thumbnail generator")
        .subcommand(stl_command)
        .subcommand(gcode_command)
        .subcommand(threemf_command)
        .get_matches();

    if let Some((subcommand, matches)) = matches.subcommand() {
        match subcommand {
            "stl" => command_stl(matches)?,
            "gcode" => command_gcode(matches)?,
            "3mf" => command_3mf(matches)?,
            _ => unimplemented!(),
        }
    }

    Ok(())
}

fn command_stl(matches: &ArgMatches) -> Result<()> {
    let input = matches.get_one::<String>("INPUT").unwrap();
    let output = matches.get_one::<String>("OUTPUT").unwrap();

    let width = matches.get_one::<u32>("WIDTH").unwrap();
    let height = matches.get_one::<u32>("HEIGHT").unwrap();

    let settings = Settings {
        verbose: matches.contains_id("VERBOSE"),
        lazy: matches.contains_id("LAZY"),
        recalculate_normals: matches.contains_id("RECALC_NORMALS"),
        size_hint: matches.contains_id("SIZE_HINT") && *height >= 256,
        turntable: matches.contains_id("TURNTABLE"),
        grid: matches.contains_id("GRID_VISIBLE"),
        cam_elevation: *matches.get_one::<f32>("CAM_ELEVATION").unwrap(),
        cam_azimuth: *matches.get_one::<f32>("CAM_AZIMUTH").unwrap(),
        timeout: matches
            .get_one::<u64>("TIMEOUT")
            .map_or(None, |v| Some(Duration::from_millis(*v))),
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
        create(*width, *height, &parsed_mesh, &output, &settings)?;
    } else {
        let parsed_mesh = parser.read_all()?;
        create(*width, *height, &parsed_mesh, &output, &settings)?;
    }

    if settings.verbose {
        println!(
            "Saved as '{}' (took {}s)",
            output,
            Instant::now().duration_since(start_time).as_secs_f32()
        );
    }

    Ok(())
}

fn command_gcode(matches: &ArgMatches) -> Result<()> {
    let input = matches.get_one::<String>("INPUT").unwrap();
    let output = matches.get_one::<String>("OUTPUT").unwrap();

    let previews = gcode::extract_previews_from_file(&input)?;

    if let Some(preview) = previews.last() {
        preview.save(output)?;
    }

    Ok(())
}

fn command_3mf(matches: &ArgMatches) -> Result<()> {
    let input = matches.get_one::<String>("INPUT").unwrap();
    let output = matches.get_one::<String>("OUTPUT").unwrap();

    let preview = threemf::extract_preview_from_file(&input)?;
    preview.save(output)?;

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
