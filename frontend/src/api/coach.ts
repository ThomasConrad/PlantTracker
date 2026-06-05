import { apiClient } from "./client";
import { logApiError, logApiSuccess, logWarn } from "@/utils/logger";

export interface CoachMessage {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  imageUrl?: string;
  suggestions: CoachSuggestion[];
  createdAt: string;
}

export interface CoachSuggestion {
  id: string;
  suggestionType:
    | "schedule_change"
    | "new_task"
    | "care_action"
    | "photo_request"
    | "species_correction";
  description: string;
  payload: Record<string, unknown>;
  status: "pending" | "accepted" | "dismissed";
  appliedAt?: string;
}

export interface CoachMessagesResponse {
  conversationId: string;
  messages: CoachMessage[];
}

export interface CoachMessageResponse {
  message: CoachMessage;
}

export interface StreamCallbacks {
  onToken: (text: string) => void;
  onDone: (message: CoachMessage) => void;
  onError: (error: string) => void;
}

export const coachApi = {
  getMessages: async (plantId: string): Promise<CoachMessagesResponse> => {
    return apiClient.request<CoachMessagesResponse>(
      `/coach/plants/${plantId}/messages`,
    );
  },

  sendMessage: async (
    plantId: string,
    content: string,
    imageUrl?: string,
  ): Promise<CoachMessageResponse> => {
    return apiClient.request<CoachMessageResponse>(
      `/coach/plants/${plantId}/messages`,
      {
        method: "POST",
        body: JSON.stringify({ content, imageUrl }),
      },
    );
  },

  /**
   * Stream a message response via SSE.
   * Returns an AbortController to cancel the stream.
   */
  streamMessage: (
    plantId: string,
    content: string,
    imageUrl: string | undefined,
    callbacks: StreamCallbacks,
  ): AbortController => {
    const controller = new AbortController();
    const baseUrl = apiClient.getBaseUrl();
    const url = `${baseUrl}/coach/plants/${plantId}/messages/stream`;
    const startTime = performance.now();

    fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify({ content, imageUrl }),
      signal: controller.signal,
    })
      .then(async (response) => {
        const durationMs = Math.round(performance.now() - startTime);

        if (!response.ok) {
          const text = await response.text();
          logApiError({
            method: "POST",
            url,
            status: response.status,
            durationMs,
            requestBody: { content, imageUrl },
            responseBody: text,
          });
          callbacks.onError(text || `HTTP ${response.status}`);
          return;
        }

        const reader = response.body?.getReader();
        if (!reader) {
          logWarn("Coach stream: no response body", { url });
          callbacks.onError("No response body");
          return;
        }

        const decoder = new TextDecoder();
        let buffer = "";

        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split("\n");
          buffer = lines.pop() || "";

          // SSE parser: accumulate event type and multi-line data,
          // dispatch when we hit a blank line (event boundary).
          let currentEvent = "";
          let dataLines: string[] = [];

          for (const line of lines) {
            if (line.startsWith("event: ")) {
              currentEvent = line.slice(7);
            } else if (line.startsWith("data: ")) {
              dataLines.push(line.slice(6));
            } else if (line.trim() === "") {
              // Blank line = end of event, dispatch
              if (currentEvent && dataLines.length > 0) {
                const data = dataLines.join("\n");
                switch (currentEvent) {
                  case "token":
                    callbacks.onToken(data);
                    break;
                  case "done": {
                    const streamDuration = Math.round(
                      performance.now() - startTime,
                    );
                    try {
                      const message = JSON.parse(data) as CoachMessage;
                      logApiSuccess({
                        method: "POST",
                        url,
                        status: response.status,
                        durationMs: streamDuration,
                      });
                      callbacks.onDone(message);
                    } catch (e) {
                      logApiError({
                        method: "POST",
                        url,
                        status: response.status,
                        durationMs: streamDuration,
                        responseBody: data,
                        error: e,
                        extra: { event: "done", parseError: true },
                      });
                      callbacks.onError("Failed to parse done event");
                    }
                    break;
                  }
                  case "error":
                    logApiError({
                      method: "POST",
                      url,
                      status: response.status,
                      durationMs: Math.round(performance.now() - startTime),
                      responseBody: data,
                      extra: { event: "error", streamError: true },
                    });
                    callbacks.onError(data);
                    break;
                }
              }
              // Reset for next event
              currentEvent = "";
              dataLines = [];
            }
          }
        }
      })
      .catch((err) => {
        if (err.name !== "AbortError") {
          logApiError({
            method: "POST",
            url,
            durationMs: Math.round(performance.now() - startTime),
            requestBody: { content, imageUrl },
            error: err,
            extra: { networkError: true },
          });
          callbacks.onError(err.message || "Stream failed");
        }
      });

    return controller;
  },

  acceptSuggestion: async (suggestionId: string): Promise<CoachSuggestion> => {
    return apiClient.request<CoachSuggestion>(
      `/coach/suggestions/${suggestionId}/accept`,
      {
        method: "POST",
      },
    );
  },

  dismissSuggestion: async (suggestionId: string): Promise<CoachSuggestion> => {
    return apiClient.request<CoachSuggestion>(
      `/coach/suggestions/${suggestionId}/dismiss`,
      {
        method: "POST",
      },
    );
  },
};
