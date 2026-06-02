import { A } from "@solidjs/router";
import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  For,
  Show,
} from "solid-js";
import { apiClient } from "@/api/client";
import { Button } from "@/components/ui/Button";
import { LoadingSpinner } from "@/components/ui/LoadingSpinner";
import { plantsStore } from "@/stores/plants";
import type { Photo } from "@/types";

interface PhotoGalleryProps {
  plantId: string;
  mode?: "preview" | "full";
  fullTimelineHref?: string;
}

type GalleryView = "timeline" | "grid" | "compare";

const IMAGE_FALLBACK =
  "data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQiIGhlaWdodD0iMjQiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZD0iTTQgMTZMOC41ODYgMTEuNDE0QzguOTYxIDExLjAzOSA5LjQ1OSAxMC44MjkgMTAgMTAuODI5QzEwLjU0MSAxMC44MjkgMTEuMDM5IDExLjAzOSAxMS40MTQgMTEuNDE0TDE2IDE2TTE0IDE0TDE1LjU4NiAxMi40MTRDMTUuOTYxIDEyLjAzOSAxNi40NTkgMTEuODI5IDE3IDExLjgyOUMxNy41NDEgMTEuODI5IDE4LjAzOSAxMi4wMzkgMTguNDE0IDEyLjQxNEwyMCAxNE0xOCA4VjhNNiAyMEgxOEMxOC41MzA0IDIwIDE5LjAzOTEgMTkuNzg5MyAxOS40MTQyIDE5LjQxNDJDMTkuNzg5MyAxOS4wMzkxIDIwIDE4LjUzMDQgMjAgMThWNkMyMCA1LjQ2OTU3IDE5Ljc4OTMgNC45NjA4NiAxOS40MTQyIDQuNTg1NzlDMTkuMDM5MSA0LjIxMDcxIDE4LjUzMDQgNCA4IDRINkM1LjQ2OTU3IDQgNC45NjA4NiA0LjIxMDcxIDQuNTg1NzkgNC41ODU3OUM0LjIxMDcxIDQuOTYwODYgNCA1LjQ2OTU3IDQgNlYxOEM0IDE4LjUzMDQgNC4yMTA3MSAxOS4wMzkxIDQuNTg1NzkgMTkuNDE0MkM0Ljk2MDg2IDE5Ljc4OTMgNS40Njk1NyAyMCA2IDIwWiIgc3Ryb2tlPSJjdXJyZW50Q29sb3IiIHN0cm9rZS13aWR0aD0iMiIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIi8+Cjwvc3ZnPgo=";

const formatMonthLabel = (date: string): string =>
  new Intl.DateTimeFormat("en-US", {
    month: "long",
    year: "numeric",
  }).format(new Date(date));

const formatFullDate = (date: string): string =>
  new Intl.DateTimeFormat("en-US", {
    weekday: "long",
    month: "long",
    day: "numeric",
    year: "numeric",
  }).format(new Date(date));

const dayDiffFromNow = (date: string): number => {
  const now = new Date();
  const current = new Date(date);
  const todayStart = new Date(
    now.getFullYear(),
    now.getMonth(),
    now.getDate(),
  ).getTime();
  const currentStart = new Date(
    current.getFullYear(),
    current.getMonth(),
    current.getDate(),
  ).getTime();

  return Math.round((todayStart - currentStart) / (1000 * 60 * 60 * 24));
};

const formatTimelineDate = (date: string): string => {
  const diff = dayDiffFromNow(date);
  const d = new Date(date);

  if (diff === 0) return "Today";
  if (diff === 1) return "Yesterday";
  if (diff < 7) {
    return new Intl.DateTimeFormat("en-US", {
      weekday: "long",
    }).format(d);
  }
  if (d.getFullYear() === new Date().getFullYear()) {
    return new Intl.DateTimeFormat("en-US", {
      month: "short",
      day: "numeric",
    }).format(d);
  }

  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(d);
};

const getDaysBetween = (older: string, newer: string): number => {
  const diff = new Date(newer).getTime() - new Date(older).getTime();
  return Math.max(0, Math.round(diff / (1000 * 60 * 60 * 24)));
};

