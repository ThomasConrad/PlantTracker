/**
 * Offline Queue — stores failed mutations in IndexedDB and replays them on reconnect.
 *
 * Design:
 * - Only queues mutating requests (POST, PUT, PATCH, DELETE)
 * - GET requests are never queued (they use cache or fail gracefully)
 * - Replays in FIFO order when connectivity returns
 * - Emits events for UI to show pending sync count
 * - Handles conflicts: if replay gets 404/409, discards the entry (stale)
 */

import { openDB, type DBSchema, type IDBPDatabase } from "idb";

interface QueuedRequest {
  id: number;
  method: string;
  url: string;
  body: string | null;
  headers: Record<string, string>;
  timestamp: number;
  retries: number;
}

interface OfflineQueueDB extends DBSchema {
  pendingRequests: {
    key: number;
    value: QueuedRequest;
    indexes: { "by-timestamp": number };
  };
  cachedResponses: {
    key: string; // URL (for GET requests)
    value: {
      url: string;
      body: string;
      status: number;
      timestamp: number;
    };
  };
}

const DB_NAME = "planty-offline";
const DB_VERSION = 1;

let dbPromise: Promise<IDBPDatabase<OfflineQueueDB>> | null = null;

function getDB(): Promise<IDBPDatabase<OfflineQueueDB>> {
  if (!dbPromise) {
    dbPromise = openDB<OfflineQueueDB>(DB_NAME, DB_VERSION, {
      upgrade(db) {
        const store = db.createObjectStore("pendingRequests", {
          keyPath: "id",
          autoIncrement: true,
        });
        store.createIndex("by-timestamp", "timestamp");

        db.createObjectStore("cachedResponses", { keyPath: "url" });
      },
    });
  }
  return dbPromise;
}

// ─── Queue Management ────────────────────────────────────────────────────────

export async function enqueueRequest(
  method: string,
  url: string,
  body: string | null,
  headers: Record<string, string>,
): Promise<void> {
  const db = await getDB();
  await db.add("pendingRequests", {
    id: undefined as unknown as number, // auto-increment
    method,
    url,
    body,
    headers,
    timestamp: Date.now(),
    retries: 0,
  });
  notifyListeners();
}

export async function getPendingCount(): Promise<number> {
  const db = await getDB();
  return db.count("pendingRequests");
}

export async function getPendingRequests(): Promise<QueuedRequest[]> {
  const db = await getDB();
  return db.getAllFromIndex("pendingRequests", "by-timestamp");
}

export async function removeRequest(id: number): Promise<void> {
  const db = await getDB();
  await db.delete("pendingRequests", id);
  notifyListeners();
}

export async function clearAllPending(): Promise<void> {
  const db = await getDB();
  await db.clear("pendingRequests");
  notifyListeners();
}

// ─── Response Cache (for offline reads) ──────────────────────────────────────

export async function cacheResponse(
  url: string,
  body: string,
  status: number,
): Promise<void> {
  const db = await getDB();
  await db.put("cachedResponses", { url, body, status, timestamp: Date.now() });
}

export async function getCachedResponse(
  url: string,
): Promise<{ body: string; status: number } | undefined> {
  const db = await getDB();
  const cached = await db.get("cachedResponses", url);
  if (!cached) return undefined;
  // Expire after 24 hours
  if (Date.now() - cached.timestamp > 24 * 60 * 60 * 1000) {
    await db.delete("cachedResponses", url);
    return undefined;
  }
  return { body: cached.body, status: cached.status };
}

// ─── Sync / Replay ──────────────────────────────────────────────────────────

let syncing = false;

export async function replayQueue(): Promise<{
  replayed: number;
  failed: number;
}> {
  if (syncing) return { replayed: 0, failed: 0 };
  syncing = true;

  let replayed = 0;
  let failed = 0;

  try {
    const pending = await getPendingRequests();

    for (const req of pending) {
      try {
        const response = await fetch(req.url, {
          method: req.method,
          headers: req.headers,
          body: req.body,
          credentials: "include",
        });

        if (response.ok || response.status === 201) {
          // Success — remove from queue
          await removeRequest(req.id);
          replayed++;
        } else if (
          response.status === 404 ||
          response.status === 409 ||
          response.status === 422
        ) {
          // Stale/conflicting request — discard
          await removeRequest(req.id);
          failed++;
        } else if (response.status >= 500) {
          // Server error — leave in queue for retry
          failed++;
          break; // Stop replaying on server errors
        } else {
          // Other client errors (400, 401, 403) — discard
          await removeRequest(req.id);
          failed++;
        }
      } catch {
        // Network error — still offline, stop replaying
        failed++;
        break;
      }
    }
  } finally {
    syncing = false;
    notifyListeners();
  }

  return { replayed, failed };
}

// ─── Online/Offline Detection & Auto-Sync ────────────────────────────────────

let initialized = false;

export function initOfflineSync(): void {
  if (initialized) return;
  initialized = true;

  window.addEventListener("online", () => {
    // Small delay to let connection stabilize
    setTimeout(() => {
      replayQueue();
    }, 1000);
  });

  // Also try to sync periodically when online (handles cases where
  // the online event fires before the connection is truly ready)
  setInterval(async () => {
    if (navigator.onLine) {
      const count = await getPendingCount();
      if (count > 0) {
        replayQueue();
      }
    }
  }, 30000);
}

// ─── Listeners (for UI reactivity) ──────────────────────────────────────────

type QueueListener = (count: number) => void;
const listeners: Set<QueueListener> = new Set();

export function onQueueChange(listener: QueueListener): () => void {
  listeners.add(listener);
  // Immediately notify with current count
  getPendingCount().then((count) => listener(count));
  return () => listeners.delete(listener);
}

async function notifyListeners(): Promise<void> {
  const count = await getPendingCount();
  listeners.forEach((fn) => fn(count));
}

// ─── Helper: check if we're offline ─────────────────────────────────────────

export function isOffline(): boolean {
  return !navigator.onLine;
}
