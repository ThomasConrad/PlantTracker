import { Component, createSignal, Show, onMount, onCleanup } from 'solid-js';
import { Button } from '@/components/ui/Button';

interface ThumbnailUploadProps {
  onFileSelect: (file: File) => void;
  selectedFile?: File | null;
  error?: string;
  loading?: boolean;
}

export const ThumbnailUpload: Component<ThumbnailUploadProps> = (props) => {
  const [preview, setPreview] = createSignal<string | null>(null);
  const [isMobile, setIsMobile] = createSignal(false);
  const [showCamera, setShowCamera] = createSignal(false);
  const [stream, setStream] = createSignal<MediaStream | null>(null);
  const [compressing, setCompressing] = createSignal(false);
  
  let fileInputRef: HTMLInputElement | undefined;
  let videoRef: HTMLVideoElement | undefined;
  let canvasRef: HTMLCanvasElement | undefined;

  // Detect if device is mobile
  onMount(() => {
    const checkMobile = () => {
      const isMobileDevice = /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent) ||
        (navigator.maxTouchPoints && navigator.maxTouchPoints > 2);
      setIsMobile(!!isMobileDevice);
    };
    
    checkMobile();
    window.addEventListener('resize', checkMobile);
    
    onCleanup(() => {
      window.removeEventListener('resize', checkMobile);
      stopCamera();
    });
  });

  // Create preview when file is selected
  const createPreview = (file: File) => {
    const reader = new FileReader();
    reader.onload = () => {
      setPreview(reader.result as string);
    };
    reader.readAsDataURL(file);
  };

  // Compress image if it's too large
  const compressImage = (file: File): Promise<File> => {
    return new Promise((resolve) => {
      // If file is smaller than 1MB, don't compress
      if (file.size < 1024 * 1024) {
        resolve(file);
        return;
      }

      setCompressing(true);

      const canvas = document.createElement('canvas');
      const ctx = canvas.getContext('2d');
      const img = new Image();
      
      img.onload = () => {
        // Calculate new dimensions (max 1920x1920 for large images)
        const maxDimension = 1920;
        let { width, height } = img;
        
        if (width > maxDimension || height > maxDimension) {
          if (width > height) {
            height = (height * maxDimension) / width;
            width = maxDimension;
          } else {
            width = (width * maxDimension) / height;
            height = maxDimension;
          }
        }
        
        canvas.width = width;
        canvas.height = height;
        
        // Draw and compress
        ctx?.drawImage(img, 0, 0, width, height);
        
        canvas.toBlob((blob) => {
          setCompressing(false);
          if (blob) {
            const compressedFile = new File([blob], file.name, {
              type: 'image/jpeg',
              lastModified: Date.now(),
            });
            // Compressed file successfully
            resolve(compressedFile);
          } else {
            resolve(file);
          }
        }, 'image/jpeg', 0.8); // 80% quality
      };
      
      img.src = URL.createObjectURL(file);
    });
  };

  // Handle file input change
  const handleFileChange = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];
    if (file && file.type.startsWith('image/')) {
      try {
        const compressedFile = await compressImage(file);
        createPreview(compressedFile);
        props.onFileSelect(compressedFile);
      } catch (error) {
        console.error('Failed to process image:', error);
        createPreview(file);
        props.onFileSelect(file);
      }
    }
  };

  // Start camera
  const startCamera = async () => {
    try {
      const mediaStream = await navigator.mediaDevices.getUserMedia({
        video: { 
          facingMode: 'environment', // Use back camera on mobile
          width: { ideal: 1280 },
          height: { ideal: 720 }
        }
      });
      
      setStream(mediaStream);
      setShowCamera(true);
      
      if (videoRef) {
        videoRef.srcObject = mediaStream;
        videoRef.play();
      }
    } catch (error) {
      console.error('Error accessing camera:', error);
      alert('Camera access denied or not available. Please use file upload instead.');
    }
  };

  // Stop camera
  const stopCamera = () => {
    const currentStream = stream();
    if (currentStream) {
      currentStream.getTracks().forEach(track => track.stop());
      setStream(null);
    }
    setShowCamera(false);
  };

  // Capture photo from camera
  const capturePhoto = () => {
    if (!videoRef || !canvasRef) return;

    const video = videoRef;
    const canvas = canvasRef;
    const context = canvas.getContext('2d');
    
    if (!context) return;

    // Set canvas size to video size
    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;
    
    // Draw video frame to canvas
    context.drawImage(video, 0, 0);
    
    // Convert canvas to blob
    canvas.toBlob(async (blob) => {
      if (blob) {
        const file = new File([blob], `plant-thumbnail-${Date.now()}.jpg`, { 
          type: 'image/jpeg' 
        });
        try {
          const compressedFile = await compressImage(file);
          createPreview(compressedFile);
          props.onFileSelect(compressedFile);
        } catch (error) {
          console.error('Failed to compress camera image:', error);
          createPreview(file);
          props.onFileSelect(file);
        }
        stopCamera();
      }
    }, 'image/jpeg', 0.8);
  };

  const removePhoto = () => {
    setPreview(null);
    if (fileInputRef) {
      fileInputRef.value = '';
    }
  };

  return (
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <label class="block text-sm font-medium text-gray-700">
          Plant Thumbnail
        </label>
        <Show when={preview()}>
          <button
            type="button"
            onClick={removePhoto}
            class="text-sm text-red-600 hover:text-red-700"
          >
            Remove
          </button>
        </Show>
      </div>

      {/* Camera Modal */}
      <Show when={showCamera()}>
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div class="bg-white rounded-lg shadow-xl max-w-md w-full">
            <div class="p-4">
              <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-semibold">Take Photo</h3>
                <button
                  onClick={stopCamera}
                  class="text-gray-400 hover:text-gray-600"
                >
                  <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
              
              <div class="relative">
                <video
                  ref={videoRef}
                  class="w-full h-64 object-cover rounded-lg bg-black"
                  autoplay
                  muted
                  playsinline
                />
                <canvas ref={canvasRef} class="hidden" />
              </div>
              
              <div class="flex justify-center mt-4">
                <Button
                  onClick={capturePhoto}
                  class="bg-blue-600 hover:bg-blue-700"
                >
                  <svg class="mr-2 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 13a3 3 0 11-6 0 3 3 0 016 0z" />
                  </svg>
                  Capture
                </Button>
              </div>
            </div>
          </div>
        </div>
      </Show>

      {/* Preview or Upload Area */}
      <Show
        when={preview()}
        fallback={
          <div class="border-2 border-dashed border-gray-300 rounded-lg p-6 text-center hover:border-gray-400 transition-colors">
            <svg class="mx-auto h-12 w-12 text-gray-400" stroke="currentColor" fill="none" viewBox="0 0 48 48">
              <path d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H12a4 4 0 01-4-4v-4m32-4l-3.172-3.172a4 4 0 00-5.656 0L28 28M8 32l9.172-9.172a4 4 0 015.656 0L28 28m0 0l4 4m4-24h8m-4-4v8m-12 4h.02" stroke-width={2} stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            <div class="mt-4">
              <p class="text-sm text-gray-600 mb-4">
                Add a thumbnail photo for your plant
              </p>
              
              <div class="flex flex-col sm:flex-row gap-2 justify-center">
                <Show when={isMobile()}>
                  <Button
                    type="button"
                    variant="outline"
                    onClick={startCamera}
                    class="flex items-center"
                  >
                    <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z" />
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 13a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    Take Photo
                  </Button>
                </Show>
                
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => fileInputRef?.click()}
                  class="flex items-center"
                >
                  <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                  </svg>
                  {isMobile() ? 'Choose File' : 'Upload Photo'}
                </Button>
              </div>
              
              <p class="text-xs text-gray-500 mt-2">
                PNG, JPG up to 10MB
              </p>
            </div>
          </div>
        }
      >
        <div class="relative">
          <img
            src={preview()!}
            alt="Plant thumbnail preview"
            class="w-full h-48 object-cover rounded-lg border border-gray-200"
          />
          <div class="absolute inset-0 bg-black bg-opacity-0 hover:bg-opacity-10 transition-colors rounded-lg flex items-center justify-center opacity-0 hover:opacity-100">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => fileInputRef?.click()}
              class="bg-white shadow-lg"
            >
              Change Photo
            </Button>
          </div>
        </div>
      </Show>

      {/* Hidden file input */}
      <input
        ref={fileInputRef}
        type="file"
        accept="image/*"
        onChange={handleFileChange}
        class="hidden"
      />

      {/* Compression loading */}
      <Show when={compressing()}>
        <div class="text-center py-4">
          <div class="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
          <p class="mt-2 text-sm text-gray-600">Compressing image...</p>
        </div>
      </Show>

      {/* Error message */}
      <Show when={props.error}>
        <p class="text-sm text-red-600">{props.error}</p>
      </Show>
    </div>
  );
};