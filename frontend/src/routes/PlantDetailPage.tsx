import { Component, createEffect, createSignal, Show, onMount, onCleanup } from 'solid-js';
import { A, useParams } from '@solidjs/router';
import { plantsStore } from '@/stores/plants';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { Button } from '@/components/ui/Button';
import { ActivityLog } from '@/components/plants/ActivityLog';
import { PhotoGallery } from '@/components/plants/PhotoGallery';
import { PlantHistoryTimeline } from '@/components/plants/PlantHistoryTimeline';
import { calculateDaysUntil, formatDate, isOverdue } from '@/utils/date';
import {
  makeTransformScheduler,
  VelocityTracker,
  translateY,
  SNAP_TRANSITION,
  FLING_THRESHOLD,
  detectDirection,
} from '@/utils/touch';

export const PlantDetailPage: Component = () => {
  const params = useParams();
  const [showArchiveConfirm, setShowArchiveConfirm] = createSignal(false);
  const [archiving, setArchiving] = createSignal(false);
  const [quickActionLoading, setQuickActionLoading] = createSignal<Record<string, boolean>>({});
  const [quickActionError, setQuickActionError] = createSignal<string | null>(null);
  
  // Mobile slide-up panel state
  const [isMobile, setIsMobile] = createSignal(false);
  const [sheetOpen, setSheetOpen] = createSignal(false);
  let overlayRef: HTMLDivElement | undefined;
  let overlayScrollRef: HTMLDivElement | undefined;
  let headerRef: HTMLDivElement | undefined;

  createEffect(() => {
    if (params.id) {
      plantsStore.loadPlant(params.id);
    }
  });

  // Mobile detection and resize handler
  let cleanupGestures: (() => void) | undefined;

  const setupPlantPanelGestures = () => {
    if (!overlayRef || !overlayScrollRef) return;

    const overlay = overlayRef;
    const overlayScroll = overlayScrollRef;

    // Snap positions: fullTop = just below header, midTop = showing ~25% of panel
    const HEADER_HEIGHT = 80; // matches header style height
    const viewportH = () => window.visualViewport?.height || window.innerHeight;
    const getFullTop = () => HEADER_HEIGHT;
    const getMidTop = () => viewportH() * 0.65; // Panel collapsed, showing ~35% of viewport

    const clampOverlayTop = (y: number) => Math.max(getFullTop(), Math.min(getMidTop(), y));

    const scheduleOverlayY = makeTransformScheduler(y => {
      overlay.style.height = Math.max(120, viewportH() - HEADER_HEIGHT) + 'px';
      translateY(overlay, y);
    });

    function getLiveOverlayTop() {
      return overlay.getBoundingClientRect().top;
    }

    function snapOverlay(velocity: number, currentTop: number) {
      overlay.style.transition = SNAP_TRANSITION;
      const midTop = getMidTop();
      const fullTop = getFullTop();
      const midPoint = (fullTop + midTop) / 2;

      let snapTo: number;
      if (velocity < -FLING_THRESHOLD) {
        snapTo = fullTop;
      } else if (velocity > FLING_THRESHOLD) {
        snapTo = midTop;
      } else {
        snapTo = currentTop < midPoint ? fullTop : midTop;
      }

      overlay.style.height = Math.max(120, viewportH() - HEADER_HEIGHT) + 'px';
      translateY(overlay, snapTo);
      setSheetOpen(snapTo <= fullTop);
    }

    const overlayVelocity = new VelocityTracker();
    let dragging = false;
    let pending = false;
    let direction: 'horizontal' | 'vertical' | null = null;
    let startY = 0;
    let startX = 0;
    let startTop = 0;
    let latestTop = 0;

    const onTouchStart = (e: TouchEvent) => {
      scheduleOverlayY.cancel();
      startX = e.touches[0].clientX;
      startY = e.touches[0].clientY;
      startTop = getLiveOverlayTop();
      latestTop = startTop;
      overlayVelocity.start(startY);
      dragging = false;
      pending = true;
      direction = null;
    };

    const onTouchMove = (e: TouchEvent) => {
      if (!pending) return;
      const x = e.touches[0].clientX;
      const y = e.touches[0].clientY;
      const dx = x - startX;
      const dy = y - startY;

      if (!direction) {
        direction = detectDirection(dx, dy);
      }
      if (direction === 'horizontal') return;
      if (direction !== 'vertical') return;

      // Allow native scroll inside content when sheet is fully open
      if (!dragging && (e.target as HTMLElement)?.closest('.plant-panel-scroll')) {
        const scrollEl = overlayScroll;
        const maxScroll = scrollEl.scrollHeight - scrollEl.clientHeight;
        const atTop = getLiveOverlayTop() <= getFullTop() + 2;
        const canScrollUp = scrollEl.scrollTop > 0 && dy > 0;
        const canScrollDown = scrollEl.scrollTop < maxScroll && dy < 0;
        if (atTop && maxScroll > 1 && (canScrollUp || canScrollDown)) {
          startY = y;
          startTop = getLiveOverlayTop();
          latestTop = startTop;
          overlayVelocity.start(y);
          return;
        }
        if (!atTop || dy > 0) scrollEl.scrollTop = 0;
      }

      if (!dragging) {
        dragging = true;
        overlay.style.transition = 'none';
        startY = y;
        startTop = getLiveOverlayTop();
        latestTop = startTop;
        overlayVelocity.start(y);
      }

      e.preventDefault();
      overlayVelocity.update(y);
      latestTop = clampOverlayTop(startTop + (y - startY));
      scheduleOverlayY(startTop + (y - startY));
    };

    const onTouchEnd = () => {
      if (!pending) return;
      pending = false;
      if (!dragging) return;
      dragging = false;
      scheduleOverlayY.cancel();
      overlay.style.height = Math.max(120, viewportH() - HEADER_HEIGHT) + 'px';
      translateY(overlay, latestTop);
      snapOverlay(overlayVelocity.velocity, latestTop);
    };

    overlay.addEventListener('touchstart', onTouchStart, { passive: true });
    overlay.addEventListener('touchmove', onTouchMove, { passive: false });
    overlay.addEventListener('touchend', onTouchEnd);
    overlay.addEventListener('touchcancel', onTouchEnd);

    // Initial position (collapsed)
    requestAnimationFrame(() => {
      const midTop = getMidTop();
      overlay.style.height = Math.max(120, viewportH() - HEADER_HEIGHT) + 'px';
      translateY(overlay, midTop);
      requestAnimationFrame(() => {
        overlay.style.transition = SNAP_TRANSITION;
      });
    });

    return () => {
      overlay.removeEventListener('touchstart', onTouchStart);
      overlay.removeEventListener('touchmove', onTouchMove);
      overlay.removeEventListener('touchend', onTouchEnd);
      overlay.removeEventListener('touchcancel', onTouchEnd);
    };
  };

  onMount(() => {
    const checkMobile = () => {
      const isNarrowViewport = window.innerWidth < 768;
      const hasCoarsePointer = window.matchMedia('(pointer: coarse)').matches;
      const hasNoHover = window.matchMedia('(hover: none)').matches;
      const mobile = isNarrowViewport && (hasCoarsePointer || hasNoHover);
      setIsMobile(mobile);
    };
    
    checkMobile();
    window.addEventListener('resize', checkMobile);

    // Setup gestures after DOM is ready
    requestAnimationFrame(() => {
      if (isMobile()) {
        cleanupGestures = setupPlantPanelGestures();
      }
    });
    
    onCleanup(() => {
      window.removeEventListener('resize', checkMobile);
      cleanupGestures?.();
    });
  });

  // Re-setup gestures when mobile state changes
  createEffect(() => {
    if (isMobile()) {
      requestAnimationFrame(() => {
        cleanupGestures?.();
        cleanupGestures = setupPlantPanelGestures();
      });
    } else {
      cleanupGestures?.();
      cleanupGestures = undefined;
    }
  });

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

  const handleQuickCare = async (careTaskId: string) => {
    try {
      setQuickActionError(null);
      setQuickActionLoading((prev) => ({ ...prev, [careTaskId]: true }));
      await plantsStore.createTrackingEntry(params.id, {
        careTaskIds: [careTaskId],
        timestamp: new Date().toISOString(),
      });
    } catch (error) {
      console.error('Failed to create quick care entry:', error);
      setQuickActionError('Could not save that action. Please try again.');
    } finally {
      setQuickActionLoading((prev) => ({ ...prev, [careTaskId]: false }));
    }
  };

  const QuickCareButton: Component<{
    careTaskId: string;
    label: string;
    overdue?: boolean;
    size?: 'sm' | 'md' | 'lg';
    class?: string;
  }> = (props) => (
    <Button
      variant={props.overdue ? 'danger' : 'primary'}
      size={props.size || 'sm'}
      class={props.class}
      onClick={() => handleQuickCare(props.careTaskId)}
      loading={quickActionLoading()[props.careTaskId]}
    >
      {props.label}
    </Button>
  );

  const getCareStatus = (lastDate: string | null, intervalDays: number | null, actionLabel: string) => {
    if (!intervalDays) {
      return {
        title: `No ${actionLabel.toLowerCase()} schedule`,
        detail: 'Enable this in Edit Plant.',
        statusClass: 'border-gray-200 bg-gray-50 text-gray-700',
        badgeLabel: 'No schedule',
        badgeClass: 'bg-gray-100 text-gray-700',
      };
    }

    const overdue = isOverdue(lastDate, intervalDays);
    const days = calculateDaysUntil(lastDate, intervalDays);

    if (overdue) {
      return {
        title: `${actionLabel} overdue`,
        detail: 'Due now.',
        statusClass: 'border-red-200 bg-red-50 text-red-800',
        badgeLabel: 'Overdue',
        badgeClass: 'bg-red-100 text-red-700',
      };
    }

    if (days === 0) {
      return {
        title: `${actionLabel} today`,
        detail: 'You are on schedule.',
        statusClass: 'border-amber-200 bg-amber-50 text-amber-800',
        badgeLabel: 'Due today',
        badgeClass: 'bg-amber-100 text-amber-700',
      };
    }

    if (days === 1) {
      return {
        title: `${actionLabel} tomorrow`,
        detail: 'You are on schedule.',
        statusClass: 'border-emerald-200 bg-emerald-50 text-emerald-800',
        badgeLabel: 'Due tomorrow',
        badgeClass: 'bg-emerald-100 text-emerald-700',
      };
    }

    return {
      title: `${actionLabel} in ${days} days`,
      detail: 'You are on schedule.',
      statusClass: 'border-emerald-200 bg-emerald-50 text-emerald-800',
      badgeLabel: `In ${days} days`,
      badgeClass: 'bg-emerald-100 text-emerald-700',
    };
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
        const activeTasks = (plant.careTasks || []).filter(t => !t.archivedAt);
        const taskStatuses = activeTasks.map(task => ({
          task,
          status: getCareStatus(
            task.lastPerformed ?? null,
            task.intervalDays ?? null,
            task.name
          ),
          overdue: Boolean(task.intervalDays) && isOverdue(task.lastPerformed ?? null, task.intervalDays!),
        }));
        
        if (isMobile()) {
          // Mobile layout with slide-up panel (same pattern as calendar)
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
                <div class="absolute inset-0 bg-black/20"></div>
              </div>

              {/* Fixed header with back button and plant name */}
              <div ref={headerRef} class="relative z-10 flex items-center justify-between p-4 bg-gradient-to-b from-black/50 to-transparent flex-shrink-0" style={{ height: '80px' }}>
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
                    href={`/plants/${plant.id}/coach`}
                    class="p-2 rounded-full bg-white/20 backdrop-blur-sm text-white hover:bg-white/30 transition-colors"
                    title="Plant Coach"
                  >
                    <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
                    </svg>
                  </A>
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

              {/* Slide-up overlay panel */}
              <div class="mc-agenda-overlay plant-detail-overlay" ref={overlayRef}>
                <button class="mc-agenda-handle" onClick={() => {
                  setSheetOpen(!sheetOpen());
                  if (overlayRef) {
                    const viewportH = window.visualViewport?.height || window.innerHeight;
                    const fullTop = 80;
                    const midTop = viewportH * 0.65;
                    overlayRef.style.transition = SNAP_TRANSITION;
                    overlayRef.style.height = Math.max(120, viewportH - 80) + 'px';
                    translateY(overlayRef, sheetOpen() ? fullTop : midTop);
                  }
                }}></button>

                <div class="plant-panel-scroll" ref={overlayScrollRef} style={{
                  flex: '1',
                  'overflow-y': 'auto',
                  'overflow-x': 'hidden',
                  '-webkit-overflow-scrolling': 'touch',
                  'overscroll-behavior': 'contain',
                  'min-height': '0',
                  'padding-bottom': '120px',
                }}>
                  <div class="space-y-5">
                    {/* Quick care buttons */}
                    <div class="sticky top-0 z-40 bg-white border-b border-gray-100 shadow-sm px-4 py-3">
                      <div class={`grid gap-3 ${taskStatuses.length > 1 ? 'grid-cols-2' : 'grid-cols-1'}`}>
                        {taskStatuses.map(({ task, overdue }) => (
                          <QuickCareButton
                            careTaskId={task.id}
                            label={`${task.icon || ''} ${task.name} now`.trim()}
                            overdue={overdue}
                            size="md"
                            class="min-h-[44px] w-full"
                          />
                        ))}
                      </div>
                      <Show when={quickActionError()}>
                        <p class="text-xs text-red-600 mt-2">{quickActionError()}</p>
                      </Show>
                    </div>

                    <div class="px-4 space-y-6">
                      <section class="bg-white shadow-sm rounded-xl border border-gray-200 overflow-hidden">
                        <div class="px-4 py-4 border-b border-gray-100 bg-gray-50/60">
                          <h2 class="text-base font-semibold text-gray-900">Care Today</h2>
                        </div>
                        <div class="p-4 space-y-3">
                          {taskStatuses.map(({ task, status }) => (
                            <div class={`rounded-xl border p-3 ${status.statusClass}`}>
                              <div class="flex items-start justify-between gap-3">
                                <div class="min-w-0">
                                  <div class="flex items-center gap-2">
                                    <span class="text-sm">{task.icon || '🌱'}</span>
                                    <p class="text-sm font-semibold">{status.title}</p>
                                  </div>
                                  <p class="text-xs mt-1 opacity-90">{status.detail}</p>
                                  <Show when={task.lastPerformed}>
                                    <p class="text-xs mt-2 opacity-80">Last: {formatDate(task.lastPerformed!)}</p>
                                  </Show>
                                </div>
                                <span class={`inline-flex items-center rounded-full px-2 py-1 text-[11px] font-semibold ${status.badgeClass}`}>
                                  {status.badgeLabel}
                                </span>
                              </div>
                            </div>
                          ))}
                          <Show when={taskStatuses.length === 0}>
                            <p class="text-sm text-gray-500">No care tasks configured.</p>
                          </Show>
                        </div>
                      </section>

                      <PhotoGallery
                        plantId={plant.id}
                        mode="preview"
                        fullTimelineHref={`/plants/${plant.id}/photos`}
                      />

                      <details class="bg-white shadow-sm rounded-xl border border-gray-200 overflow-hidden">
                        <summary class="px-4 py-4 cursor-pointer list-none select-none flex items-center justify-between gap-3 bg-gray-50/60">
                          <h2 class="text-base font-semibold text-gray-900">Advanced</h2>
                          <span class="text-xs text-gray-500">Expand</span>
                        </summary>
                        <div class="p-4 border-t border-gray-100 space-y-4">
                          <ActivityLog plant={plant} />
                          <PlantHistoryTimeline plant={plant} />
                        </div>
                      </details>
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
                          href={`/plants/${plant.id}/coach`}
                          class="inline-flex items-center gap-2 px-3 sm:px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500 transition-all duration-200 shadow-sm"
                        >
                          <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
                          </svg>
                          Coach
                        </A>
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
                  <div class="max-w-5xl mx-auto space-y-6">
                        <section class="bg-white shadow-sm rounded-xl sm:rounded-2xl border border-gray-200 overflow-hidden">
                          <div class="px-4 sm:px-6 py-4 sm:py-5 border-b border-gray-100 bg-gray-50/50">
                            <h2 class="text-base sm:text-lg font-semibold text-gray-900">Care Today</h2>
                          </div>
                          <div class="p-4 sm:p-6 space-y-4">
                            {taskStatuses.map(({ task, status, overdue }) => (
                              <div class={`rounded-xl border p-4 ${status.statusClass}`}>
                                <div class="flex items-start justify-between gap-4">
                                  <div>
                                    <p class="text-sm font-semibold">{task.icon || '🌱'} {status.title}</p>
                                    <p class="text-xs mt-1 opacity-90">{status.detail}</p>
                                    <Show when={task.lastPerformed}>
                                      <p class="text-xs mt-2 opacity-80">Last: {formatDate(task.lastPerformed!)}</p>
                                    </Show>
                                  </div>
                                  <QuickCareButton careTaskId={task.id} label={`${task.name} now`} overdue={overdue} />
                                </div>
                              </div>
                            ))}
                            <Show when={taskStatuses.length === 0}>
                              <p class="text-sm text-gray-500">No care tasks configured.</p>
                            </Show>
                            <Show when={quickActionError()}>
                              <p class="text-sm text-red-600">{quickActionError()}</p>
                            </Show>
                          </div>
                        </section>

                        <PhotoGallery
                          plantId={plant.id}
                          mode="preview"
                          fullTimelineHref={`/plants/${plant.id}/photos`}
                        />

                        <details class="bg-white shadow-sm rounded-xl sm:rounded-2xl border border-gray-200 overflow-hidden">
                          <summary class="px-4 sm:px-6 py-4 sm:py-5 cursor-pointer list-none select-none flex items-center justify-between gap-3 bg-gray-50/50">
                            <h2 class="text-base sm:text-lg font-semibold text-gray-900">Advanced</h2>
                            <span class="text-sm text-gray-500">Expand</span>
                          </summary>
                          <div class="p-4 sm:p-6 border-t border-gray-100 space-y-6">
                            <ActivityLog plant={plant} />
                            <PlantHistoryTimeline plant={plant} />
                          </div>
                        </details>
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
