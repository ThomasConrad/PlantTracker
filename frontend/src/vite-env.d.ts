/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_WAITLIST_ENABLED?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
