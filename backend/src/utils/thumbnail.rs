use image::{DynamicImage, GenericImageView, ImageFormat, ImageOutputFormat};
use std::io::Cursor;

use crate::models::photo::ThumbnailInfo;
use crate::utils::errors::{AppError, Result};

const THUMBNAIL_MAX_WIDTH: u32 = 300;
const THUMBNAIL_MAX_HEIGHT: u32 = 300;
const THUMBNAIL_QUALITY: u8 = 80;

/// Generate a thumbnail from image data
pub fn generate_thumbnail(
    image_data: &[u8],
    content_type: &str,
) -> Result<ThumbnailInfo> {
    // Determine the image format
    let format = match content_type {
        "image/jpeg" => ImageFormat::Jpeg,
        "image/png" => ImageFormat::Png,
        "image/gif" => ImageFormat::Gif,
        "image/webp" => ImageFormat::WebP,
        _ => {
            return Err(AppError::Validation(
                validator::ValidationErrors::new()
            ));
        }
    };

    // Load the image
    let img = image::load_from_memory_with_format(image_data, format)
        .map_err(|e| {
            tracing::error!("Failed to load image: {}", e);
            AppError::Internal {
                message: "Failed to process image".to_string(),
            }
        })?;

    // Generate thumbnail
    let thumbnail = resize_image_to_thumbnail(&img);
    let (width, height) = (thumbnail.width(), thumbnail.height());

    // Encode thumbnail as JPEG for consistency and smaller file size
    let mut thumbnail_data = Vec::new();
    let mut cursor = Cursor::new(&mut thumbnail_data);
    
    thumbnail
        .write_to(&mut cursor, ImageOutputFormat::Jpeg(THUMBNAIL_QUALITY))
        .map_err(|e| {
            tracing::error!("Failed to encode thumbnail: {}", e);
            AppError::Internal {
                message: "Failed to generate thumbnail".to_string(),
            }
        })?;

    Ok(ThumbnailInfo {
        width: width as i32,
        height: height as i32,
        data: thumbnail_data,
    })
}

/// Resize image to thumbnail size while maintaining aspect ratio
fn resize_image_to_thumbnail(img: &DynamicImage) -> DynamicImage {
    let (original_width, original_height) = img.dimensions();
    
    // Calculate new dimensions while maintaining aspect ratio
    let (new_width, new_height) = calculate_thumbnail_dimensions(
        original_width,
        original_height,
        THUMBNAIL_MAX_WIDTH,
        THUMBNAIL_MAX_HEIGHT,
    );

    // For large images (>2MP), use faster Triangle filter for better performance
    // For smaller images, use higher quality Lanczos3
    let total_pixels = original_width * original_height;
    let filter = if total_pixels > 2_000_000 {
        // For large images, use Triangle which is faster
        image::imageops::FilterType::Triangle
    } else {
        // For smaller images, use high-quality Lanczos3
        image::imageops::FilterType::Lanczos3
    };

    tracing::debug!(
        "Resizing {}x{} image to {}x{} using {:?} filter", 
        original_width, original_height, new_width, new_height, filter
    );

    img.resize(new_width, new_height, filter)
}

/// Calculate thumbnail dimensions while maintaining aspect ratio
fn calculate_thumbnail_dimensions(
    original_width: u32,
    original_height: u32,
    max_width: u32,
    max_height: u32,
) -> (u32, u32) {
    // If image is already smaller than max dimensions, keep original size
    if original_width <= max_width && original_height <= max_height {
        return (original_width, original_height);
    }

    // Calculate scaling factors for width and height
    let width_scale = max_width as f64 / original_width as f64;
    let height_scale = max_height as f64 / original_height as f64;

    // Use the smaller scale factor to ensure both dimensions fit
    let scale = width_scale.min(height_scale);

    let new_width = (original_width as f64 * scale) as u32;
    let new_height = (original_height as f64 * scale) as u32;

    (new_width, new_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_thumbnail_dimensions_landscape() {
        let (width, height) = calculate_thumbnail_dimensions(800, 600, 300, 300);
        assert_eq!(width, 300);
        assert_eq!(height, 225);
    }

    #[test]
    fn test_calculate_thumbnail_dimensions_portrait() {
        let (width, height) = calculate_thumbnail_dimensions(600, 800, 300, 300);
        assert_eq!(width, 225);
        assert_eq!(height, 300);
    }

    #[test]
    fn test_calculate_thumbnail_dimensions_square() {
        let (width, height) = calculate_thumbnail_dimensions(600, 600, 300, 300);
        assert_eq!(width, 300);
        assert_eq!(height, 300);
    }

    #[test]
    fn test_calculate_thumbnail_dimensions_already_small() {
        let (width, height) = calculate_thumbnail_dimensions(200, 150, 300, 300);
        assert_eq!(width, 200);
        assert_eq!(height, 150);
    }

    #[test]
    fn test_calculate_thumbnail_dimensions_very_wide() {
        let (width, height) = calculate_thumbnail_dimensions(1200, 200, 300, 300);
        assert_eq!(width, 300);
        assert_eq!(height, 50);
    }

    #[test]
    fn test_calculate_thumbnail_dimensions_very_tall() {
        let (width, height) = calculate_thumbnail_dimensions(200, 1200, 300, 300);
        assert_eq!(width, 50);
        assert_eq!(height, 300);
    }
}