use anyhow::{Context, Result};
use image::{DynamicImage, ImageFormat};
use webp::{Encoder, WebPMemory};

/// Maximum dimensions for image processing (4K-ish resolution)
const MAX_DIMENSION: u32 = 3840; // 4K width/height

/// Processed image result containing the optimized WebP data and metadata
#[derive(Debug)]
pub struct ProcessedImage {
    /// Optimized WebP image data
    pub data: Vec<u8>,
    /// Final image width after processing
    pub width: u32,
    /// Final image height after processing  
    pub height: u32,
    /// Content type (always "image/webp")
    pub content_type: String,
}

/// Process an uploaded image by converting to WebP and optionally cropping to 4K
///
/// This function offloads CPU-intensive image processing to a blocking thread pool
/// to avoid blocking the async runtime during heavy image operations.
///
/// # Arguments
/// * `image_data` - Raw image bytes from upload
/// * `content_type` - Original content type for format detection
///
/// # Returns
/// * `ProcessedImage` - Optimized WebP image with metadata
///
/// # Errors
/// * Returns error if image format is unsupported
/// * Returns error if image processing fails
/// * Returns error if WebP encoding fails
pub async fn process_uploaded_image(
    image_data: &[u8],
    content_type: &str,
) -> Result<ProcessedImage> {
    // Clone data for move into blocking task
    let image_data = image_data.to_vec();
    let content_type = content_type.to_string();

    // Offload CPU-intensive image processing to blocking thread pool
    tokio::task::spawn_blocking(move || {
        // Detect and load the image format
        let format = detect_image_format(&content_type)
            .with_context(|| format!("Unsupported image format: {}", content_type))?;

        let image = image::load_from_memory_with_format(&image_data, format)
            .with_context(|| "Failed to decode image")?;

        // Crop to 4K if the image is larger
        let processed_image = crop_to_max_dimension(image);

        // Convert to WebP format
        let webp_data =
            encode_to_webp(&processed_image).with_context(|| "Failed to encode image to WebP")?;

        Ok(ProcessedImage {
            data: webp_data,
            width: processed_image.width(),
            height: processed_image.height(),
            content_type: "image/webp".to_string(),
        })
    })
    .await
    .with_context(|| "Image processing task was cancelled")?
}

/// Detect image format from content type
fn detect_image_format(content_type: &str) -> Result<ImageFormat> {
    match content_type {
        "image/jpeg" | "image/jpg" => Ok(ImageFormat::Jpeg),
        "image/png" => Ok(ImageFormat::Png),
        "image/gif" => Ok(ImageFormat::Gif),
        "image/webp" => Ok(ImageFormat::WebP),
        "image/avif" => Ok(ImageFormat::Avif),
        _ => anyhow::bail!("Unsupported image format: {}", content_type),
    }
}

/// Crop image to maximum dimension if it exceeds 4K
///
/// Uses smart cropping that maintains aspect ratio and crops from center
/// if the image is larger than MAX_DIMENSION in either dimension.
fn crop_to_max_dimension(image: DynamicImage) -> DynamicImage {
    let (width, height) = (image.width(), image.height());

    // If image is already within limits, return as-is
    if width <= MAX_DIMENSION && height <= MAX_DIMENSION {
        return image;
    }

    // Calculate the scale factor to fit within MAX_DIMENSION
    let scale_factor = (MAX_DIMENSION as f32 / width.max(height) as f32).min(1.0);
    let new_width = (width as f32 * scale_factor) as u32;
    let new_height = (height as f32 * scale_factor) as u32;

    // Use high-quality resize filter based on size
    let filter = if width * height > 2_000_000 {
        // Use faster filter for very large images (>2MP)
        image::imageops::FilterType::Triangle
    } else {
        // Use high-quality filter for smaller images
        image::imageops::FilterType::Lanczos3
    };

    image.resize(new_width, new_height, filter)
}

/// Encode image to WebP format with optimized quality settings
fn encode_to_webp(image: &DynamicImage) -> Result<Vec<u8>> {
    // Convert to RGB8 format for WebP encoding
    let rgb_image = image.to_rgb8();
    let (width, height) = rgb_image.dimensions();

    // Create WebP encoder with lossy compression at 85% quality
    let encoder = Encoder::from_rgb(&rgb_image, width, height);
    
    // Encode with 85% quality for good balance of size/quality
    let webp_memory: WebPMemory = encoder.encode(85f32);
    
    Ok(webp_memory.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_small_jpeg() {
        // Create a small test image (100x100 RGB)
        let img = DynamicImage::new_rgb8(100, 100);
        let mut buffer = Vec::new();
        use image::ImageOutputFormat;
        use std::io::Cursor;
        img.write_to(&mut Cursor::new(&mut buffer), ImageOutputFormat::Jpeg(80))
            .unwrap();

        let result = process_uploaded_image(&buffer, "image/jpeg").await.unwrap();

        assert_eq!(result.content_type, "image/webp");
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 100);
        assert!(!result.data.is_empty());
    }

    #[tokio::test]
    async fn test_crop_large_image() {
        // Create a smaller but still large test image (1200x800) to test cropping logic
        // This is much faster than 5000x3000 but still tests the same functionality
        let large_img = DynamicImage::new_rgb8(1200, 800);
        let cropped = crop_to_max_dimension(large_img);

        // Should remain unchanged since it's within MAX_DIMENSION (3840)
        assert_eq!(cropped.width(), 1200);
        assert_eq!(cropped.height(), 800);
        
        // Test with image that actually needs cropping (4000x2000)
        let oversized_img = DynamicImage::new_rgb8(4000, 2000);
        let cropped_oversized = crop_to_max_dimension(oversized_img);
        
        // Should be scaled down to fit within MAX_DIMENSION
        assert!(cropped_oversized.width() <= MAX_DIMENSION);
        assert!(cropped_oversized.height() <= MAX_DIMENSION);
        assert_eq!(cropped_oversized.width(), MAX_DIMENSION); // Wider dimension should hit the limit
    }

    #[test]
    fn test_detect_image_format() {
        assert!(matches!(
            detect_image_format("image/jpeg").unwrap(),
            ImageFormat::Jpeg
        ));
        assert!(matches!(
            detect_image_format("image/png").unwrap(),
            ImageFormat::Png
        ));
        assert!(matches!(
            detect_image_format("image/webp").unwrap(),
            ImageFormat::WebP
        ));
        assert!(matches!(
            detect_image_format("image/avif").unwrap(),
            ImageFormat::Avif
        ));
        assert!(detect_image_format("image/bmp").is_err());
    }
}