const formatGap = (days: number): string => {
  if (days <= 0) return "Same day";
  if (days === 1) return "1 day later";
  if (days < 14) return `${days} days later`;
  if (days < 60) return `${Math.round(days / 7)} weeks later`;

  const months = Math.max(2, Math.round(days / 30));
  return `${months} months later`;
};

export const PhotoGallery: Component<PhotoGalleryProps> = (props) => {
  const mode = () => props.mode ?? "full";
  const [photos, setPhotos] = createSignal<Photo[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [uploading, setUploading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [view, setView] = createSignal<GalleryView>("timeline");

  const [selectedPhotoIndex, setSelectedPhotoIndex] = createSignal<
    number | null
  >(null);
  const [showPhotoModal, setShowPhotoModal] = createSignal(false);
  const [modalActionLoading, setModalActionLoading] = createSignal<
    "preview" | "delete" | null
  >(null);
  const [compareLeft, setCompareLeft] = createSignal<number>(0);
  const [compareRight, setCompareRight] = createSignal<number>(0);

  let fileInputRef: HTMLInputElement | undefined;

  const photosByNewest = createMemo(() =>
    [...photos()].sort(
      (a, b) =>
        new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
    ),
  );

  const previewPhotos = createMemo(() => photosByNewest().slice(0, 6));
  const photosByOldest = createMemo(() => [...photosByNewest()].reverse());

  // Initialize compare indices when photos change
  createEffect(() => {
    const all = photosByOldest();
    if (all.length >= 2) {
      setCompareLeft(0);
      setCompareRight(all.length - 1);
    }
  });

  const timelinePhotos = createMemo(() => {
    let lastDate: string | null = null;
    return photosByOldest().map((photo) => {
      const daysSincePrevious = lastDate
        ? getDaysBetween(lastDate, photo.createdAt)
        : null;
      lastDate = photo.createdAt;
      return {
        photo,
        daysSincePrevious,
        monthLabel: formatMonthLabel(photo.createdAt),
      };
    });
  });

  const currentPhoto = createMemo(() => {
    const index = selectedPhotoIndex();
    if (index === null) return null;
    return photosByNewest()[index] ?? null;
  });

  createEffect(() => {
    void loadPhotos();
  });

  createEffect(() => {
    if (showPhotoModal()) {
      document.addEventListener("keydown", handleKeyPress);
      document.body.style.overflow = "hidden";
    } else {
      document.removeEventListener("keydown", handleKeyPress);
      document.body.style.overflow = "auto";
      setModalActionLoading(null);
    }

    return () => {
      document.removeEventListener("keydown", handleKeyPress);
      document.body.style.overflow = "auto";
    };
  });

  const loadPhotos = async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await apiClient.getPlantPhotos(props.plantId, {
        limit: mode() === "full" ? 120 : 24,
      });
      setPhotos(response.photos);
    } catch (err: unknown) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to load photos";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleFileSelect = async (files: FileList | null) => {
    if (!files || files.length === 0) return;

    const file = files[0];
    if (!file.type.startsWith("image/")) {
      setError("Please select a valid image file");
      return;
    }

    if (file.size > 10 * 1024 * 1024) {
      setError("File size must be less than 10MB");
      return;
    }

    try {
      setUploading(true);
      setError(null);
      const newPhoto = await apiClient.uploadPlantPhoto(props.plantId, file);
      setPhotos((prev) => [newPhoto, ...prev]);
    } catch (err: unknown) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to upload photo";
      setError(errorMessage);
    } finally {
      setUploading(false);
      if (fileInputRef) {
        fileInputRef.value = "";
      }
    }
  };

  const handleDeletePhoto = async (photoId: string, fromModal = false) => {
    try {
      if (fromModal) {
        setModalActionLoading("delete");
      }
      await apiClient.deletePlantPhoto(props.plantId, photoId);
      setPhotos((prev) => prev.filter((photo) => photo.id !== photoId));
      setError(null);

      if (fromModal) {
        const remaining = photosByNewest().filter(
          (photo) => photo.id !== photoId,
        );
        if (remaining.length === 0) {
          closePhotoModal();
          return;
        }

        setSelectedPhotoIndex((prev) => {
          if (prev === null) return prev;
          return Math.min(prev, remaining.length - 1);
        });
      }
    } catch (err: unknown) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to delete photo";
      setError(errorMessage);
    } finally {
      if (fromModal) {
        setModalActionLoading(null);
      }
    }
  };

  const handleSetPreview = async (photoId: string, fromModal = false) => {
    try {
      if (fromModal) {
        setModalActionLoading("preview");
      }
      setError(null);
      await plantsStore.setPlantPreview(props.plantId, photoId);
    } catch (err: unknown) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to set preview";
      setError(errorMessage);
    } finally {
      if (fromModal) {
        setModalActionLoading(null);
      }
    }
  };

  const openPhotoModal = (photoIndex: number) => {
    setSelectedPhotoIndex(photoIndex);
    setShowPhotoModal(true);
  };

  const openPhotoModalById = (photoId: string) => {
    const index = photosByNewest().findIndex((photo) => photo.id === photoId);
    if (index >= 0) {
      openPhotoModal(index);
    }
  };

  const closePhotoModal = () => {
    setShowPhotoModal(false);
    setSelectedPhotoIndex(null);
  };

  const navigatePhoto = (direction: "prev" | "next") => {
    const currentIndex = selectedPhotoIndex();
    if (currentIndex === null) return;

    const totalPhotos = photosByNewest().length;
    const newIndex =
      direction === "prev"
        ? currentIndex > 0
          ? currentIndex - 1
          : totalPhotos - 1
        : currentIndex < totalPhotos - 1
          ? currentIndex + 1
          : 0;

    setSelectedPhotoIndex(newIndex);
  };

  const handleKeyPress = (e: KeyboardEvent) => {
    if (!showPhotoModal()) return;

    switch (e.key) {
      case "ArrowLeft":
        navigatePhoto("prev");
        break;
      case "ArrowRight":
        navigatePhoto("next");
        break;
      case "Escape":
        closePhotoModal();
        break;
    }
  };

  return (
    <div class="bg-white shadow-sm rounded-xl sm:rounded-2xl border border-gray-200 overflow-hidden">
      <div class="px-4 sm:px-6 py-4 sm:py-5 border-b border-gray-100 bg-gray-50/50">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h3 class="text-base sm:text-lg font-semibold text-gray-900">
              {mode() === "full" ? "Growth Timeline" : "Recent Photos"}
            </h3>
            <p class="text-xs sm:text-sm text-gray-500">
              {mode() === "full"
                ? "A visual timeline of this plant over time"
                : "Quick look at your latest progress shots"}
            </p>
          </div>

          <div class="flex items-center gap-2">
            <Show when={mode() === "full"}>
              <div class="inline-flex rounded-lg border border-gray-200 bg-white p-1">
                <button
                  type="button"
                  onClick={() => setView("timeline")}
                  class={`px-3 py-1.5 text-xs sm:text-sm rounded-md transition-colors ${
                    view() === "timeline"
                      ? "bg-emerald-600 text-white shadow-sm"
                      : "text-gray-600 hover:text-gray-900 hover:bg-gray-50"
                  }`}
                >
                  Timeline
                </button>
                <button
                  type="button"
                  onClick={() => setView("grid")}
                  class={`px-3 py-1.5 text-xs sm:text-sm rounded-md transition-colors ${
                    view() === "grid"
                      ? "bg-emerald-600 text-white shadow-sm"
                      : "text-gray-600 hover:text-gray-900 hover:bg-gray-50"
                  }`}
                >
                  Grid
                </button>
                <button
                  type="button"
                  onClick={() => setView("compare")}
                  class={`px-3 py-1.5 text-xs sm:text-sm rounded-md transition-colors ${
                    view() === "compare"
                      ? "bg-emerald-600 text-white shadow-sm"
                      : "text-gray-600 hover:text-gray-900 hover:bg-gray-50"
                  }`}
                >
                  Compare
                </button>
              </div>
            </Show>

            <Button
              variant="outline"
              size="sm"
              onClick={() => fileInputRef?.click()}
              loading={uploading()}
            >
              <svg
                class="mr-2 h-4 w-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width={2}
                  d="M12 4v16m8-8H4"
                />
              </svg>
              Add Photo
            </Button>

            <Show when={mode() === "preview" && props.fullTimelineHref}>
              <A
                href={props.fullTimelineHref!}
                class="inline-flex items-center rounded-md border border-emerald-200 bg-emerald-50 px-3 py-2 text-xs sm:text-sm font-medium text-emerald-700 hover:bg-emerald-100 transition-colors"
              >
                Full timeline
              </A>
            </Show>
          </div>
        </div>
      </div>

      <div class="p-4 sm:p-6">
        <input
          ref={fileInputRef}
          type="file"
          accept="image/*"
          class="hidden"
          onChange={(e) => void handleFileSelect(e.currentTarget.files)}
        />

        <Show when={error()}>
          <div class="mb-4 bg-red-50 border border-red-200 rounded-md p-3">
            <p class="text-sm text-red-600">{error()}</p>
          </div>
        </Show>

        <Show
          when={!loading()}
          fallback={
            <div class="flex justify-center py-10">
              <LoadingSpinner />
            </div>
          }
        >
          <Show
            when={photosByNewest().length > 0}
            fallback={
              <div class="text-center py-10">
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
                <h4 class="mt-2 text-sm font-medium text-gray-900">
                  No photos yet
                </h4>
                <p class="mt-1 text-sm text-gray-500">
                  Start documenting your plant's growth.
                </p>
              </div>
            }
          >
            <Show
              when={mode() === "preview"}
              fallback={
                <>
                  <Show
                    when={view() === "compare" && photosByOldest().length >= 2}
                  >
                    <div class="space-y-4">
                      <div class="grid grid-cols-2 gap-4">
                        {/* Left (older) photo */}
                        <div>
                          <p class="text-xs font-medium text-gray-500 mb-2 text-center">
                            Before
                          </p>
                          <div class="relative rounded-xl overflow-hidden border border-gray-200">
                            <img
                              src={apiClient.getPhotoUrl(
                                props.plantId,
                                photosByOldest()[compareLeft()].id,
                              )}
                              alt="Before"
                              class="w-full aspect-square object-cover"
                              loading="lazy"
                              onError={(e) => {
                                e.currentTarget.src = IMAGE_FALLBACK;
                              }}
                            />
                            <div class="absolute bottom-2 left-2 bg-black/55 text-white text-xs px-2 py-1 rounded-full">
                              {formatTimelineDate(
                                photosByOldest()[compareLeft()].createdAt,
                              )}
                            </div>
                          </div>
                          <input
                            type="range"
                            min={0}
                            max={photosByOldest().length - 1}
                            value={compareLeft()}
                            onInput={(e) =>
                              setCompareLeft(
                                Math.min(
                                  parseInt(e.currentTarget.value),
                                  compareRight() - 1,
                                ),
                              )
                            }
                            class="w-full mt-2 accent-emerald-600"
                          />
                        </div>
                        {/* Right (newer) photo */}
                        <div>
                          <p class="text-xs font-medium text-gray-500 mb-2 text-center">
                            After
                          </p>
                          <div class="relative rounded-xl overflow-hidden border border-gray-200">
                            <img
                              src={apiClient.getPhotoUrl(
                                props.plantId,
                                photosByOldest()[compareRight()].id,
                              )}
                              alt="After"
                              class="w-full aspect-square object-cover"
                              loading="lazy"
                              onError={(e) => {
                                e.currentTarget.src = IMAGE_FALLBACK;
                              }}
                            />
                            <div class="absolute bottom-2 left-2 bg-black/55 text-white text-xs px-2 py-1 rounded-full">
                              {formatTimelineDate(
                                photosByOldest()[compareRight()].createdAt,
                              )}
                            </div>
                          </div>
                          <input
                            type="range"
                            min={0}
                            max={photosByOldest().length - 1}
                            value={compareRight()}
                            onInput={(e) =>
                              setCompareRight(
                                Math.max(
                                  parseInt(e.currentTarget.value),
                                  compareLeft() + 1,
                                ),
                              )
                            }
                            class="w-full mt-2 accent-emerald-600"
                          />
                        </div>
                      </div>
                      <div class="text-center">
                        <span class="inline-flex items-center gap-2 text-sm text-gray-600 bg-gray-50 px-3 py-1.5 rounded-full">
                          <svg
                            class="w-4 h-4"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                          >
                            <path
                              stroke-linecap="round"
                              stroke-linejoin="round"
                              stroke-width="2"
                              d="M13 7l5 5m0 0l-5 5m5-5H6"
                            />
                          </svg>
                          {formatGap(
                            getDaysBetween(
                              photosByOldest()[compareLeft()].createdAt,
                              photosByOldest()[compareRight()].createdAt,
                            ),
                          )}
                        </span>
                      </div>
                    </div>
                  </Show>

                  <Show when={view() === "timeline"}>
                    <div class="relative pl-4 sm:pl-6">
                      <div class="absolute left-[7px] sm:left-[11px] top-0 bottom-0 w-px bg-emerald-200" />
                      <For each={timelinePhotos()}>
                        {(item, index) => {
                          const previous = timelinePhotos()[index() - 1];
                          const showMonth =
                            !previous ||
                            previous.monthLabel !== item.monthLabel;

                          return (
                            <div class="relative pb-8 last:pb-0">
                              <Show when={item.daysSincePrevious !== null}>
                                <div class="pl-6 sm:pl-8 mb-3">
                                  <span class="inline-flex items-center rounded-full bg-gray-100 text-gray-600 px-2.5 py-1 text-xs font-medium border border-gray-200">
                                    {formatGap(item.daysSincePrevious!)}
                                  </span>
                                </div>
                              </Show>

                              <div class="absolute left-[-1px] sm:left-[3px] top-2 h-4 w-4 rounded-full border-2 border-white bg-emerald-500 shadow" />

                              <Show when={showMonth}>
                                <div class="mb-3 pl-6 sm:pl-8">
                                  <span class="inline-flex items-center rounded-full bg-emerald-50 text-emerald-700 px-2.5 py-1 text-xs font-medium border border-emerald-200">
                                    {item.monthLabel}
                                  </span>
                                </div>
                              </Show>

                              <div class="pl-6 sm:pl-8">
                                <button
                                  type="button"
                                  class="block w-full rounded-xl border border-gray-200 bg-white overflow-hidden shadow-sm text-left"
                                  onClick={() =>
                                    openPhotoModalById(item.photo.id)
                                  }
                                >
                                  <img
                                    src={apiClient.getPhotoUrl(
                                      props.plantId,
                                      item.photo.id,
                                    )}
                                    alt={`Plant photo from ${formatFullDate(item.photo.createdAt)}`}
                                    class="w-full h-52 sm:h-64 object-cover"
                                    loading="lazy"
                                    onError={(e) => {
                                      e.currentTarget.src = IMAGE_FALLBACK;
                                    }}
                                  />
                                  <div class="p-3 sm:p-4 flex items-center justify-between gap-2">
                                    <p class="text-sm font-semibold text-gray-900">
                                      {formatTimelineDate(item.photo.createdAt)}
                                    </p>
                                    <p class="text-xs text-gray-500">
                                      {formatFullDate(item.photo.createdAt)}
                                    </p>
                                  </div>
                                </button>
                              </div>
                            </div>
                          );
                        }}
                      </For>
                    </div>
                  </Show>

                  <Show when={view() === "grid"}>
                    <div class="grid grid-cols-2 sm:grid-cols-3 gap-3">
                      <For each={photosByNewest()}>
                        {(photo, index) => (
                          <button
                            type="button"
                            class="relative group overflow-hidden rounded-lg border border-gray-200 bg-gray-100 text-left"
                            onClick={() => openPhotoModal(index())}
                          >
                            <img
                              src={apiClient.getPhotoUrl(
                                props.plantId,
                                photo.id,
                              )}
                              alt={`Plant photo from ${formatFullDate(photo.createdAt)}`}
                              class="w-full h-36 sm:h-40 object-cover cursor-pointer group-hover:scale-[1.02] transition-transform"
                              loading="lazy"
                              onError={(e) => {
                                e.currentTarget.src = IMAGE_FALLBACK;
                              }}
                            />
                            <div class="absolute left-2 bottom-2 bg-black/55 text-white text-xs px-2 py-1 rounded-full">
                              {formatTimelineDate(photo.createdAt)}
                            </div>
                          </button>
                        )}
                      </For>
                    </div>
                  </Show>
                </>
              }
            >
              <div class="grid grid-cols-2 sm:grid-cols-3 gap-3">
                <For each={previewPhotos()}>
                  {(photo, index) => (
                    <button
                      type="button"
                      class="relative group overflow-hidden rounded-lg border border-gray-200 bg-gray-100 text-left"
                      onClick={() => openPhotoModal(index())}
                    >
                      <img
                        src={apiClient.getPhotoUrl(props.plantId, photo.id)}
                        alt={`Plant photo from ${formatFullDate(photo.createdAt)}`}
                        class="w-full h-28 sm:h-32 object-cover group-hover:scale-[1.02] transition-transform"
                        loading="lazy"
                        onError={(e) => {
                          e.currentTarget.src = IMAGE_FALLBACK;
                        }}
                      />
                      <div class="absolute left-2 bottom-2 bg-black/55 text-white text-xs px-2 py-1 rounded-full">
                        {formatTimelineDate(photo.createdAt)}
                      </div>
                    </button>
                  )}
                </For>
              </div>
            </Show>
          </Show>
        </Show>
      </div>

      <Show
        when={
          showPhotoModal() && selectedPhotoIndex() !== null && currentPhoto()
        }
      >
        <div
          class="fixed inset-0 bg-black/90 flex items-center justify-center z-[70]"
          onClick={closePhotoModal}
        >
          <div class="relative w-full h-full flex items-center justify-center p-4">
            <button
              type="button"
              onClick={closePhotoModal}
              class="absolute top-4 right-4 z-20 text-white hover:text-gray-300 transition-colors p-2"
              aria-label="Close"
            >
              <svg
                class="h-8 w-8"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>

            <div class="absolute top-4 left-4 z-20 bg-black/50 text-white px-3 py-1 rounded-full text-sm">
              {selectedPhotoIndex()! + 1} of {photosByNewest().length}
            </div>

            <img
              src={apiClient.getPhotoUrl(props.plantId, currentPhoto()!.id)}
              alt={`Plant photo from ${formatFullDate(currentPhoto()!.createdAt)}`}
              class="max-w-full max-h-full object-contain"
              onClick={(e) => e.stopPropagation()}
            />

            <Show when={photosByNewest().length > 1}>
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  navigatePhoto("prev");
                }}
                class="absolute left-4 top-1/2 -translate-y-1/2 text-white hover:text-gray-300 transition-colors p-2 bg-black/30 rounded-full"
                aria-label="Previous photo"
              >
                <svg
                  class="h-8 w-8"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width={2}
                    d="M15 19l-7-7 7-7"
                  />
                </svg>
              </button>

              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  navigatePhoto("next");
                }}
                class="absolute right-4 top-1/2 -translate-y-1/2 text-white hover:text-gray-300 transition-colors p-2 bg-black/30 rounded-full"
                aria-label="Next photo"
              >
                <svg
                  class="h-8 w-8"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width={2}
                    d="M9 5l7 7-7 7"
                  />
                </svg>
              </button>
            </Show>

            <div
              class="absolute bottom-4 left-1/2 -translate-x-1/2 w-[min(92vw,38rem)] bg-black/55 text-white px-4 py-3 rounded-xl"
              onClick={(e) => e.stopPropagation()}
            >
              <div class="flex flex-wrap items-center justify-between gap-2">
                <p class="text-sm font-medium">
                  {formatFullDate(currentPhoto()!.createdAt)}
                </p>
                <p class="text-xs text-gray-200">
                  {(currentPhoto()!.size / 1024 / 1024).toFixed(1)}MB
                  <Show when={currentPhoto()!.width && currentPhoto()!.height}>
                    <span>
                      {" "}
                      • {currentPhoto()!.width}x{currentPhoto()!.height}
                    </span>
                  </Show>
                </p>
              </div>
              <div class="mt-3 flex items-center gap-2">
                <Button
                  variant="primary"
                  size="sm"
                  loading={modalActionLoading() === "preview"}
                  onClick={() =>
                    void handleSetPreview(currentPhoto()!.id, true)
                  }
                >
                  Set as preview
                </Button>
                <Button
                  variant="danger"
                  size="sm"
                  loading={modalActionLoading() === "delete"}
                  onClick={() =>
                    void handleDeletePhoto(currentPhoto()!.id, true)
                  }
                >
                  Delete
                </Button>
              </div>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
