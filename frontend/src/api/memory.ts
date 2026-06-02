import { apiClient } from "./client";

export interface PlantMemory {
  id: string;
  plantId: string;
  factType: string;
  content: string;
  confidence: number;
  source: "coach" | "user";
  sourceMessageId?: string;
  createdAt: string;
  updatedAt: string;
}

export interface PlantMemoriesResponse {
  memories: PlantMemory[];
}

export interface CreateMemoryRequest {
  factType: string;
  content: string;
}

export interface UpdateMemoryRequest {
  content?: string;
  factType?: string;
}

export async function getPlantMemories(
  plantId: string,
): Promise<PlantMemoriesResponse> {
  return apiClient.request<PlantMemoriesResponse>(
    `/coach/plants/${plantId}/memories`,
  );
}

export async function createPlantMemory(
  plantId: string,
  request: CreateMemoryRequest,
): Promise<PlantMemory> {
  return apiClient.request<PlantMemory>(`/coach/plants/${plantId}/memories`, {
    method: "POST",
    body: JSON.stringify(request),
  });
}

export async function updatePlantMemory(
  plantId: string,
  memoryId: string,
  request: UpdateMemoryRequest,
): Promise<PlantMemory> {
  return apiClient.request<PlantMemory>(
    `/coach/plants/${plantId}/memories/${memoryId}`,
    {
      method: "PUT",
      body: JSON.stringify(request),
    },
  );
}

export async function deletePlantMemory(
  plantId: string,
  memoryId: string,
): Promise<void> {
  await apiClient.request<void>(
    `/coach/plants/${plantId}/memories/${memoryId}`,
    { method: "DELETE" },
  );
}

// Human-readable labels for fact types
export const FACT_TYPE_LABELS: Record<string, { label: string; icon: string }> =
  {
    location: { label: "Location", icon: "📍" },
    light: { label: "Light", icon: "☀️" },
    soil: { label: "Soil", icon: "🪴" },
    pot: { label: "Pot", icon: "🏺" },
    watering_preference: { label: "Watering", icon: "💧" },
    temperature: { label: "Temperature", icon: "🌡️" },
    humidity: { label: "Humidity", icon: "💨" },
    growth_habit: { label: "Growth", icon: "🌱" },
    symptom_pattern: { label: "Symptoms", icon: "🔍" },
    pest_history: { label: "Pests", icon: "🐛" },
    fertilizer_preference: { label: "Fertilizer", icon: "🧪" },
    propagation: { label: "Propagation", icon: "✂️" },
    acquisition: { label: "Origin", icon: "🏷️" },
    species_note: { label: "Species", icon: "🧬" },
    general: { label: "Note", icon: "📝" },
  };

export const FACT_TYPES = Object.keys(FACT_TYPE_LABELS);
