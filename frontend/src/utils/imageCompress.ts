/**
 * Client-side image compression for LLM uploads.
 *
 * Resizes images to a max dimension and compresses to JPEG/WebP
 * to reduce payload size before sending to the coach endpoint.
 */

export interface CompressOptions {
  /** Max width or height in pixels (default: 1024) */
  maxDimension?: number;
  /** Output quality 0–1 (default: 0.8) */
  quality?: number;
  /** Output MIME type (default: 'image/webp', falls back to 'image/jpeg') */
  mimeType?: string;
}

const DEFAULT_OPTIONS: Required<CompressOptions> = {
  maxDimension: 1024,
  quality: 0.8,
  mimeType: 'image/webp',
};

/**
 * Compress an image file and return a base64 data URL.
 *
 * Uses OffscreenCanvas when available for performance,
 * falls back to a regular canvas element.
 */
export async function compressImage(
  file: File,
  opts?: CompressOptions
): Promise<string> {
  const { maxDimension, quality, mimeType } = { ...DEFAULT_OPTIONS, ...opts };

  // Load the image
  const bitmap = await createImageBitmap(file);
  const { width, height } = bitmap;

  // Calculate scaled dimensions preserving aspect ratio
  let targetWidth = width;
  let targetHeight = height;
  if (width > maxDimension || height > maxDimension) {
    const ratio = Math.min(maxDimension / width, maxDimension / height);
    targetWidth = Math.round(width * ratio);
    targetHeight = Math.round(height * ratio);
  }

  // Use OffscreenCanvas if available (worker-safe, faster)
  if (typeof OffscreenCanvas !== 'undefined') {
    const canvas = new OffscreenCanvas(targetWidth, targetHeight);
    const ctx = canvas.getContext('2d')!;
    ctx.drawImage(bitmap, 0, 0, targetWidth, targetHeight);
    bitmap.close();

    // Try preferred mime type, fall back to jpeg
    let blob = await canvas.convertToBlob({ type: mimeType, quality });
    if (blob.type !== mimeType && mimeType !== 'image/jpeg') {
      blob = await canvas.convertToBlob({ type: 'image/jpeg', quality });
    }
    return blobToDataUrl(blob);
  }

  // Fallback: regular canvas
  const canvas = document.createElement('canvas');
  canvas.width = targetWidth;
  canvas.height = targetHeight;
  const ctx = canvas.getContext('2d')!;
  ctx.drawImage(bitmap, 0, 0, targetWidth, targetHeight);
  bitmap.close();

  // Try preferred format
  let dataUrl = canvas.toDataURL(mimeType, quality);
  // If browser doesn't support webp, it returns png — detect and retry as jpeg
  if (mimeType === 'image/webp' && dataUrl.startsWith('data:image/png')) {
    dataUrl = canvas.toDataURL('image/jpeg', quality);
  }
  return dataUrl;
}

function blobToDataUrl(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = reject;
    reader.readAsDataURL(blob);
  });
}
