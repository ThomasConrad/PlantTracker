import { Component, createEffect, createSignal, Show, onMount, onCleanup } from 'solid-js';
import { A, useParams } from '@solidjs/router';
import { plantsStore } from '@/stores/plants';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { Button } from '@/components/ui/Button';
import { PlantCareStatus } from '@/components/plants/PlantCareStatus';
import { ActivityLog } from '@/components/plants/ActivityLog';
import { PhotoGallery } from '@/components/plants/PhotoGallery';
import { PlantHistoryTimeline } from '@/components/plants/PlantHistoryTimeline';
import { formatDate } from '@/utils/date';

export const PlantDetailPage: Component = () => {
  const params = useParams();
  const [showArchiveConfirm, setShowArchiveConfirm] = createSignal(false);
  const [archiving, setArchiving] = createSignal(false);
  
  // Mobile swipe-up panel state
  const [isMobile, setIsMobile] = createSignal(false);
  const [panelOffset, setPanelOffset] = createSignal(60); // Start at 60% of screen height
  const [isDragging, setIsDragging] = createSignal(false);
  let panelContentRef: HTMLDivElement | undefined;
  let lastTouchY = 0;
  let lastTouchTime = 0;
  let touchVelocityY = 0;
  const PANEL_EXPANDED_OFFSET = 12;
  const PANEL_DEFAULT_OFFSET = 60;
  const PANEL_COLLAPSED_OFFSET = 90;
  const PANEL_SNAP_EPSILON = 1;

  const normalizePanelOffset = (offset: number) => {
    if (offset <= PANEL_EXPANDED_OFFSET + PANEL_SNAP_EPSILON) {
      return PANEL_EXPANDED_OFFSET;
    }
    if (offset >= PANEL_COLLAPSED_OFFSET - PANEL_SNAP_EPSILON) {
      return PANEL_COLLAPSED_OFFSET;
    }
    return offset;
  };

  const clampPanelOffset = (offset: number) =>
    normalizePanelOffset(
      Math.max(PANEL_EXPANDED_OFFSET, Math.min(PANEL_COLLAPSED_OFFSET, offset))
    );

  createEffect(() => {
    if (!isMobile()) return;
    if (panelOffset() > PANEL_EXPANDED_OFFSET + PANEL_SNAP_EPSILON && panelContentRef) {
      panelContentRef.scrollTop = 0;
    }
  });

  createEffect(() => {
    if (params.id) {
      plantsStore.loadPlant(params.id);
    }
  });

  // Mobile detection and resize handler
  onMount(() => {
    const checkMobile = () => {
      const isNarrowViewport = window.innerWidth < 768;
      const hasCoarsePointer = window.matchMedia('(pointer: coarse)').matches;
      const hasNoHover = window.matchMedia('(hover: none)').matches;
      setIsMobile(isNarrowViewport && (hasCoarsePointer || hasNoHover));
    };
    
    checkMobile();
    window.addEventListener('resize', checkMobile);
    
    onCleanup(() => {
      window.removeEventListener('resize', checkMobile);
    });
  });

  // Touch event handlers for mobile swipe panel
  const handlePanelTouchStart = (e: TouchEvent) => {
    if (!isMobile()) return;
    const now = performance.now();
    setIsDragging(true);
    lastTouchY = e.touches[0].clientY;
    lastTouchTime = now;
    touchVelocityY = 0;
  };

  const handleTouchMove = (e: TouchEvent) => {
    if (!isMobile() || !isDragging()) return;

    const currentY = e.touches[0].clientY;
    const now = performance.now();
    const stepDeltaY = currentY - lastTouchY;
    const activeScrollTop = panelContentRef?.scrollTop || 0;
    const expanded = panelOffset() <= PANEL_EXPANDED_OFFSET + PANEL_SNAP_EPSILON;
    const dt = Math.max(1, now - lastTouchTime);
    const instantaneousVelocity = stepDeltaY / dt;
    touchVelocityY = touchVelocityY * 0.7 + instantaneousVelocity * 0.3;
    lastTouchY = currentY;
    lastTouchTime = now;
    if (Math.abs(stepDeltaY) < 0.5) return;

    const contentAreaHeight = window.innerHeight - 80; // 80px header height
    const offsetDelta = (stepDeltaY / contentAreaHeight) * 100;
    // Simple model:
    // 1) If sheet is not expanded, gesture always moves sheet.
    // 2) If expanded, content scrolls normally except pull-down at content top collapses sheet.
    if (!expanded) {
      e.preventDefault();
      if (panelContentRef) {
        panelContentRef.scrollTop = 0;
      }
      setPanelOffset((prev) => clampPanelOffset(prev + offsetDelta));
      return;
    }

    if (stepDeltaY > 0 && activeScrollTop <= 0.5) {
      e.preventDefault();
      if (panelContentRef) {
        panelContentRef.scrollTop = 0;
      }
      setPanelOffset((prev) => clampPanelOffset(prev + offsetDelta));
    }
  };

  const handleTouchEnd = () => {
    if (!isMobile() || !isDragging()) return;
    const releaseVelocityY = touchVelocityY;
    setIsDragging(false);
    lastTouchY = 0;
    lastTouchTime = 0;
    touchVelocityY = 0;
    
    // Project velocity to choose nearest snap point without abrupt jumps.
    const currentOffset = panelOffset();
    const absVelocity = Math.abs(releaseVelocityY);
    const projectedOffset =
      absVelocity >= 0.15
        ? clampPanelOffset(currentOffset + releaseVelocityY * 18)
        : currentOffset;
    const snapPoints = [PANEL_EXPANDED_OFFSET, PANEL_DEFAULT_OFFSET, PANEL_COLLAPSED_OFFSET];
    const nearestSnap = snapPoints.reduce((closest, point) =>
      Math.abs(point - projectedOffset) < Math.abs(closest - projectedOffset) ? point : closest
    );

    if (nearestSnap <= PANEL_EXPANDED_OFFSET + 0.5) {
      setPanelOffset(PANEL_EXPANDED_OFFSET);
    } else if (nearestSnap >= PANEL_COLLAPSED_OFFSET - 0.5) {
      setPanelOffset(PANEL_COLLAPSED_OFFSET);
    } else {
      setPanelOffset(PANEL_DEFAULT_OFFSET);
    }
  };

  const handlePanelWheel = (e: WheelEvent) => {
    if (e.defaultPrevented) return;

    const contentAreaHeight = window.innerHeight - 80;
    const currentOffset = panelOffset();
    const scrollTop = panelContentRef?.scrollTop || 0;
    const expanded = currentOffset <= PANEL_EXPANDED_OFFSET + PANEL_SNAP_EPSILON;
    const collapsed = currentOffset >= PANEL_COLLAPSED_OFFSET - PANEL_SNAP_EPSILON;
    const deltaY = e.deltaY;

    // Consume inertial wheel at hard boundaries without introducing any delay.
    const pushingPastBottom = collapsed && deltaY < 0;
    if (pushingPastBottom) {
      e.preventDefault();
      if (panelContentRef) {
        panelContentRef.scrollTop = 0;
      }
      return;
    }

    // While not expanded, wheel always moves the sheet.
    if (!expanded) {
      e.preventDefault();
      if (panelContentRef) {
        panelContentRef.scrollTop = 0;
      }
      setPanelOffset(clampPanelOffset(currentOffset - (deltaY / contentAreaHeight) * 100));
      return;
    }

    // When expanded and content is at top, scrolling up collapses the sheet.
    if (deltaY < 0 && scrollTop <= 0.5) {
      e.preventDefault();
      if (panelContentRef) {
        panelContentRef.scrollTop = 0;
      }
      setPanelOffset(clampPanelOffset(currentOffset - (deltaY / contentAreaHeight) * 100));
    }
  };

  const handleArchiveToggle = async () => {
    try {
      setArchiving(true);
      const plant = plantsStore.selectedPlant;
      if (!plant) return;

      if (plant.archivedAt) {
        await plantsStore.unarchivePlant(params.id);
      } else {
        await plantsStore.archivePlant(params.id);
      }
    } catch (error) {
      console.error('Failed to update archive status:', error);
    } finally {
      setArchiving(false);
      setShowArchiveConfirm(false);
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
            <div class="h-full flex flex-col overflow-hidden relative">
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
              <div class="relative z-10 flex items-center justify-between p-4 bg-gradient-to-b from-black/50 to-transparent flex-shrink-0" style={{ height: '80px' }}>
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
                <div class="flex items-center gap-2">
                  <A
                    href={`/plants/${plant.id}/edit`}
                    class="p-2 rounded-full bg-white/20 backdrop-blur-sm text-white hover:bg-white/30 transition-colors"
                  >
                    <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                  </A>
                  <button
                    type="button"
                    onClick={() => setShowArchiveConfirm(true)}
                    class="p-2 rounded-full bg-white/20 backdrop-blur-sm text-white hover:bg-white/30 transition-colors"
                    aria-label={plant.archivedAt ? 'Unarchive plant' : 'Archive plant'}
                  >
                    <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 8h14M7 8V6a1 1 0 011-1h8a1 1 0 011 1v2m-9 4h8m-8 4h8M6 8v10a2 2 0 002 2h8a2 2 0 002-2V8" />
                    </svg>
                  </button>
                </div>
              </div>

              {/* Content area below header */}
              <div
                class="flex-1 relative overflow-hidden"
                style={{ 'overscroll-behavior': 'none' }}
                onWheel={handlePanelWheel}
              >
                {/* Swipe-up content panel */}
                <div 
                  class={`absolute inset-x-0 bottom-0 bg-white rounded-t-3xl shadow-2xl z-30 ${
                    isDragging() ? '' : 'transition-transform duration-300 ease-out'
                  }`}
                  style={{
                    bottom: '4rem',
                    transform: `translateY(${panelOffset()}%)`,
                    height: `${100 - panelOffset() + 10}%`,
                    'min-height': '20%',
                    'touch-action':
                      panelOffset() > PANEL_EXPANDED_OFFSET + PANEL_SNAP_EPSILON
                        ? 'none'
                        : 'pan-y',
                  }}
                  onTouchStart={handlePanelTouchStart}
                  onTouchMove={handleTouchMove}
                  onTouchEnd={handleTouchEnd}
                  onTouchCancel={handleTouchEnd}
                >
                  {/* Content */}
                  <div
                    ref={panelContentRef}
                    class="px-4 pb-8 flex-1"
                    style={{
                      height: '100%',
                      'padding-top': '1rem',
                      'overflow-y':
                        panelOffset() > PANEL_EXPANDED_OFFSET + PANEL_SNAP_EPSILON
                          ? 'hidden'
                          : 'auto',
                      'overscroll-behavior': 'contain',
                    }}
                  >
                    <div class="space-y-6">
                      <PlantCareStatus plant={plant} />
                      <ActivityLog plant={plant} />
                      <PlantHistoryTimeline plant={plant} />
                    </div>
                  </div>
                </div>
              </div>

              <Show when={showArchiveConfirm()}>
                <div class="fixed inset-0 bg-gray-900/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
                  <div class="bg-white rounded-2xl max-w-md w-full shadow-2xl ring-1 ring-gray-200">
                    <div class="px-6 py-5 border-b border-gray-100">
                      <h3 class="text-lg font-semibold text-gray-900">
                        {plant.archivedAt ? 'Unarchive Plant' : 'Archive Plant'}
                      </h3>
                      <p class="text-sm text-gray-500">
                        {plant.archivedAt ? 'This plant will appear in your active list again.' : 'This plant will be hidden from the default list.'}
                      </p>
                    </div>
                    <div class="px-6 py-4">
                      <p class="text-sm text-gray-600 leading-relaxed">
                        {plant.archivedAt
                          ? `Unarchive "${plant.name}"?`
                          : `Archive "${plant.name}"? You can unarchive it later.`}
                      </p>
                    </div>
                    <div class="px-6 py-4 bg-gray-50 rounded-b-2xl flex justify-end space-x-3">
                      <Button
                        variant="outline"
                        onClick={() => setShowArchiveConfirm(false)}
                        disabled={archiving()}
                        class="shadow-sm"
                      >
                        Cancel
                      </Button>
                      <Button
                        variant="primary"
                        onClick={handleArchiveToggle}
                        loading={archiving()}
                        disabled={archiving()}
                        class="shadow-sm"
                      >
                        {plant.archivedAt ? 'Unarchive' : 'Archive'}
                      </Button>
                    </div>
                  </div>
                </div>
              </Show>
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
                          variant="outline"
                          size="sm"
                          onClick={() => setShowArchiveConfirm(true)}
                          loading={archiving()}
                          disabled={archiving()}
                          class="shadow-sm"
                        >
                          <svg class="h-4 w-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 8h14M7 8V6a1 1 0 011-1h8a1 1 0 011 1v2m-9 4h8m-8 4h8M6 8v10a2 2 0 002 2h8a2 2 0 002-2V8" />
                          </svg>
                          {plant.archivedAt ? 'Unarchive' : 'Archive'}
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
                        <PlantHistoryTimeline plant={plant} />
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

              <Show when={showArchiveConfirm()}>
                <div class="fixed inset-0 bg-gray-900/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
                  <div class="bg-white rounded-2xl max-w-md w-full shadow-2xl ring-1 ring-gray-200">
                    <div class="px-6 py-5 border-b border-gray-100">
                      <h3 class="text-lg font-semibold text-gray-900">
                        {plant.archivedAt ? 'Unarchive Plant' : 'Archive Plant'}
                      </h3>
                      <p class="text-sm text-gray-500">
                        {plant.archivedAt ? 'This plant will appear in your active list again.' : 'This plant will be hidden from the default list.'}
                      </p>
                    </div>
                    <div class="px-6 py-4">
                      <p class="text-sm text-gray-600 leading-relaxed">
                        {plant.archivedAt
                          ? `Unarchive "${plant.name}"?`
                          : `Archive "${plant.name}"? You can unarchive it later.`}
                      </p>
                    </div>
                    <div class="px-6 py-4 bg-gray-50 rounded-b-2xl flex justify-end space-x-3">
                      <Button
                        variant="outline"
                        onClick={() => setShowArchiveConfirm(false)}
                        disabled={archiving()}
                        class="shadow-sm"
                      >
                        Cancel
                      </Button>
                      <Button
                        variant="primary"
                        onClick={handleArchiveToggle}
                        loading={archiving()}
                        disabled={archiving()}
                        class="shadow-sm"
                      >
                        {plant.archivedAt ? 'Unarchive' : 'Archive'}
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
