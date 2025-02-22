use anyhow::Result;
use clap::Parser;
use futures::future::join_all;
use imx::{get_image_dimensions, is_image_file, process_jxl_file};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use xio::{read_file_content, walk_directory, write_to_file};

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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let image_paths = get_image_paths(&args.inputs).await?;

    // Create a semaphore to limit concurrent tasks
    let semaphore = Arc::new(Semaphore::new(num_cpus::get()));

    let tasks: Vec<_> = image_paths
        .into_iter()
        .map(|path| {
            let sem = semaphore.clone();
            let components_x = args.components_x;
            let components_y = args.components_y;
            tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                process_image(path, components_x, components_y).await
            })
        })
        .collect();

    // Wait for all tasks to complete
    let results = join_all(tasks).await;

    // Check for any errors
    for result in results {
        if let Err(e) = result? {
            eprintln!("Error processing image: {}", e);
        }
    }

    Ok(())
}

async fn get_image_paths(inputs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut image_paths = Vec::new();

    for input in inputs {
        if input.as_os_str().is_empty() || input == Path::new(".") {
            // If input is empty or ".", use the current directory
            walk_directory(".", "*", |path| {
                let path = path.to_path_buf();
                async move {
                    if is_image_file(&path.to_string_lossy()) {
                        check_and_add_image_path(&path, &mut image_paths).await?;
                    }
                    Ok(())
                }
            })
            .await?;
        } else if input.is_dir() {
            walk_directory(input, "*", |path| {
                let path = path.to_path_buf();
                async move {
                    if is_image_file(&path.to_string_lossy()) {
                        check_and_add_image_path(&path, &mut image_paths).await?;
                    }
                    Ok(())
                }
            })
            .await?;
        } else if is_image_file(&input.to_string_lossy()) {
            check_and_add_image_path(input, &mut image_paths).await?;
        }
    }

    Ok(image_paths)
}

async fn check_and_add_image_path(path: &Path, image_paths: &mut Vec<PathBuf>) -> Result<()> {
    // Generate the output filename
    let mut output_filename = path.to_path_buf();
    let new_extension = format!(
        "{}.bh",
        output_filename
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("")
    );
    output_filename.set_extension(new_extension);

    // Check if the .bh file already exists
    if !output_filename.exists() {
        image_paths.push(path.to_path_buf());
    } else {
        println!("Skipping {}: BlurHash file already exists", path.display());
    }
    Ok(())
}

async fn process_image(input: PathBuf, components_x: usize, components_y: usize) -> Result<()> {
    // Generate the output filename
    let mut output_filename = input.clone();
    let new_extension = format!(
        "{}.bh",
        output_filename
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("")
    );
    output_filename.set_extension(new_extension);

    // Check if the .bh file already exists
    if output_filename.exists() {
        println!("Skipping {}: BlurHash file already exists", input.display());
        return Ok(());
    }

    // Handle JXL files specially
    if is_jxl_file(&input.to_string_lossy()) {
        let temp_png = input.with_extension("png");
        process_jxl_file(&input, Some(|_| async move { Ok(()) })).await?;
        let blurhash = process_regular_image(&temp_png, components_x, components_y).await?;
        write_to_file(&output_filename, &blurhash).await?;
        tokio::fs::remove_file(&temp_png).await?;
    } else {
        let blurhash = process_regular_image(&input, components_x, components_y).await?;
        write_to_file(&output_filename, &blurhash).await?;
    }

    println!("BlurHash saved to: {}", output_filename.display());

    Ok(())
}

async fn process_regular_image(
    input: &Path,
    components_x: usize,
    components_y: usize,
) -> Result<String> {
    let img = tokio::task::spawn_blocking(move || image::open(input)).await??;
    let (width, height) = get_image_dimensions(input)?;
    let rgba_image = img.to_rgba8();
    let pixels: Vec<u8> = rgba_image.into_raw();

    let blurhash = blurhash::encode(
        pixels,
        components_x,
        components_y,
        width as usize,
        height as usize,
    )?;

    Ok(blurhash)
}
