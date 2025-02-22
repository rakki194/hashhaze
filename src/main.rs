use anyhow::Result;
use clap::Parser;
use futures::future::join_all;
use imx::{get_image_dimensions, is_jxl_file, process_jxl_file};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use xio::{walk_directory, write_to_file};

mod blurhash;
#[cfg(test)]
mod tests;

// Helper function to check if a file is an image based on extension
pub(crate) fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif" | "webp")
    } else {
        false
    }
}

// Helper function to process files with a specific extension
async fn process_files_with_extension(dir: &Path, extension: &str, image_paths: Arc<Mutex<Vec<PathBuf>>>) -> Result<()> {
    walk_directory(dir, extension, move |path| {
        let image_paths = image_paths.clone();
        let path = path.to_path_buf();
        async move {
            let mut paths = image_paths.lock().await;
            paths.push(path);
            Ok(())
        }
    })
    .await
}

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

pub(crate) async fn get_image_paths(inputs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let image_paths = Arc::new(Mutex::new(Vec::new()));

    for input in inputs {
        let input_path = if input.as_os_str().is_empty() || input == Path::new(".") {
            Path::new(".")
        } else {
            input.as_path()
        };

        if input_path.is_dir() {
            // Process each supported image extension
            for ext in ["jpg", "jpeg", "png", "gif", "webp"] {
                process_files_with_extension(input_path, ext, image_paths.clone()).await?;
            }
        } else if is_image_file(input_path) {
            let mut paths = image_paths.lock().await;
            paths.push(input_path.to_path_buf());
        }
    }

    let paths = Arc::try_unwrap(image_paths)
        .expect("Failed to unwrap Arc")
        .into_inner();
    Ok(paths)
}

async fn process_image(input: PathBuf, components_x: usize, components_y: usize) -> Result<()> {
    // Generate the output filename
    let mut output_filename = input.clone();
    output_filename.set_extension(format!("{}.bh",
        input.extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("")));

    // Check if the .bh file already exists
    if output_filename.exists() {
        println!("Skipping {}: BlurHash file already exists", input.display());
        return Ok(());
    }

    // Handle JXL files specially
    if is_jxl_file(input.as_path()) {
        let temp_png = input.with_extension("png");
        let noop = |_: &Path| async { Ok(()) };
        process_jxl_file(input.as_path(), Some(noop)).await?;
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
    let input = input.to_path_buf();
    let input_clone = input.clone();
    let img = tokio::task::spawn_blocking(move || image::open(&input)).await??;
    let (width, height) = get_image_dimensions(&input_clone)?;
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
