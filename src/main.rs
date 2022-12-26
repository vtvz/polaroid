mod image;
mod things;

use anyhow::Result;
use clap::arg;
use clap::Parser;
use image::Polaroid;
use magick_rust::magick_wand_genesis;
use std::{fs, path::PathBuf, sync::Once};

extern crate dimensioned as dim;

// Used to make sure MagickWand is initialized exactly once. Note that we
// do not bother shutting down, we simply exit when we're done.
static START: Once = Once::new();

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(help = "input files", required = true)]
    files: Vec<PathBuf>,

    #[arg(last = true, help = "output dir", default_value = "./polaroid-output/")]
    output: PathBuf,

    #[arg(long, default_value_t = 300)]
    dpi: usize,

    #[arg(long, short = 'f', default_value = "tif")]
    output_format: String,
}

fn process_file(cli: &Cli, path: &PathBuf) -> Result<()> {
    let path_str = path.to_str().unwrap();
    println!("Process {} file", path_str);
    if !path.is_file() {
        println!("{} is not a file", path_str);

        return Ok(());
    }

    let mut output_file = cli.output.clone();
    output_file.push(path.file_stem().unwrap());

    output_file.set_extension(&cli.output_format);

    let mut polaroid = Polaroid::new_load_predict(path_str)?;

    polaroid.set_dpi(cli.dpi);
    polaroid.resize()?;
    polaroid.add_border()?;
    polaroid.add_frame()?;
    polaroid.add_output_filler()?;
    polaroid.write(output_file.to_str().unwrap())?;

    Ok(())
}

fn main() -> Result<()> {
    START.call_once(|| {
        magick_wand_genesis();
    });

    let cli: Cli = Cli::parse();

    if !cli.output.is_dir() {
        fs::create_dir(&cli.output)?;
    }

    for path in &cli.files {
        if let Err(err) = process_file(&cli, path) {
            println!("Error occured while processing file\n{}", err);
        }
    }

    Ok(())
}
