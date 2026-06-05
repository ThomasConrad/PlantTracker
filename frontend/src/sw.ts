/// <reference lib="webworker" />
import { precacheAndRoute, cleanupOutdatedCaches } from "workbox-precaching";
import { clientsClaim } from "workbox-core";
import { registerRoute } from "workbox-routing";
import { NetworkFirst, CacheFirst } from "workbox-strategies";
import { ExpirationPlugin } from "workbox-expiration";

declare let self: ServiceWorkerGlobalScope;

// ─── Workbox Precaching ─────────────────────────────────────────────────────
// The __WB_MANIFEST placeholder is replaced by vite-plugin-pwa with the
// precache manifest at build time.
precacheAndRoute(self.__WB_MANIFEST);
cleanupOutdatedCaches();

// Take control immediately
self.skipWaiting();
clientsClaim();

// ─── Runtime Caching ────────────────────────────────────────────────────────

// Cache API GET requests with network-first strategy
registerRoute(
  ({ url }) => url.pathname.startsWith("/api/v1/"),
  new NetworkFirst({
    cacheName: "api-cache",
    plugins: [
      new ExpirationPlugin({
        maxEntries: 200,
        maxAgeSeconds: 60 * 60 * 24, // 24 hours
      }),
    ],
    networkTimeoutSeconds: 5,
  })
);

// Cache plant photos with cache-first (they don't change)
registerRoute(
  ({ url }) =>
    /\/api\/photos\/.*\/(file|thumbnail)/.test(url.pathname),
  new CacheFirst({
    cacheName: "photo-cache",
    plugins: [
      new ExpirationPlugin({
        maxEntries: 500,
        maxAgeSeconds: 60 * 60 * 24 * 30, // 30 days
      }),
    ],
  })
);

// ─── Push Notification Handlers ─────────────────────────────────────────────

interface PushPayload {
  title: string;
  body: string;
  url?: string;
  icon?: string;
  tag?: string;
}

self.addEventListener("push", (event: PushEvent) => {
  if (!event.data) return;

  let payload: PushPayload;
  try {
    payload = event.data.json() as PushPayload;
  } catch {
    // If not valid JSON, use text as body
    payload = {
      title: "Planty",
      body: event.data.text(),
    };
  }

  const options: NotificationOptions = {
    body: payload.body,
    icon: payload.icon || "/pwa-192x192.png",
    badge: "/pwa-192x192.png",
    tag: payload.tag || "planty-notification",
    data: {
      url: payload.url || "/",
    },
    // Vibrate pattern: short-short-long
    vibrate: [100, 50, 200],
    // Require interaction for urgent notifications
    requireInteraction: payload.tag?.includes("urgent") ?? false,
  };

  event.waitUntil(self.registration.showNotification(payload.title, options));
});

self.addEventListener("notificationclick", (event: NotificationEvent) => {
  event.notification.close();

  const url = (event.notification.data?.url as string) || "/";

  event.waitUntil(
    self.clients.matchAll({ type: "window", includeUncontrolled: true }).then((windowClients) => {
      // If a window is already open, focus it and navigate
      for (const client of windowClients) {
        if ("focus" in client) {
          client.focus();
          client.navigate(url);
          return;
        }
      }
      // Otherwise open a new window
      return self.clients.openWindow(url);
    })
  );
});

// Handle notification close (for analytics if needed)
self.addEventListener("notificationclose", (_event: NotificationEvent) => {
  // Could send analytics here in the future
});
