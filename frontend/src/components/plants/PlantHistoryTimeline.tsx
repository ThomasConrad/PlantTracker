import { Component, For, Show, createEffect, createMemo, createSignal } from 'solid-js';
import { apiClient } from '@/api/client';
import { plantsStore } from '@/stores/plants';
import { Button } from '@/components/ui/Button';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { formatDateTime } from '@/utils/date';
import type { Plant, Photo, components } from '@/types';

type TrackingEntry = components['schemas']['TrackingEntry'];

type HistoryCategory = 'lifecycle' | 'care' | 'photo';

interface HistoryEvent {
  id: string;
  timestamp: string;
  title: string;
  details?: string;
  category: HistoryCategory;
}

interface PlantHistoryTimelineProps {
  plant: Plant;
}

const MAX_VISIBLE_EVENTS = 12;

const getTrackingEntryTitle = (entry: TrackingEntry, plant: Plant): string => {
  if (entry.careTaskIds && entry.careTaskIds.length > 0) {
    const task = (plant.careTasks ?? []).find(t => t.id === entry.careTaskIds![0]);
    return task ? task.name : 'Care';
  }
  if (entry.measurements && entry.measurements.length > 0) {
    const metric = plant.customMetrics.find((m) => m.id === entry.measurements![0].metricId);
    return metric ? `${metric.name} logged` : 'Measurement logged';
  }
  if (entry.photoIds && entry.photoIds.length > 0) {
    return 'Photo added';
  }
  if (entry.notes) {
    return 'Note added';
  }
  return 'Activity logged';
};

const getTrackingEntryDetails = (entry: TrackingEntry, plant: Plant): string | undefined => {
  const parts: string[] = [];

  if (entry.measurements && entry.measurements.length > 0) {
    const m = entry.measurements[0];
    const metric = plant.customMetrics.find((cm) => cm.id === m.metricId);
    const value = m.value;
    if (value !== undefined && value !== null) {
      if (typeof value === 'number') {
        const unit = metric?.unit ? ` ${metric.unit}` : '';
        parts.push(`Value: ${value}${unit}`);
      } else if (typeof value === 'boolean') {
        parts.push(`Value: ${value ? 'Yes' : 'No'}`);
      } else if (typeof value === 'string' && value.trim().length > 0) {
        parts.push(`Value: ${value}`);
      }
    }
  }

  if (entry.notes && entry.notes.trim().length > 0) {
    parts.push(entry.notes.trim());
  }

  return parts.length > 0 ? parts.join(' · ') : undefined;
};

const getCategoryBadgeClass = (category: HistoryCategory): string => {
  switch (category) {
    case 'care':
      return 'bg-blue-100 text-blue-700';
    case 'photo':
      return 'bg-amber-100 text-amber-700';
    case 'lifecycle':
      return 'bg-gray-100 text-gray-700';
    default:
      return 'bg-gray-100 text-gray-700';
  }
};

const getCategoryLabel = (category: HistoryCategory): string => {
  switch (category) {
    case 'care':
      return 'Care';
    case 'photo':
      return 'Photo';
    case 'lifecycle':
      return 'System';
    default:
      return 'Event';
  }
};

