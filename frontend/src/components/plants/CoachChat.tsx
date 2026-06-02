import {
  Component,
  createSignal,
  createEffect,
  For,
  Show,
  onMount,
} from "solid-js";
import { coachApi, CoachMessage, CoachSuggestion } from "@/api/coach";
import { apiClient } from "@/api/client";
import { Button } from "@/components/ui/Button";
import { LoadingSpinner } from "@/components/ui/LoadingSpinner";
import { compressImage } from "@/utils/imageCompress";
import { renderMarkdown } from "@/utils/markdown";
import type { Photo } from "@/types/api";

interface CoachChatProps {
  plantId: string;
  plantName: string;
}

export const CoachChat: Component<CoachChatProps> = (props) => {
  const [messages, setMessages] = createSignal<CoachMessage[]>([]);
  const [input, setInput] = createSignal("");
  const [pendingImage, setPendingImage] = createSignal<string | undefined>();
  const [loading, setLoading] = createSignal(true);
  const [sending, setSending] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [showGallery, setShowGallery] = createSignal(false);
  const [galleryPhotos, setGalleryPhotos] = createSignal<Photo[]>([]);
  const [galleryLoading, setGalleryLoading] = createSignal(false);

  let messagesEndRef: HTMLDivElement | undefined;
  let fileInputRef: HTMLInputElement | undefined;

  const scrollToBottom = () => {
    messagesEndRef?.scrollIntoView({ behavior: "smooth" });
  };

  onMount(async () => {
    try {
      const response = await coachApi.getMessages(props.plantId);
      setMessages(response.messages);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load messages");
    } finally {
      setLoading(false);
    }
  });

  createEffect(() => {
    if (messages().length > 0) {
      scrollToBottom();
    }
  });

  const handleSend = async () => {
    const content = input().trim();
    const imageUrl = pendingImage();
    if (!content && !imageUrl) return;

    setInput("");
    setPendingImage(undefined);
    setSending(true);
    setError(null);

    // Optimistic user message
    const tempMsg: CoachMessage = {
      id: `temp-${Date.now()}`,
      role: "user",
      content,
      imageUrl,
      suggestions: [],
      createdAt: new Date().toISOString(),
    };
    setMessages((prev) => [...prev, tempMsg]);

    try {
      const response = await coachApi.sendMessage(
        props.plantId,
        content,
        imageUrl,
      );
      // Replace temp message and add assistant response
      setMessages((prev) => [
        ...prev.filter((m) => m.id !== tempMsg.id),
        { ...tempMsg, id: response.message.id.replace(/.*/, tempMsg.id) },
        response.message,
      ]);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to send message");
      // Remove optimistic message on error
      setMessages((prev) => prev.filter((m) => m.id !== tempMsg.id));
    } finally {
      setSending(false);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleFileSelect = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    try {
      const dataUrl = await compressImage(file, {
        maxDimension: 1024,
        quality: 0.8,
      });
      setPendingImage(dataUrl);
    } catch {
      // Fallback: read uncompressed if compression fails
      const reader = new FileReader();
      reader.onload = () => setPendingImage(reader.result as string);
      reader.readAsDataURL(file);
    }
    target.value = "";
  };

  const openGallery = async () => {
    setShowGallery(true);
    if (galleryPhotos().length === 0) {
      setGalleryLoading(true);
      try {
        const response = await apiClient.getPlantPhotos(props.plantId, {
          limit: 50,
        });
        setGalleryPhotos(response.photos);
      } catch {
        // silently fail — user can still use camera
      } finally {
        setGalleryLoading(false);
      }
    }
  };

  const selectGalleryPhoto = async (photo: Photo) => {
    setShowGallery(false);
    // Fetch the photo and convert to data URL for the coach API
    try {
      const response = await fetch(`/api/photos/${photo.id}/file`);
      const blob = await response.blob();
      // Compress it like a new upload
      const file = new File([blob], photo.originalFilename, {
        type: photo.contentType,
      });
      const dataUrl = await compressImage(file, {
        maxDimension: 1024,
        quality: 0.8,
      });
      setPendingImage(dataUrl);
    } catch {
      // Fallback: use thumbnail URL directly if fetch fails
      setPendingImage(`/api/photos/${photo.id}/file`);
    }
  };

  const handleAcceptSuggestion = async (suggestion: CoachSuggestion) => {
    try {
      const updated = await coachApi.acceptSuggestion(suggestion.id);
      setMessages((prev) =>
        prev.map((m) => ({
          ...m,
          suggestions: m.suggestions.map((s) =>
            s.id === suggestion.id ? updated : s,
          ),
        })),
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to accept suggestion");
    }
  };

  const handleDismissSuggestion = async (suggestion: CoachSuggestion) => {
    try {
      const updated = await coachApi.dismissSuggestion(suggestion.id);
      setMessages((prev) =>
        prev.map((m) => ({
          ...m,
          suggestions: m.suggestions.map((s) =>
            s.id === suggestion.id ? updated : s,
          ),
        })),
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to dismiss suggestion");
    }
  };

  const suggestionLabel = (type: CoachSuggestion["suggestionType"]) => {
    switch (type) {
      case "schedule_change":
        return "Schedule Change";
      case "new_task":
        return "New Task";
      case "care_action":
        return "Care Action";
      case "photo_request":
        return "Photo Request";
    }
  };

  return (
    <div class="flex flex-col h-full">
      {/* Messages area */}
      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        <Show when={loading()}>
          <div class="flex justify-center py-8">
            <LoadingSpinner size="lg" />
          </div>
        </Show>

        <Show when={!loading() && messages().length === 0}>
          <div class="text-center text-gray-500 dark:text-gray-400 py-8">
            <div class="text-4xl mb-3">✨</div>
            <p class="font-medium">Ask your plant coach anything</p>
            <p class="text-sm mt-1">
              Get personalized care advice for {props.plantName}
            </p>
          </div>
        </Show>

        <For each={messages()}>
          {(message) => (
            <div
              class={`flex ${message.role === "user" ? "justify-end" : "justify-start"}`}
            >
              <div
                class={`max-w-[80%] rounded-2xl px-4 py-2.5 ${
                  message.role === "user"
                    ? "bg-primary-600 text-white rounded-br-md"
                    : "bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-gray-100 rounded-bl-md"
                }`}
              >
                <Show when={message.imageUrl}>
                  <img
                    src={message.imageUrl}
                    alt="Attached photo"
                    class="rounded-lg mb-2 max-h-48 object-cover"
                  />
                </Show>
                <p
                  class="whitespace-pre-wrap text-sm"
                  innerHTML={
                    message.role === "assistant"
                      ? renderMarkdown(message.content)
                      : undefined
                  }
                >
                  {message.role === "user" ? message.content : undefined}
                </p>

                {/* Suggestions */}
                <Show when={message.suggestions.length > 0}>
                  <div class="mt-3 space-y-2">
                    <For each={message.suggestions}>
                      {(suggestion) => (
                        <div class="bg-white dark:bg-gray-700 rounded-lg p-3 border border-gray-200 dark:border-gray-600">
                          <div class="flex items-center gap-2 mb-1">
                            <span class="text-xs font-medium text-primary-600 dark:text-primary-400 uppercase">
                              {suggestionLabel(suggestion.suggestionType)}
                            </span>
                            <Show when={suggestion.status !== "pending"}>
                              <span
                                class={`text-xs px-1.5 py-0.5 rounded ${
                                  suggestion.status === "accepted"
                                    ? "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300"
                                    : "bg-gray-100 text-gray-500 dark:bg-gray-600 dark:text-gray-400"
                                }`}
                              >
                                {suggestion.status}
                              </span>
                            </Show>
                          </div>
                          <Show when={suggestion.description}>
                            <p class="text-sm text-gray-700 dark:text-gray-300 mb-2">
                              {suggestion.description}
                            </p>
                          </Show>
                          <Show when={suggestion.status === "pending"}>
                            <div class="flex gap-2 mt-2">
                              <Button
                                size="sm"
                                variant="primary"
                                onClick={() =>
                                  handleAcceptSuggestion(suggestion)
                                }
                              >
                                Accept
                              </Button>
                              <Button
                                size="sm"
                                variant="outline"
                                onClick={() =>
                                  handleDismissSuggestion(suggestion)
                                }
                              >
                                Dismiss
                              </Button>
                            </div>
                          </Show>
                        </div>
                      )}
                    </For>
                  </div>
                </Show>
              </div>
            </div>
          )}
        </For>

        {/* Typing indicator */}
        <Show when={sending()}>
          <div class="flex justify-start">
            <div class="bg-gray-100 dark:bg-gray-800 rounded-2xl rounded-bl-md px-4 py-3">
              <div class="flex gap-1">
                <div
                  class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"
                  style="animation-delay: 0ms"
                />
                <div
                  class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"
                  style="animation-delay: 150ms"
                />
                <div
                  class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"
                  style="animation-delay: 300ms"
                />
              </div>
            </div>
          </div>
        </Show>

        <div ref={messagesEndRef} />
      </div>

      {/* Error */}
      <Show when={error()}>
        <div class="px-4 py-2 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 text-sm">
          {error()}
        </div>
      </Show>

      {/* Pending image preview */}
      <Show when={pendingImage()}>
        <div class="px-4 py-2 border-t border-gray-200 dark:border-gray-700">
          <div class="relative inline-block">
            <img
              src={pendingImage()}
              alt="To upload"
              class="h-16 rounded-lg object-cover"
            />
            <button
              onClick={() => setPendingImage(undefined)}
              class="absolute -top-1.5 -right-1.5 w-5 h-5 bg-red-500 text-white rounded-full text-xs flex items-center justify-center"
            >
              &times;
            </button>
          </div>
        </div>
      </Show>

      {/* Gallery picker overlay */}
      <Show when={showGallery()}>
        <div class="border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 px-4 py-3 max-h-48 overflow-y-auto">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
              Select from gallery
            </span>
            <button
              onClick={() => setShowGallery(false)}
              class="text-xs text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
            >
              Cancel
            </button>
          </div>
          <Show when={galleryLoading()}>
            <div class="flex justify-center py-4">
              <LoadingSpinner size="sm" />
            </div>
          </Show>
          <Show when={!galleryLoading() && galleryPhotos().length === 0}>
            <p class="text-sm text-gray-400 dark:text-gray-500 text-center py-4">
              No photos yet
            </p>
          </Show>
          <div class="grid grid-cols-5 gap-1.5">
            <For each={galleryPhotos()}>
              {(photo) => (
                <button
                  onClick={() => selectGalleryPhoto(photo)}
                  class="aspect-square rounded-lg overflow-hidden border-2 border-transparent hover:border-primary-500 transition-colors focus:outline-none focus:border-primary-500"
                >
                  <img
                    src={`/api/photos/${photo.id}/thumbnail`}
                    alt={photo.originalFilename}
                    class="w-full h-full object-cover"
                    loading="lazy"
                  />
                </button>
              )}
            </For>
          </div>
        </div>
      </Show>

      {/* Input area */}
      <div class="border-t border-gray-200 dark:border-gray-700 p-3 flex items-end gap-2">
        <input
          ref={fileInputRef}
          type="file"
          accept="image/*"
          class="hidden"
          onChange={handleFileSelect}
        />
        <button
          onClick={() => fileInputRef?.click()}
          class="flex-shrink-0 p-2 text-gray-500 hover:text-primary-600 dark:text-gray-400 dark:hover:text-primary-400 transition-colors"
          title="Take or upload photo"
        >
          <svg
            class="w-5 h-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z"
            />
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M15 13a3 3 0 11-6 0 3 3 0 016 0z"
            />
          </svg>
        </button>
        <button
          onClick={openGallery}
          class="flex-shrink-0 p-2 text-gray-500 hover:text-primary-600 dark:text-gray-400 dark:hover:text-primary-400 transition-colors"
          title="Choose from gallery"
        >
          <svg
            class="w-5 h-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
            />
          </svg>
        </button>
        <textarea
          value={input()}
          onInput={(e) => setInput(e.currentTarget.value)}
          onKeyDown={handleKeyDown}
          placeholder={`Ask about ${props.plantName}...`}
          rows={1}
          class="flex-1 resize-none rounded-xl border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 dark:text-gray-100 placeholder:text-gray-400"
        />
        <Button
          size="sm"
          variant="primary"
          onClick={handleSend}
          disabled={sending() || (!input().trim() && !pendingImage())}
          class="flex-shrink-0"
        >
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
              d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
            />
          </svg>
        </Button>
      </div>
    </div>
  );
};
