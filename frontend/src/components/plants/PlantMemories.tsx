import { Component, createSignal, For, Show, onMount } from "solid-js";
import {
  getPlantMemories,
  createPlantMemory,
  deletePlantMemory,
  updatePlantMemory,
  FACT_TYPE_LABELS,
  FACT_TYPES,
} from "@/api/memory";
import type { PlantMemory } from "@/api/memory";

interface Props {
  plantId: string;
}

export const PlantMemories: Component<Props> = (props) => {
  const [memories, setMemories] = createSignal<PlantMemory[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [showAddForm, setShowAddForm] = createSignal(false);
  const [editingId, setEditingId] = createSignal<string | null>(null);
  const [editContent, setEditContent] = createSignal("");
  const [newFactType, setNewFactType] = createSignal("general");
  const [newContent, setNewContent] = createSignal("");
  const [error, setError] = createSignal<string | null>(null);

  const loadMemories = async () => {
    try {
      const response = await getPlantMemories(props.plantId);
      setMemories(response.memories);
    } catch {
      // Silently fail — memories are supplementary
    } finally {
      setLoading(false);
    }
  };

  onMount(loadMemories);

  const handleAdd = async () => {
    if (!newContent().trim()) return;
    setError(null);
    try {
      const memory = await createPlantMemory(props.plantId, {
        factType: newFactType(),
        content: newContent().trim(),
      });
      setMemories((prev) => [...prev, memory]);
      setNewContent("");
      setShowAddForm(false);
    } catch {
      setError("Failed to add note");
    }
  };

  const handleDelete = async (memoryId: string) => {
    try {
      await deletePlantMemory(props.plantId, memoryId);
      setMemories((prev) => prev.filter((m) => m.id !== memoryId));
    } catch {
      setError("Failed to delete");
    }
  };

  const handleStartEdit = (memory: PlantMemory) => {
    setEditingId(memory.id);
    setEditContent(memory.content);
  };

  const handleSaveEdit = async (memoryId: string) => {
    if (!editContent().trim()) return;
    try {
      const updated = await updatePlantMemory(props.plantId, memoryId, {
        content: editContent().trim(),
      });
      setMemories((prev) => prev.map((m) => (m.id === memoryId ? updated : m)));
      setEditingId(null);
    } catch {
      setError("Failed to update");
    }
  };

  const groupedMemories = () => {
    const groups: Record<string, PlantMemory[]> = {};
    for (const mem of memories()) {
      const key = mem.factType;
      if (!groups[key]) groups[key] = [];
      groups[key].push(mem);
    }
    return groups;
  };

  return (
    <div class="rounded-xl border border-gray-200 bg-white p-4">
      <div class="mb-3 flex items-center justify-between">
        <h3 class="text-sm font-semibold text-gray-700">
          🧠 Plant Memory
          <Show when={memories().length > 0}>
            <span class="ml-1 text-xs text-gray-400">
              ({memories().length})
            </span>
          </Show>
        </h3>
        <button
          class="text-xs text-green-600 hover:text-green-700 font-medium"
          onClick={() => setShowAddForm(!showAddForm())}
        >
          {showAddForm() ? "Cancel" : "+ Add"}
        </button>
      </div>

      <Show when={error()}>
        <p class="text-xs text-red-500 mb-2">{error()}</p>
      </Show>

      {/* Add form */}
      <Show when={showAddForm()}>
        <div class="mb-3 space-y-2 rounded-lg bg-gray-50 p-3">
          <select
            class="w-full rounded border border-gray-200 px-2 py-1.5 text-sm"
            value={newFactType()}
            onChange={(e) => setNewFactType(e.currentTarget.value)}
          >
            <For each={FACT_TYPES}>
              {(type_) => (
                <option value={type_}>
                  {FACT_TYPE_LABELS[type_].icon} {FACT_TYPE_LABELS[type_].label}
                </option>
              )}
            </For>
          </select>
          <textarea
            class="w-full rounded border border-gray-200 px-2 py-1.5 text-sm resize-none"
            rows={2}
            placeholder="e.g., On south-facing kitchen windowsill"
            value={newContent()}
            onInput={(e) => setNewContent(e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                handleAdd();
              }
            }}
          />
          <button
            class="w-full rounded bg-green-600 px-3 py-1.5 text-sm text-white font-medium hover:bg-green-700 disabled:opacity-50"
            disabled={!newContent().trim()}
            onClick={handleAdd}
          >
            Add Note
          </button>
        </div>
      </Show>

      {/* Memory list */}
      <Show
        when={!loading()}
        fallback={<p class="text-xs text-gray-400">Loading...</p>}
      >
        <Show
          when={memories().length > 0}
          fallback={
            <p class="text-xs text-gray-400 italic">
              No memories yet. Chat with the AI coach or add your own notes.
            </p>
          }
        >
          <div class="space-y-1.5">
            <For each={Object.entries(groupedMemories())}>
              {([factType, mems]) => {
                const info = FACT_TYPE_LABELS[factType] || {
                  label: factType,
                  icon: "📝",
                };
                return (
                  <div>
                    <For each={mems}>
                      {(mem) => (
                        <div class="group flex items-start gap-2 rounded-md px-2 py-1.5 hover:bg-gray-50">
                          <span
                            class="text-sm flex-shrink-0"
                            title={info.label}
                          >
                            {info.icon}
                          </span>
                          <Show
                            when={editingId() !== mem.id}
                            fallback={
                              <div class="flex-1 flex gap-1">
                                <input
                                  class="flex-1 rounded border border-gray-300 px-2 py-0.5 text-sm"
                                  value={editContent()}
                                  onInput={(e) =>
                                    setEditContent(e.currentTarget.value)
                                  }
                                  onKeyDown={(e) => {
                                    if (e.key === "Enter")
                                      handleSaveEdit(mem.id);
                                    if (e.key === "Escape") setEditingId(null);
                                  }}
                                />
                                <button
                                  class="text-xs text-green-600"
                                  onClick={() => handleSaveEdit(mem.id)}
                                >
                                  ✓
                                </button>
                              </div>
                            }
                          >
                            <span
                              class="flex-1 text-sm text-gray-700 cursor-pointer"
                              onClick={() => handleStartEdit(mem)}
                              title="Click to edit"
                            >
                              {mem.content}
                              <Show when={mem.confidence < 0.8}>
                                <span class="ml-1 text-xs text-gray-400">
                                  (uncertain)
                                </span>
                              </Show>
                            </span>
                          </Show>
                          <Show when={editingId() !== mem.id}>
                            <button
                              class="opacity-0 group-hover:opacity-100 text-xs text-gray-400 hover:text-red-500 transition-opacity"
                              onClick={() => handleDelete(mem.id)}
                              title="Delete"
                            >
                              ×
                            </button>
                          </Show>
                        </div>
                      )}
                    </For>
                  </div>
                );
              }}
            </For>
          </div>
        </Show>
      </Show>

      {/* Coach attribution */}
      <Show when={memories().some((m) => m.source === "coach")}>
        <p class="mt-2 text-[10px] text-gray-400 italic">
          Some notes were automatically learned by the AI coach from your
          conversations.
        </p>
      </Show>
    </div>
  );
};
