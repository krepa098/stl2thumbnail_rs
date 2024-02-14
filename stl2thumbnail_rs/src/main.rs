use stl2thumbnail::*;

use anyhow::{bail, Result};
use stl::mesh::LazyMesh;
use stl::parser::Parser;

use clap::{builder::PathBufValueParser, Arg, ArgAction, ArgMatches, Command};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

fn main() -> Result<()> {
    let stl_command = Command::new("stl")
        .about("Renders an image of an stl file")
        .arg(
            Arg::new("INPUT")
                .index(1)
                .help("Input filename")
                .required(true)
                .value_parser(PathBufValueParser::new()),
        )
        .arg(
            Arg::new("OUTPUT")
                .index(2)
                .help("Output filename")
                .required(true)
                .value_parser(PathBufValueParser::new()),
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
                .value_parser(clap::value_parser!(u32))
                .help("Width of the generated image"),
        )
        .arg(
            Arg::new("HEIGHT")
                .short('h')
                .long("height")
                .action(ArgAction::Set)
                .default_value("256")
                .value_parser(clap::value_parser!(u32))
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
                .value_parser(clap::value_parser!(f32))
                .help("The camera's elevation"),
        )
        .arg(
            Arg::new("CAM_AZIMUTH")
                .long("cam-azimuth")
                .action(ArgAction::Set)
                .default_value("45.0")
                .value_parser(clap::value_parser!(f32))
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
                .index(1)
                .help("Input filename")
                .required(true)
                .value_parser(PathBufValueParser::new()),
        )
        .arg(
            Arg::new("OUTPUT")
                .index(2)
                .help("Output filename")
                .required(true)
                .value_parser(PathBufValueParser::new()),
        )
        .arg(
            Arg::new("WIDTH")
                .short('w')
                .long("width")
                .action(ArgAction::Set)
                .default_value("256")
                .value_parser(clap::value_parser!(u32))
                .help("Width of the generated image"),
        )
        .arg(
            Arg::new("HEIGHT")
                .short('h')
                .long("height")
                .action(ArgAction::Set)
                .default_value("256")
                .value_parser(clap::value_parser!(u32))
                .help("Height of the generated image"),
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
                .index(1)
                .help("Input filename")
                .required(true)
                .value_parser(PathBufValueParser::new()),
        )
        .arg(
            Arg::new("OUTPUT")
                .index(2)
                .help("Output filename")
                .required(true)
                .value_parser(PathBufValueParser::new()),
        )
        .arg(
            Arg::new("WIDTH")
                .short('w')
                .long("width")
                .action(ArgAction::Set)
                .default_value("256")
                .value_parser(clap::value_parser!(u32))
                .help("Width of the generated image"),
        )
        .arg(
            Arg::new("HEIGHT")
                .short('h')
                .long("height")
                .action(ArgAction::Set)
                .default_value("256")
                .value_parser(clap::value_parser!(u32))
                .help("Height of the generated image"),
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
    let input = matches.get_one::<PathBuf>("INPUT").unwrap();
    let output = matches.get_one::<PathBuf>("OUTPUT").unwrap();

    let file_extension = input.extension().map(|ex| ex.to_ascii_lowercase());

    let width = matches.get_one::<u32>("WIDTH").unwrap();
    let height = matches.get_one::<u32>("HEIGHT").unwrap();

    let settings = Settings {
        verbose: *matches.get_one::<bool>("VERBOSE").unwrap(),
        lazy: *matches.get_one::<bool>("LAZY").unwrap(),
        recalculate_normals: *matches.get_one::<bool>("RECALC_NORMALS").unwrap(),
        size_hint: *matches.get_one::<bool>("SIZE_HINT").unwrap() && *height >= 256,
        turntable: *matches.get_one::<bool>("TURNTABLE").unwrap(),
        grid: *matches.get_one::<bool>("GRID_VISIBLE").unwrap(),
        cam_elevation: *matches.get_one::<f32>("CAM_ELEVATION").unwrap(),
        cam_azimuth: *matches.get_one::<f32>("CAM_AZIMUTH").unwrap(),
        timeout: matches.get_one::<u64>("TIMEOUT").map(|v| Duration::from_millis(*v)),
    };

    if settings.verbose {
        println!("Size                  '{}x{}'", width, height);
        println!("Input                 '{}'", input.to_string_lossy());
        println!("Output                '{}'", output.to_string_lossy());
        println!("Recalculate normals   '{}'", settings.recalculate_normals);
        println!("Low memory usage mode '{}'", settings.lazy);
        println!("Draw dimensions       '{}'", settings.size_hint);
        println!("Grid visible          '{}'", settings.grid);
        println!("Cam elevation         {}°", settings.cam_elevation);
        println!("Cam azimuth           {}°", settings.cam_azimuth);
        println!("Timeout               {:?}", settings.timeout);
    }

    if file_extension == Some("stl".into()) {
        let start_time = Instant::now();
        let mut parser = Parser::from_file(input, settings.recalculate_normals)?;

        if settings.lazy {
            let parsed_mesh = LazyMesh::new(&mut parser);
            stl::render_stl(*width, *height, &parsed_mesh, output, &settings)?;
        } else {
            let parsed_mesh = parser.read_all()?;
            stl::render_stl(*width, *height, &parsed_mesh, output, &settings)?;
        }

        if settings.verbose {
            println!(
                "Saved as '{}' (took {}s)",
                output.to_string_lossy(),
                Instant::now().duration_since(start_time).as_secs_f32()
            );
        }
    } else {
        bail!("not a stl file");
    }

    Ok(())
}

fn command_gcode(matches: &ArgMatches) -> Result<()> {
    let input = matches.get_one::<PathBuf>("INPUT").unwrap();
    let output = matches.get_one::<PathBuf>("OUTPUT").unwrap();
    let width = matches.get_one::<u32>("WIDTH").unwrap();
    let height = matches.get_one::<u32>("HEIGHT").unwrap();

    let mut previews = gcode::extract_previews_from_file(input)?;

    let file_extension = input.extension().map(|ex| ex.to_ascii_lowercase());

    if file_extension == Some("gcode".into()) || file_extension == Some("bgcode".into()) {
        if let Some(preview) = previews.last_mut() {
            preview.resize_keep_aspect_ratio(*width, *height).save(output)?;
        }
    } else {
        bail!("not a gcode file");
    }

    Ok(())
}

fn command_3mf(matches: &ArgMatches) -> Result<()> {
    let input = matches.get_one::<PathBuf>("INPUT").unwrap();
    let output = matches.get_one::<PathBuf>("OUTPUT").unwrap();
    let width = *matches.get_one::<u32>("WIDTH").unwrap();
    let height = *matches.get_one::<u32>("HEIGHT").unwrap();

    let file_extension = input.extension().map(|ex| ex.to_ascii_lowercase());

    if file_extension == Some("3mf".into()) {
        let mut preview = threemf::extract_preview_from_file(input)?;
        preview.resize_keep_aspect_ratio(width, height).save(output)?;
    } else {
        bail!("not a 3mf file");
    }

    Ok(())
}
