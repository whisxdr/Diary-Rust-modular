/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Absolute URL of the API when the frontend is not served from the API's origin. */
  readonly VITE_API_BASE_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}