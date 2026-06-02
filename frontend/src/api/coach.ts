import { apiClient } from './client';

export interface CoachMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  imageUrl?: string;
  suggestions: CoachSuggestion[];
  createdAt: string;
}

export interface CoachSuggestion {
  id: string;
  suggestionType: 'schedule_change' | 'new_task' | 'care_action' | 'photo_request';
  description: string;
  payload: any;
  status: 'pending' | 'accepted' | 'dismissed';
  appliedAt?: string;
}

export interface CoachMessagesResponse {
  conversationId: string;
  messages: CoachMessage[];
}

export interface CoachMessageResponse {
  message: CoachMessage;
}

export const coachApi = {
  getMessages: async (plantId: string): Promise<CoachMessagesResponse> => {
    return apiClient.request<CoachMessagesResponse>(`/coach/plants/${plantId}/messages`);
  },

  sendMessage: async (plantId: string, content: string, imageUrl?: string): Promise<CoachMessageResponse> => {
    return apiClient.request<CoachMessageResponse>(`/coach/plants/${plantId}/messages`, {
      method: 'POST',
      body: JSON.stringify({ content, image_url: imageUrl }),
    });
  },

  acceptSuggestion: async (suggestionId: string): Promise<CoachSuggestion> => {
    return apiClient.request<CoachSuggestion>(`/coach/suggestions/${suggestionId}/accept`, {
      method: 'POST',
    });
  },

  dismissSuggestion: async (suggestionId: string): Promise<CoachSuggestion> => {
    return apiClient.request<CoachSuggestion>(`/coach/suggestions/${suggestionId}/dismiss`, {
      method: 'POST',
    });
  },
};
