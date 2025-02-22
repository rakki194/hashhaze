#[cfg(test)]
mod tests {
    use crate::*;
    use tempfile::tempdir;
    use tokio::fs::File;
    use anyhow::Result;
    use image::{ImageBuffer, Rgb, DynamicImage};

    #[tokio::test]
    async fn test_get_image_paths_empty_input() -> Result<()> {
        let temp_dir = tempdir()?;
        // Create a valid PNG image
        let test_file = temp_dir.path().join("test.png");
        let img = ImageBuffer::from_pixel(2, 2, Rgb([255u8, 0, 0]));
        img.save(&test_file)?;

        // Verify that the image can be opened
        let test_path = test_file.clone();
        tokio::task::spawn_blocking(move || {
            image::open(&test_path).unwrap();
        }).await?;

        let inputs = vec![temp_dir.path().to_path_buf()];
        let paths = get_image_paths(&inputs).await?;

        assert!(!paths.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_image_paths_directory() -> Result<()> {
        let temp_dir = tempdir()?;

        // Create test image files
        let test_png = temp_dir.path().join("test.png");
        let test_jpg = temp_dir.path().join("test.jpg");
        let test_txt = temp_dir.path().join("test.txt");

        // Create valid RGB images
        let png_img = ImageBuffer::from_pixel(2, 2, Rgb([255u8, 0, 0]));
        let jpg_img = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(2, 2, Rgb([0u8, 255, 0])));
        
        png_img.save(&test_png)?;
        jpg_img.save(&test_jpg)?;
        File::create(&test_txt).await?;

        // Verify that the images can be opened
        let png_path = test_png.clone();
        let jpg_path = test_jpg.clone();
        tokio::task::spawn_blocking(move || {
            image::open(&png_path).unwrap();
            image::open(&jpg_path).unwrap();
        }).await?;

        let inputs = vec![temp_dir.path().to_path_buf()];
        let paths = get_image_paths(&inputs).await?;

        assert_eq!(paths.len(), 2); // Should only include PNG and JPG
        assert!(paths.iter().any(|p| p.extension().unwrap_or_default() == "png"));
        assert!(paths.iter().any(|p| p.extension().unwrap_or_default() == "jpg"));
        Ok(())
    }

    #[tokio::test]
    async fn test_check_and_add_image_path() -> Result<()> {
        let temp_dir = tempdir()?;
        let test_file = temp_dir.path().join("test.png");
        
        // Create a valid PNG image
        let img = ImageBuffer::from_pixel(2, 2, Rgb([255u8, 0, 0]));
        img.save(&test_file)?;

        let mut image_paths = Vec::new();
        check_and_add_image_path(&test_file, &mut image_paths).await?;

        assert_eq!(image_paths.len(), 1);
        assert_eq!(image_paths[0], test_file);

        // Test skipping existing .bh file
        let bh_file = temp_dir.path().join("test.png.bh");
        File::create(&bh_file).await?;

        let mut image_paths = Vec::new();
        check_and_add_image_path(&test_file, &mut image_paths).await?;

        assert_eq!(image_paths.len(), 0); // Should skip because .bh exists
        Ok(())
    }

    #[tokio::test]
    async fn test_process_image() -> Result<()> {
        let temp_dir = tempdir()?;
        let test_file = temp_dir.path().join("test.png");

        // Create a valid PNG image
        let img = ImageBuffer::from_pixel(2, 2, Rgb([255u8, 0, 0]));
        img.save(&test_file)?;

        process_image(test_file.clone(), 4, 3).await?;

        let bh_file = temp_dir.path().join("test.png.bh");
        assert!(bh_file.exists());

        // Test skipping existing file
        process_image(test_file, 4, 3).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_process_regular_image() -> Result<()> {
        let temp_dir = tempdir()?;
        let test_file = temp_dir.path().join("test.png");

        // Create a valid PNG image
        let img = ImageBuffer::from_pixel(2, 2, Rgb([255u8, 0, 0]));
        img.save(&test_file)?;

        let blurhash = process_regular_image(&test_file, 4, 3).await?;
        assert!(!blurhash.is_empty());
        Ok(())
    }
}
