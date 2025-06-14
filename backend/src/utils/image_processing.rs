use anyhow::{Context, Result};
use image::codecs::avif::AvifEncoder;
use image::{ColorType, DynamicImage, ImageEncoder, ImageFormat};

/// Maximum dimensions for image processing (4K-ish resolution)
const MAX_DIMENSION: u32 = 3840; // 4K width/height

/// Processed image result containing the optimized AVIF data and metadata
#[derive(Debug)]
pub struct ProcessedImage {
    /// Optimized AVIF image data
    pub data: Vec<u8>,
    /// Final image width after processing
    pub width: u32,
    /// Final image height after processing  
    pub height: u32,
    /// Content type (always "image/avif")
    pub content_type: String,
}

/// Process an uploaded image by converting to AVIF and optionally cropping to 4K
///
/// # Arguments
/// * `image_data` - Raw image bytes from upload
/// * `content_type` - Original content type for format detection
///
/// # Returns
/// * `ProcessedImage` - Optimized AVIF image with metadata
///
/// # Errors
/// * Returns error if image format is unsupported
/// * Returns error if image processing fails
/// * Returns error if AVIF encoding fails
pub async fn process_uploaded_image(
    image_data: &[u8],
    content_type: &str,
) -> Result<ProcessedImage> {
    // Detect and load the image format
    let format = detect_image_format(content_type)
        .with_context(|| format!("Unsupported image format: {}", content_type))?;

    let image = image::load_from_memory_with_format(image_data, format)
        .with_context(|| "Failed to decode image")?;

    // Crop to 4K if the image is larger
    let processed_image = crop_to_max_dimension(image);

    // Convert to AVIF format
    let avif_data =
        encode_to_avif(&processed_image).with_context(|| "Failed to encode image to AVIF")?;

    Ok(ProcessedImage {
        data: avif_data,
        width: processed_image.width(),
        height: processed_image.height(),
        content_type: "image/avif".to_string(),
    })
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

/// Encode image to AVIF format with optimized quality and speed settings
fn encode_to_avif(image: &DynamicImage) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();

    // Create AVIF encoder with optimized settings
    // Speed 4 (good balance of speed/quality), Quality 85 (high quality)
    let encoder = AvifEncoder::new_with_speed_quality(&mut buffer, 4, 85)
        .with_num_threads(Some(std::thread::available_parallelism()?.get()));

    // Convert image to RGBA8 format for encoding
    let rgba_image = image.to_rgba8();
    let (width, height) = rgba_image.dimensions();

    // Encode the image data
    encoder
        .write_image(rgba_image.as_raw(), width, height, ColorType::Rgba8)
        .with_context(|| "Failed to encode image as AVIF")?;

    Ok(buffer)
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

        assert_eq!(result.content_type, "image/avif");
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 100);
        assert!(!result.data.is_empty());
    }

    #[tokio::test]
    async fn test_crop_large_image() {
        // Create a large test image (5000x3000)
        let large_img = DynamicImage::new_rgb8(5000, 3000);
        let cropped = crop_to_max_dimension(large_img);

        // Should be scaled down to fit within MAX_DIMENSION
        assert!(cropped.width() <= MAX_DIMENSION);
        assert!(cropped.height() <= MAX_DIMENSION);
        assert_eq!(cropped.width(), MAX_DIMENSION); // Wider dimension should hit the limit
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
        assert!(detect_image_format("image/bmp").is_err());
    }
}
