import { apiClient } from "./client";

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
        body: JSON.stringify({ content, image_url: imageUrl }),
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

    fetch(`${baseUrl}/coach/plants/${plantId}/messages/stream`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify({ content, image_url: imageUrl }),
      signal: controller.signal,
    })
      .then(async (response) => {
        if (!response.ok) {
          const text = await response.text();
          callbacks.onError(text || `HTTP ${response.status}`);
          return;
        }

        const reader = response.body?.getReader();
        if (!reader) {
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

          let currentEvent = "";
          for (const line of lines) {
            if (line.startsWith("event: ")) {
              currentEvent = line.slice(7);
            } else if (line.startsWith("data: ")) {
              const data = line.slice(6);
              switch (currentEvent) {
                case "token":
                  callbacks.onToken(data);
                  break;
                case "done":
                  try {
                    const message = JSON.parse(data) as CoachMessage;
                    callbacks.onDone(message);
                  } catch {
                    callbacks.onError("Failed to parse done event");
                  }
                  break;
                case "error":
                  callbacks.onError(data);
                  break;
              }
              currentEvent = "";
            }
          }
        }
      })
      .catch((err) => {
        if (err.name !== "AbortError") {
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
