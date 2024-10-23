use clap::Parser;
use image::GenericImageView;
use std::path::{Path, PathBuf};
use std::fs;

mod blurhash;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input image files or directories
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    /// Number of X components for BlurHash
    #[arg(short = 'x', long, default_value_t = 4)]
    components_x: usize,

    /// Number of Y components for BlurHash
    #[arg(short = 'y', long, default_value_t = 3)]
    components_y: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let image_paths = get_image_paths(&args.inputs)?;

    for image_path in image_paths {
        process_image(&image_path, args.components_x, args.components_y)?;
    }

    Ok(())
}

fn get_image_paths(inputs: &[PathBuf]) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut image_paths = Vec::new();

    for input in inputs {
        if input.is_dir() {
            for entry in fs::read_dir(input)? {
                let entry = entry?;
                let path = entry.path();
                if is_image_file(&path) {
                    image_paths.push(path);
                }
            }
        } else if is_image_file(input) {
            image_paths.push(input.to_path_buf());
        }
    }

    Ok(image_paths)
}

fn is_image_file(path: &Path) -> bool {
    let extensions = ["jpg", "jpeg", "png", "gif", "bmp", "tiff"];
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn process_image(input: &Path, components_x: usize, components_y: usize) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(input)?;
    let (width, height) = img.dimensions();
    let rgba_image = img.to_rgba8();
    let pixels: Vec<u8> = rgba_image.into_raw();

    let blurhash = blurhash::encode(
        pixels,
        components_x,
        components_y,
        width as usize,
        height as usize,
    )?;

    // Generate the output filename
    let mut output_filename = input.to_path_buf();
    let new_extension = format!("{}.bh", output_filename.extension().unwrap_or_default().to_str().unwrap_or(""));
    output_filename.set_extension(new_extension);

    // Save the BlurHash to the file
    std::fs::write(&output_filename, &blurhash)?;

    println!("BlurHash saved to: {}", output_filename.display());

    Ok(())
}
