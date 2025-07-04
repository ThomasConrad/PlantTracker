import { Component, createSignal, createEffect, Show, For } from 'solid-js';
import { apiClient } from '@/api/client';
import { Button } from '@/components/ui/Button';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { plantsStore } from '@/stores/plants';
import type { Photo } from '@/types';

interface PhotoGalleryProps {
  plantId: string;
}

export const PhotoGallery: Component<PhotoGalleryProps> = (props) => {
  const [photos, setPhotos] = createSignal<Photo[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [uploading, setUploading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  
  // Full-screen photo modal state
  const [selectedPhotoIndex, setSelectedPhotoIndex] = createSignal<number | null>(null);
  const [showPhotoModal, setShowPhotoModal] = createSignal(false);

  let fileInputRef: HTMLInputElement | undefined;

  createEffect(() => {
    loadPhotos();
  });

  // Add keyboard event listeners for photo modal
  createEffect(() => {
    if (showPhotoModal()) {
      document.addEventListener('keydown', handleKeyPress);
      document.body.style.overflow = 'hidden'; // Prevent background scrolling
    } else {
      document.removeEventListener('keydown', handleKeyPress);
      document.body.style.overflow = 'auto';
    }
    
    // Cleanup on unmount
    return () => {
      document.removeEventListener('keydown', handleKeyPress);
      document.body.style.overflow = 'auto';
    };
  });

  const loadPhotos = async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await apiClient.getPlantPhotos(props.plantId, { limit: 20 });
      setPhotos(response.photos);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load photos';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleFileSelect = async (files: FileList | null) => {
    if (!files || files.length === 0) return;

    const file = files[0];
    if (!file.type.startsWith('image/')) {
      setError('Please select a valid image file');
      return;
    }

    // Check file size (10MB limit)
    if (file.size > 10 * 1024 * 1024) {
      setError('File size must be less than 10MB');
      return;
    }

    try {
      setUploading(true);
      setError(null);
      const newPhoto = await apiClient.uploadPlantPhoto(props.plantId, file);
      setPhotos(prev => [newPhoto, ...prev]);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to upload photo';
      setError(errorMessage);
    } finally {
      setUploading(false);
      if (fileInputRef) {
        fileInputRef.value = '';
      }
    }
  };

  const handleDeletePhoto = async (photoId: string) => {
    try {
      await apiClient.deletePlantPhoto(props.plantId, photoId);
      setPhotos(prev => prev.filter(photo => photo.id !== photoId));
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to delete photo';
      setError(errorMessage);
    }
  };

  const handleSetPreview = async (photoId: string) => {
    try {
      setError(null);
      await plantsStore.setPlantPreview(props.plantId, photoId);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to set preview';
      setError(errorMessage);
    }
  };

  const openPhotoModal = (photoIndex: number) => {
    setSelectedPhotoIndex(photoIndex);
    setShowPhotoModal(true);
  };

  const closePhotoModal = () => {
    setShowPhotoModal(false);
    setSelectedPhotoIndex(null);
  };

  const navigatePhoto = (direction: 'prev' | 'next') => {
    const currentIndex = selectedPhotoIndex();
    if (currentIndex === null) return;
    
    const totalPhotos = photos().length;
    let newIndex;
    
    if (direction === 'prev') {
      newIndex = currentIndex > 0 ? currentIndex - 1 : totalPhotos - 1;
    } else {
      newIndex = currentIndex < totalPhotos - 1 ? currentIndex + 1 : 0;
    }
    
    setSelectedPhotoIndex(newIndex);
  };

  const handleKeyPress = (e: KeyboardEvent) => {
    if (!showPhotoModal()) return;
    
    switch (e.key) {
      case 'ArrowLeft':
        navigatePhoto('prev');
        break;
      case 'ArrowRight':
        navigatePhoto('next');
        break;
      case 'Escape':
        closePhotoModal();
        break;
    }
  };

  return (
    <div class="card">
      <div class="card-header">
        <div class="flex items-center justify-between">
          <h3 class="text-lg font-medium text-gray-900">Photos</h3>
          <Button
            variant="outline"
            size="sm"
            onClick={() => fileInputRef?.click()}
            loading={uploading()}
          >
            <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 4v16m8-8H4" />
            </svg>
            Add Photo
          </Button>
        </div>
      </div>
      
      <div class="card-body">
        <input
          ref={fileInputRef}
          type="file"
          accept="image/*"
          class="hidden"
          onChange={(e) => handleFileSelect(e.currentTarget.files)}
        />

        {error() && (
          <div class="mb-4 bg-red-50 border border-red-200 rounded-md p-3">
            <p class="text-sm text-red-600">{error()}</p>
          </div>
        )}

        <Show
          when={!loading()}
          fallback={
            <div class="flex justify-center py-8">
              <LoadingSpinner />
            </div>
          }
        >
          <Show
            when={photos().length > 0}
            fallback={
              <div class="text-center py-8">
                <svg
                  class="mx-auto h-12 w-12 text-gray-400"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width={2}
                    d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
                  />
                </svg>
                <h4 class="mt-2 text-sm font-medium text-gray-900">No photos yet</h4>
                <p class="mt-1 text-sm text-gray-500">
                  Document your plant's growth with photos.
                </p>
                <div class="mt-4">
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={() => fileInputRef?.click()}
                  >
                    Upload your first photo
                  </Button>
                </div>
              </div>
            }
          >
            <div class="grid grid-cols-2 gap-3">
              <For each={photos()}>
                {(photo, index) => (
                  <div class="relative group">
                    <img
                      src={apiClient.getPhotoUrl(props.plantId, photo.id)}
                      alt={photo.originalFilename}
                      class="w-full h-24 object-cover rounded-lg bg-gray-100 cursor-pointer hover:opacity-90 transition-opacity"
                      loading="lazy"
                      onClick={() => openPhotoModal(index())}
                      onError={(e) => {
                        // If image fails to load, show a placeholder
                        e.currentTarget.src = 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQiIGhlaWdodD0iMjQiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZD0iTTQgMTZMOC41ODYgMTEuNDE0QzguOTYxIDExLjAzOSA5LjQ1OSAxMC44MjkgMTAgMTAuODI5QzEwLjU0MSAxMC44MjkgMTEuMDM5IDExLjAzOSAxMS40MTQgMTEuNDE0TDE2IDE2TTE0IDE0TDE1LjU4NiAxMi40MTRDMTUuOTYxIDEyLjAzOSAxNi40NTkgMTEuODI5IDE3IDExLjgyOUMxNy41NDEgMTEuODI5IDE4LjAzOSAxMi4wMzkgMTguNDE0IDEyLjQxNEwyMCAxNE0xOCA4VjhNNiAyMEgxOEMxOC41MzA0IDIwIDE5LjAzOTEgMTkuNzg5MyAxOS40MTQyIDE5LjQxNDJDMTkuNzg5MyAxOS4wMzkxIDIwIDE4LjUzMDQgMjAgMThWNkMyMCA1LjQ2OTU3IDE5Ljc4OTMgNC45NjA4NiAxOS40MTQyIDQuNTg1NzlDMTkuMDM5MSA0LjIxMDcxIDE4LjUzMDQgNCA4IDRINkM1LjQ2OTU3IDQgNC45NjA4NiA0LjIxMDcxIDQuNTg1NzkgNC41ODU3OUM0LjIxMDcxIDQuOTYwODYgNCA1LjQ2OTU3IDQgNlYxOEM0IDE4LjUzMDQgNC4yMTA3MSAxOS4wMzkxIDQuNTg1NzkgMTkuNDE0MkM0Ljk2MDg2IDE5Ljc4OTMgNS40Njk1NyAyMCA2IDIwWiIgc3Ryb2tlPSJjdXJyZW50Q29sb3IiIHN0cm9rZS13aWR0aD0iMiIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIi8+Cjwvc3ZnPgo=';
                      }}
                    />
                    <div class="absolute top-1 right-1 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleSetPreview(photo.id);
                        }}
                        class="bg-blue-600 text-white rounded-full w-6 h-6 flex items-center justify-center hover:bg-blue-700 transition-colors"
                        aria-label="Set as preview"
                        title="Set as preview"
                      >
                        <svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                        </svg>
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDeletePhoto(photo.id);
                        }}
                        class="bg-red-600 text-white rounded-full w-6 h-6 flex items-center justify-center hover:bg-red-700 transition-colors"
                        aria-label="Delete photo"
                      >
                        <svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    </div>
                    <div class="absolute bottom-1 left-1 bg-black bg-opacity-75 text-white text-xs px-1 py-0.5 rounded opacity-0 group-hover:opacity-100 transition-opacity">
                      <p class="truncate max-w-20">{photo.originalFilename}</p>
                      <p class="text-gray-300">{(photo.size / 1024).toFixed(1)}KB</p>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </Show>
      </div>
      
      {/* Full-screen Photo Modal */}
      <Show when={showPhotoModal() && selectedPhotoIndex() !== null}>
        {(() => {
          const currentPhoto = photos()[selectedPhotoIndex()!];
          const totalPhotos = photos().length;
          const currentIndex = selectedPhotoIndex()! + 1;
          
          return (
            <div 
              class="fixed inset-0 bg-black bg-opacity-90 flex items-center justify-center z-50"
              onClick={closePhotoModal}
            >
              <div class="relative w-full h-full flex items-center justify-center p-4">
                {/* Close button */}
                <button
                  onClick={closePhotoModal}
                  class="absolute top-4 right-4 z-10 text-white hover:text-gray-300 transition-colors p-2"
                  aria-label="Close"
                >
                  <svg class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>

                {/* Photo counter */}
                <div class="absolute top-4 left-4 z-10 bg-black bg-opacity-50 text-white px-3 py-1 rounded-full text-sm">
                  {currentIndex} of {totalPhotos}
                </div>

                {/* Main photo */}
                <img
                  src={apiClient.getPhotoUrl(props.plantId, currentPhoto.id)}
                  alt={currentPhoto.originalFilename}
                  class="max-w-full max-h-full object-contain"
                  onClick={(e) => e.stopPropagation()}
                />

                {/* Navigation arrows - only show if more than 1 photo */}
                <Show when={totalPhotos > 1}>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      navigatePhoto('prev');
                    }}
                    class="absolute left-4 top-1/2 -translate-y-1/2 text-white hover:text-gray-300 transition-colors p-2 bg-black bg-opacity-30 rounded-full"
                    aria-label="Previous photo"
                  >
                    <svg class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 19l-7-7 7-7" />
                    </svg>
                  </button>
                  
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      navigatePhoto('next');
                    }}
                    class="absolute right-4 top-1/2 -translate-y-1/2 text-white hover:text-gray-300 transition-colors p-2 bg-black bg-opacity-30 rounded-full"
                    aria-label="Next photo"
                  >
                    <svg class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 5l7 7-7 7" />
                    </svg>
                  </button>
                </Show>

                {/* Photo info */}
                <div class="absolute bottom-4 left-1/2 -translate-x-1/2 bg-black bg-opacity-50 text-white px-4 py-2 rounded-lg text-center">
                  <p class="font-medium">{currentPhoto.originalFilename}</p>
                  <p class="text-sm text-gray-300">
                    {(currentPhoto.size / 1024 / 1024).toFixed(1)}MB
                    {currentPhoto.width && currentPhoto.height && (
                      <span> • {currentPhoto.width}×{currentPhoto.height}</span>
                    )}
                  </p>
                </div>
              </div>
            </div>
          );
        })()}
      </Show>
    </div>
  );
};