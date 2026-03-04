/// <reference types="vite/client" />
/// <reference types="vite-plugin-pwa/client" />

interface ImportMetaEnv {
  readonly VITE_WAITLIST_ENABLED?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
