/**
 * Push Notifications Store
 *
 * Manages Web Push subscription lifecycle:
 * - Fetches VAPID public key from backend
 * - Subscribes/unsubscribes the browser's PushManager
 * - Persists subscription to backend for server-side delivery
 * - Exposes reactive state for UI bindings
 */

import { createSignal } from "solid-js";
import { apiClient } from "@/api/client";

export type PushState =
  | "unsupported" // Browser doesn't support Push API
  | "denied" // User denied permission
  | "prompt" // User hasn't decided yet
  | "subscribed" // Active subscription
  | "unsubscribed"; // Permission granted but not subscribed

const [pushState, setPushState] = createSignal<PushState>("unsupported");
const [loading, setLoading] = createSignal(false);
const [error, setError] = createSignal<string | null>(null);

function isPushSupported(): boolean {
  return (
    typeof window !== "undefined" &&
    "serviceWorker" in navigator &&
    "PushManager" in window &&
    "Notification" in window
  );
}

async function getRegistration(): Promise<ServiceWorkerRegistration | null> {
  if (!("serviceWorker" in navigator)) return null;
  return navigator.serviceWorker.ready;
}

/**
 * Initialize push state by checking current browser status
 */
async function initializePushState(): Promise<void> {
  if (!isPushSupported()) {
    setPushState("unsupported");
    return;
  }

  const permission = Notification.permission;
  if (permission === "denied") {
    setPushState("denied");
    return;
  }

  // Check if already subscribed
  const registration = await getRegistration();
  if (!registration) {
    setPushState("prompt");
    return;
  }

  const subscription = await registration.pushManager.getSubscription();
  if (subscription) {
    setPushState("subscribed");
  } else if (permission === "granted") {
    setPushState("unsubscribed");
  } else {
    setPushState("prompt");
  }
}

/**
 * Subscribe to push notifications:
 * 1. Request notification permission
 * 2. Get VAPID public key from server
 * 3. Subscribe via PushManager
 * 4. Send subscription to backend
 */
async function subscribe(): Promise<boolean> {
  if (!isPushSupported()) return false;

  setLoading(true);
  setError(null);

  try {
    // Request permission
    const permission = await Notification.requestPermission();
    if (permission === "denied") {
      setPushState("denied");
      return false;
    }
    if (permission !== "granted") {
      setPushState("prompt");
      return false;
    }

    // Get VAPID key from server
    const { publicKey } = await apiClient.getVapidPublicKey();
    const applicationServerKey = urlBase64ToUint8Array(publicKey);

    // Subscribe via PushManager
    const registration = await getRegistration();
    if (!registration) {
      setError("Service worker not ready");
      return false;
    }

    const subscription = await registration.pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey,
    });

    // Send subscription to backend
    await apiClient.subscribePush(subscription);

    setPushState("subscribed");
    return true;
  } catch (err) {
    const message =
      err instanceof Error ? err.message : "Failed to subscribe to push notifications";
    setError(message);
    console.error("Push subscribe error:", err);
    return false;
  } finally {
    setLoading(false);
  }
}

/**
 * Unsubscribe from push notifications:
 * 1. Unsubscribe from PushManager
 * 2. Remove subscription from backend
 */
async function unsubscribe(): Promise<boolean> {
  setLoading(true);
  setError(null);

  try {
    const registration = await getRegistration();
    if (!registration) return false;

    const subscription = await registration.pushManager.getSubscription();
    if (subscription) {
      // Remove from backend first
      await apiClient.unsubscribePush(subscription.endpoint);
      // Then unsubscribe locally
      await subscription.unsubscribe();
    }

    setPushState("unsubscribed");
    return true;
  } catch (err) {
    const message =
      err instanceof Error ? err.message : "Failed to unsubscribe from push notifications";
    setError(message);
    console.error("Push unsubscribe error:", err);
    return false;
  } finally {
    setLoading(false);
  }
}

/**
 * Convert a URL-safe base64 string to Uint8Array
 * (needed for applicationServerKey)
 */
function urlBase64ToUint8Array(base64String: string): Uint8Array {
  const padding = "=".repeat((4 - (base64String.length % 4)) % 4);
  const base64 = (base64String + padding).replace(/-/g, "+").replace(/_/g, "/");
  const rawData = atob(base64);
  const outputArray = new Uint8Array(rawData.length);
  for (let i = 0; i < rawData.length; i++) {
    outputArray[i] = rawData.charCodeAt(i);
  }
  return outputArray;
}

export const pushStore = {
  get state() {
    return pushState();
  },
  get loading() {
    return loading();
  },
  get error() {
    return error();
  },
  get isSubscribed() {
    return pushState() === "subscribed";
  },
  get canSubscribe() {
    const s = pushState();
    return s === "prompt" || s === "unsubscribed";
  },

  initialize: initializePushState,
  subscribe,
  unsubscribe,
};
