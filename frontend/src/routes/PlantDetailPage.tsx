import { Component, createEffect, createSignal, Show, onMount, onCleanup } from 'solid-js';
import { A, useNavigate, useParams } from '@solidjs/router';
import { plantsStore } from '@/stores/plants';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { Button } from '@/components/ui/Button';
import { PlantCareStatus } from '@/components/plants/PlantCareStatus';
import { ActivityLog } from '@/components/plants/ActivityLog';
import { PhotoGallery } from '@/components/plants/PhotoGallery';
import { formatDate } from '@/utils/date';

export const PlantDetailPage: Component = () => {
  const navigate = useNavigate();
  const params = useParams();
  const [showDeleteConfirm, setShowDeleteConfirm] = createSignal(false);
  const [deleting, setDeleting] = createSignal(false);
  
  // Mobile swipe-up panel state
  const [isMobile, setIsMobile] = createSignal(false);
  const [panelOffset, setPanelOffset] = createSignal(60); // Start at 60% of screen height
  const [isDragging, setIsDragging] = createSignal(false);
  const [dragStartY, setDragStartY] = createSignal(0);
  const [dragStartOffset, setDragStartOffset] = createSignal(0);

  createEffect(() => {
    if (params.id) {
      plantsStore.loadPlant(params.id);
    }
  });

  // Mobile detection and resize handler
  onMount(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768);
    };
    
    checkMobile();
    window.addEventListener('resize', checkMobile);
    
    onCleanup(() => {
      window.removeEventListener('resize', checkMobile);
    });
  });

  // Touch event handlers for mobile swipe panel
  const handleTouchStart = (e: TouchEvent) => {
    if (!isMobile()) return;
    setIsDragging(true);
    setDragStartY(e.touches[0].clientY);
    setDragStartOffset(panelOffset());
  };

  const handleTouchMove = (e: TouchEvent) => {
    if (!isMobile() || !isDragging()) return;
    
    const currentY = e.touches[0].clientY;
    const deltaY = currentY - dragStartY();
    const windowHeight = window.innerHeight;
    const deltaPercent = (deltaY / windowHeight) * 100;
    
    const newOffset = Math.max(10, Math.min(90, dragStartOffset() + deltaPercent));
    setPanelOffset(newOffset);
  };

  const handleTouchEnd = () => {
    if (!isMobile() || !isDragging()) return;
    setIsDragging(false);
    
    // Snap to positions based on final offset
    const currentOffset = panelOffset();
    if (currentOffset < 30) {
      setPanelOffset(10); // Fully expanded
    } else if (currentOffset > 70) {
      setPanelOffset(85); // Mostly hidden
    } else {
      setPanelOffset(60); // Default position
    }
  };

  const handleDelete = async () => {
    try {
      setDeleting(true);
      await plantsStore.deletePlant(params.id);
      navigate('/plants');
    } catch (error) {
      console.error('Failed to delete plant:', error);
    } finally {
      setDeleting(false);
      setShowDeleteConfirm(false);
    }
  };

  return (
    <Show
      when={plantsStore.selectedPlant && !plantsStore.loading}
      fallback={
        <div class="flex justify-center py-16 sm:py-20">
          <div class="flex flex-col items-center gap-4">
            <LoadingSpinner size="lg" />
            <div class="text-center">
              <p class="text-sm sm:text-base text-gray-500 font-medium">
                Loading plant details...
              </p>
              <p class="text-xs sm:text-sm text-gray-400 mt-1">
                Please wait while we fetch your plant information
              </p>
            </div>
          </div>
        </div>
      }
    >
      {(() => {
        const plant = plantsStore.selectedPlant!;
        
        if (isMobile()) {
          // Mobile layout with swipe-up panel
          return (
            <div class="h-screen flex flex-col overflow-hidden">
              {/* Full-screen plant preview background */}
              <div class="absolute inset-0">
                <Show when={plant.previewUrl} fallback={
                  <div class="w-full h-full bg-gradient-to-br from-green-400 via-green-500 to-green-600 flex items-center justify-center">
                    <svg class="h-24 w-24 text-white/50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={1.5} d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
                    </svg>
                  </div>
                }>
                  <img 
                    src={plant.previewUrl!} 
                    alt={plant.name}
                    class="w-full h-full object-cover"
                  />
                </Show>
                {/* Overlay gradient */}
                <div class="absolute inset-0 bg-black/20"></div>
              </div>

              {/* Fixed header with back button and plant name */}
              <div class="relative z-10 flex items-center justify-between p-4 bg-gradient-to-b from-black/50 to-transparent">
                <A
                  href="/plants"
                  class="p-2 rounded-full bg-white/20 backdrop-blur-sm text-white hover:bg-white/30 transition-colors"
                >
                  <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 19l-7-7 7-7" />
                  </svg>
                </A>
                <div class="text-center">
                  <h1 class="text-xl font-bold text-white drop-shadow-lg">{plant.name}</h1>
                  <p class="text-sm text-white/80 italic">{plant.genus}</p>
                </div>
                <A
                  href={`/plants/${plant.id}/edit`}
                  class="p-2 rounded-full bg-white/20 backdrop-blur-sm text-white hover:bg-white/30 transition-colors"
                >
                  <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                </A>
              </div>

              {/* Swipe-up content panel */}
              <div 
                class="relative z-20 bg-white rounded-t-3xl shadow-2xl transition-transform duration-300 ease-out"
                style={{
                  transform: `translateY(${panelOffset()}vh)`,
                  height: `${100 - panelOffset() + 10}vh`,
                  'min-height': '20vh'
                }}
                onTouchStart={handleTouchStart}
                onTouchMove={handleTouchMove}
                onTouchEnd={handleTouchEnd}
              >
                {/* Drag handle */}
                <div class="flex justify-center pt-3 pb-1">
                  <div class="w-12 h-1.5 bg-gray-300 rounded-full"></div>
                </div>
                
                {/* Content */}
                <div class="px-4 pb-8 overflow-y-auto h-full">
                  <div class="space-y-6">
                    <PlantCareStatus plant={plant} />
                    <ActivityLog plant={plant} />
                  </div>
                </div>
              </div>
            </div>
          );
        } else {
          // Desktop layout (existing structure)
          return (
            <>
              <div class="min-h-full pb-20 sm:pb-8">
                {/* Header Section with improved mobile margins */}
                <div class="px-4 sm:px-6 pt-4 sm:pt-6 pb-6">
                  <div class="max-w-7xl mx-auto">
                    <div class="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-4 sm:gap-6">
                      <div class="flex-1 min-w-0">
                        <div class="flex items-center space-x-3 sm:space-x-4">
                          <A
                            href="/plants"
                            class="flex-shrink-0 p-2 -ml-2 text-gray-400 hover:text-gray-600 hover:bg-gray-50 rounded-lg transition-colors duration-200"
                            aria-label="Back to plants"
                          >
                            <svg class="h-5 w-5 sm:h-6 sm:w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 19l-7-7 7-7" />
                            </svg>
                          </A>
                          <div class="min-w-0 flex-1">
                            <div class="flex items-center gap-3 mb-1">
                              <h1 class="text-xl sm:text-2xl lg:text-3xl font-bold text-gray-900 truncate">
                                {plant.name}
                              </h1>
                              <div class="flex-shrink-0">
                                <div class="w-2 h-2 bg-green-400 rounded-full ring-2 ring-green-100"></div>
                              </div>
                            </div>
                            <p class="text-sm sm:text-base text-gray-600 italic font-medium">
                              {plant.genus}
                            </p>
                            <div class="flex items-center gap-4 mt-2">
                              <div class="text-xs sm:text-sm text-gray-500">
                                Added {formatDate(plant.createdAt)}
                              </div>
                            </div>
                          </div>
                        </div>
                      </div>
                      
                      {/* Action Buttons */}
                      <div class="flex items-center gap-2 sm:gap-3 flex-shrink-0">
                        <A
                          href={`/plants/${plant.id}/edit`}
                          class="inline-flex items-center gap-2 px-3 sm:px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500 transition-all duration-200 shadow-sm"
                        >
                          <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                          </svg>
                          Edit
                        </A>
                        <Button
                          variant="danger"
                          size="sm"
                          onClick={() => setShowDeleteConfirm(true)}
                          class="shadow-sm"
                        >
                          <svg class="h-4 w-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                          </svg>
                          Delete
                        </Button>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Main Content */}
                <div class="px-4 sm:px-6">
                  <div class="max-w-7xl mx-auto">
                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 lg:gap-8">
                      {/* Plant Care Status and Activity Log - Takes up 2 columns on large screens */}
                      <div class="lg:col-span-2 space-y-6">
                        <PlantCareStatus plant={plant} />
                        <ActivityLog plant={plant} />
                      </div>
                      
                      {/* Plant Info Sidebar */}
                      <div class="space-y-6">
                        {/* Plant Information Card */}
                        <div class="bg-white shadow-sm rounded-xl sm:rounded-2xl border border-gray-200 overflow-hidden">
                          <div class="px-4 sm:px-6 py-4 sm:py-5 border-b border-gray-100 bg-gray-50/50">
                            <div class="flex items-center space-x-3">
                              <div class="flex-shrink-0">
                                <div class="w-8 h-8 sm:w-10 sm:h-10 bg-primary-100 rounded-lg flex items-center justify-center">
                                  <svg class="h-4 w-4 sm:h-5 sm:w-5 text-primary-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                  </svg>
                                </div>
                              </div>
                              <div>
                                <h3 class="text-base sm:text-lg font-semibold text-gray-900">
                                  Plant Information
                                </h3>
                                <p class="text-xs sm:text-sm text-gray-500">
                                  Care schedule and details
                                </p>
                              </div>
                            </div>
                          </div>
                          
                          <div class="p-4 sm:p-6 space-y-5">
                            <div class="space-y-1">
                              <div class="flex items-center justify-between">
                                <label class="text-sm font-medium text-gray-500">Watering Schedule</label>
                                <Show when={plant.wateringSchedule?.intervalDays} fallback={
                                  <div class="flex items-center gap-1.5">
                                    <div class="w-2 h-2 bg-gray-300 rounded-full"></div>
                                    <span class="text-xs text-gray-400">Inactive</span>
                                  </div>
                                }>
                                  <div class="flex items-center gap-1.5">
                                    <div class="w-2 h-2 bg-blue-400 rounded-full"></div>
                                    <span class="text-xs text-gray-400">Active</span>
                                  </div>
                                </Show>
                              </div>
                              <Show when={plant.wateringSchedule?.intervalDays} fallback={<p class="text-sm text-gray-500 italic">No watering schedule</p>}>
                                <p class="text-sm text-gray-900 font-medium">Every {plant.wateringSchedule.intervalDays} days</p>
                                <Show when={plant.wateringSchedule.amount}>
                                  <p class="text-xs text-gray-600">{plant.wateringSchedule.amount}{plant.wateringSchedule.unit}</p>
                                </Show>
                                <Show when={plant.wateringSchedule.notes}>
                                  <p class="text-xs text-gray-600 italic">{plant.wateringSchedule.notes}</p>
                                </Show>
                              </Show>
                              {plant.lastWatered && (
                                <p class="text-xs text-gray-500">Last watered: {formatDate(plant.lastWatered)}</p>
                              )}
                            </div>
                            
                            <div class="border-t border-gray-100 pt-4">
                              <div class="space-y-1">
                                <div class="flex items-center justify-between">
                                  <label class="text-sm font-medium text-gray-500">Fertilizing Schedule</label>
                                  <Show when={plant.fertilizingSchedule?.intervalDays} fallback={
                                    <div class="flex items-center gap-1.5">
                                      <div class="w-2 h-2 bg-gray-300 rounded-full"></div>
                                      <span class="text-xs text-gray-400">Inactive</span>
                                    </div>
                                  }>
                                    <div class="flex items-center gap-1.5">
                                      <div class="w-2 h-2 bg-green-400 rounded-full"></div>
                                      <span class="text-xs text-gray-400">Active</span>
                                    </div>
                                  </Show>
                                </div>
                                <Show when={plant.fertilizingSchedule?.intervalDays} fallback={<p class="text-sm text-gray-500 italic">No fertilizing schedule</p>}>
                                  <p class="text-sm text-gray-900 font-medium">Every {plant.fertilizingSchedule.intervalDays} days</p>
                                  <Show when={plant.fertilizingSchedule.amount}>
                                    <p class="text-xs text-gray-600">{plant.fertilizingSchedule.amount}{plant.fertilizingSchedule.unit}</p>
                                  </Show>
                                  <Show when={plant.fertilizingSchedule.notes}>
                                    <p class="text-xs text-gray-600 italic">{plant.fertilizingSchedule.notes}</p>
                                  </Show>
                                </Show>
                                {plant.lastFertilized && (
                                  <p class="text-xs text-gray-500">Last fertilized: {formatDate(plant.lastFertilized)}</p>
                                )}
                              </div>
                            </div>
                            
                            <div class="border-t border-gray-100 pt-4">
                              <div class="space-y-1">
                                <label class="text-sm font-medium text-gray-500">Date Added</label>
                                <p class="text-sm text-gray-900 font-medium">{formatDate(plant.createdAt)}</p>
                              </div>
                            </div>
                          </div>
                        </div>

                        {/* Photo Gallery Card */}
                        <div class="bg-white shadow-sm rounded-xl sm:rounded-2xl border border-gray-200 overflow-hidden">
                          <div class="px-4 sm:px-6 py-4 sm:py-5 border-b border-gray-100 bg-gray-50/50">
                            <div class="flex items-center space-x-3">
                              <div class="flex-shrink-0">
                                <div class="w-8 h-8 sm:w-10 sm:h-10 bg-primary-100 rounded-lg flex items-center justify-center">
                                  <svg class="h-4 w-4 sm:h-5 sm:w-5 text-primary-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                  </svg>
                                </div>
                              </div>
                              <div>
                                <h3 class="text-base sm:text-lg font-semibold text-gray-900">
                                  Photo Gallery
                                </h3>
                                <p class="text-xs sm:text-sm text-gray-500">
                                  Plant photos and memories
                                </p>
                              </div>
                            </div>
                          </div>
                          <div class="p-4 sm:p-6">
                            <PhotoGallery plantId={plant.id} />
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              {/* Enhanced Delete Confirmation Modal */}
              <Show when={showDeleteConfirm()}>
                <div class="fixed inset-0 bg-gray-900/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
                  <div class="bg-white rounded-2xl max-w-md w-full shadow-2xl ring-1 ring-gray-200">
                    {/* Modal Header */}
                    <div class="px-6 py-5 border-b border-gray-100">
                      <div class="flex items-center space-x-3">
                        <div class="flex-shrink-0">
                          <div class="w-10 h-10 bg-red-100 rounded-lg flex items-center justify-center">
                            <svg class="h-5 w-5 text-red-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                            </svg>
                          </div>
                        </div>
                        <div>
                          <h3 class="text-lg font-semibold text-gray-900">
                            Delete Plant
                          </h3>
                          <p class="text-sm text-gray-500">
                            This action cannot be undone
                          </p>
                        </div>
                      </div>
                    </div>
                    
                    {/* Modal Content */}
                    <div class="px-6 py-4">
                      <p class="text-sm text-gray-600 leading-relaxed">
                        Are you sure you want to delete <strong class="font-semibold text-gray-900">"{plant.name}"</strong>? 
                        This will permanently remove all plant data, photos, and care history.
                      </p>
                    </div>
                    
                    {/* Modal Actions */}
                    <div class="px-6 py-4 bg-gray-50 rounded-b-2xl flex justify-end space-x-3">
                      <Button
                        variant="outline"
                        onClick={() => setShowDeleteConfirm(false)}
                        disabled={deleting()}
                        class="shadow-sm"
                      >
                        Cancel
                      </Button>
                      <Button
                        variant="danger"
                        onClick={handleDelete}
                        loading={deleting()}
                        disabled={deleting()}
                        class="shadow-sm"
                      >
                        <Show when={!deleting()}>
                          Delete Plant
                        </Show>
                        <Show when={deleting()}>
                          Deleting...
                        </Show>
                      </Button>
                    </div>
                  </div>
                </div>
              </Show>
            </>
          );
        }
      })()}
    </Show>
  );
};