export const PlantHistoryTimeline: Component<PlantHistoryTimelineProps> = (props) => {
  const [events, setEvents] = createSignal<HistoryEvent[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [showAll, setShowAll] = createSignal(false);

  const visibleEvents = createMemo(() => {
    if (showAll()) {
      return events();
    }
    return events().slice(0, MAX_VISIBLE_EVENTS);
  });

  const loadHistory = async () => {
    try {
      setLoading(true);
      setError(null);

      const [trackingResponse, photosResponse] = await Promise.all([
        plantsStore.getTrackingEntries(props.plant.id),
        apiClient.getPlantPhotos(props.plant.id, { limit: 100 }),
      ]);

      const timelineEvents: HistoryEvent[] = [];

      timelineEvents.push({
        id: `plant-created-${props.plant.id}`,
        timestamp: props.plant.createdAt,
        title: 'Plant added',
        details: `${props.plant.name} was added to your collection`,
        category: 'lifecycle',
      });

      if (props.plant.updatedAt !== props.plant.createdAt) {
        timelineEvents.push({
          id: `plant-updated-${props.plant.id}`,
          timestamp: props.plant.updatedAt,
          title: 'Plant details updated',
          category: 'lifecycle',
        });
      }

      if (props.plant.archivedAt) {
        timelineEvents.push({
          id: `plant-archived-${props.plant.id}`,
          timestamp: props.plant.archivedAt,
          title: 'Plant archived',
          category: 'lifecycle',
        });
      }

      timelineEvents.push(
        ...trackingResponse.entries.map((entry) => ({
          id: `tracking-${entry.id}`,
          timestamp: entry.timestamp,
          title: getTrackingEntryTitle(entry, props.plant),
          details: getTrackingEntryDetails(entry, props.plant),
          category: 'care' as const,
        }))
      );

      timelineEvents.push(
        ...photosResponse.photos.map((photo: Photo) => ({
          id: `photo-${photo.id}`,
          timestamp: photo.createdAt,
          title: 'Photo added',
          details: photo.originalFilename,
          category: 'photo' as const,
        }))
      );

      timelineEvents.sort(
        (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
      );

      setEvents(timelineEvents);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Failed to load history';
      setError(message);
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    // Reload when the selected plant changes or metadata updates.
    props.plant.id;
    props.plant.updatedAt;
    props.plant.archivedAt;
    void loadHistory();
  });

  return (
    <div class="bg-white shadow-sm rounded-xl sm:rounded-2xl border border-gray-200 overflow-hidden">
      <div class="px-4 sm:px-6 py-4 sm:py-5 border-b border-gray-100 bg-gray-50/50">
        <div class="flex items-center justify-between gap-3">
          <div>
            <h3 class="text-base sm:text-lg font-semibold text-gray-900">Full History</h3>
            <p class="text-xs sm:text-sm text-gray-500">
              Complete timeline including date added and activity updates
            </p>
          </div>
          <Button variant="outline" size="sm" onClick={() => void loadHistory()} loading={loading()}>
            Refresh
          </Button>
        </div>
      </div>

      <div class="p-4 sm:p-6">
        <Show
          when={!loading()}
          fallback={
            <div class="flex justify-center py-10">
              <LoadingSpinner />
            </div>
          }
        >
          <Show when={!error()} fallback={<p class="text-sm text-red-600">{error()}</p>}>
            <Show when={events().length > 0} fallback={<p class="text-sm text-gray-500">No history yet.</p>}>
              <div class="space-y-4">
                <For each={visibleEvents()}>
                  {(event) => (
                    <div class="flex gap-3">
                      <div class="mt-1 h-2.5 w-2.5 rounded-full bg-primary-500 flex-shrink-0" />
                      <div class="min-w-0 flex-1 border-b border-gray-100 pb-4 last:border-b-0 last:pb-0">
                        <div class="flex flex-wrap items-center gap-2">
                          <p class="text-sm font-medium text-gray-900">{event.title}</p>
                          <span
                            class={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${getCategoryBadgeClass(event.category)}`}
                          >
                            {getCategoryLabel(event.category)}
                          </span>
                        </div>
                        <p class="text-xs text-gray-500 mt-0.5">{formatDateTime(event.timestamp)}</p>
                        <Show when={event.details}>
                          <p class="text-sm text-gray-600 mt-1 break-words">{event.details}</p>
                        </Show>
                      </div>
                    </div>
                  )}
                </For>

                <Show when={events().length > MAX_VISIBLE_EVENTS}>
                  <div class="pt-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setShowAll((prev) => !prev)}
                      class="w-full"
                    >
                      {showAll() ? 'Show less' : `Show all ${events().length} events`}
                    </Button>
                  </div>
                </Show>
              </div>
            </Show>
          </Show>
        </Show>
      </div>
    </div>
  );
};
