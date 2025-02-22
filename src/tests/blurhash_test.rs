#![warn(clippy::all, clippy::pedantic)]

#[cfg(test)]
mod tests {
    use crate::blurhash::*;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn test_sign_pow() {
        assert_eq!(sign_pow(2.0, 2.0), 4.0);
        assert_eq!(sign_pow(-2.0, 2.0), 4.0);
        assert_eq!(sign_pow(-2.0, 3.0), -8.0);
        assert_eq!(sign_pow(0.0, 2.0), 0.0);
    }

    #[test]
    fn test_linear_to_srgb() {
        assert_eq!(linear_to_srgb(0.0), 0);
        assert_eq!(linear_to_srgb(1.0), 255);
        assert_eq!(linear_to_srgb(0.5), 188); // Approximate value
    }

    #[test]
    fn test_srgb_to_linear() {
        assert!(f64::abs(srgb_to_linear(0) - 0.0) < 0.001);
        assert!(f64::abs(srgb_to_linear(255) - 1.0) < 0.001);
        assert!(f64::abs(srgb_to_linear(188) - 0.5) < 0.01); // Approximate value
    }

    #[test]
    fn test_encode_base83_string() {
        assert_eq!(encode_base83_string(0, 1), "0");
        assert_eq!(encode_base83_string(82, 1), "~");
        assert_eq!(encode_base83_string(83, 2), "10");
    }

    #[test]
    fn test_encode_dc() {
        let dc = [1.0, 0.0, 0.5];
        let result = encode_dc(dc);
        assert!(result > 0);
    }

    #[test]
    fn test_encode_ac() {
        let ac = [0.5, -0.5, 0.0];
        let max_value = 1.0;
        let result = encode_ac(ac, max_value);
        assert!(result < 19 * 19 * 19);
    }

    #[test]
    fn test_encode_invalid_components() {
        let pixels = vec![0u8; 16]; // 2x2 RGBA image
        let result = encode(pixels.clone(), 0, 1, 2, 2);
        assert!(matches!(
            result,
            Err(EncodingError::ComponentsNumberInvalid)
        ));

        let result = encode(pixels, 10, 1, 2, 2);
        assert!(matches!(
            result,
            Err(EncodingError::ComponentsNumberInvalid)
        ));
    }

    #[test]
    fn test_encode_invalid_pixel_count() {
        let pixels = vec![0u8; 15]; // Invalid size for RGBA
        let result = encode(pixels, 4, 3, 2, 2);
        assert!(matches!(result, Err(EncodingError::BytesPerPixelMismatch)));
    }

    #[test]
    fn test_encode_valid_image() {
        // Create a 2x2 test image with known RGBA values
        let pixels: Vec<u8> = vec![
            255, 0, 0, 255, // Red pixel
            0, 255, 0, 255, // Green pixel
            0, 0, 255, 255, // Blue pixel
            255, 255, 255, 255, // White pixel
        ];

        let result = encode(pixels, 4, 3, 2, 2);
        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(!hash.is_empty());
        assert!(hash.chars().all(|c| ENCODE_CHARACTERS.contains(&c)));
    }

    // Helper function to create test images
    fn create_test_image(width: u32, height: u32, color: Rgba<u8>) -> Vec<u8> {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(width, height, color);
        img.into_raw()
    }

    #[test]
    fn test_encode_solid_color() {
        let red = Rgba([255, 0, 0, 255]);
        let pixels = create_test_image(4, 4, red);
        let result = encode(pixels, 4, 3, 4, 4);
        assert!(result.is_ok());
    }
}
