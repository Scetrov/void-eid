// Central configuration for API endpoints
// Check for runtime config injected by the container, then build-time config, then default
declare global {
  interface Window {
    ENV?: {
      VITE_API_URL?: string;
    };
  }
}

export const API_URL = window.ENV?.VITE_API_URL || import.meta.env.VITE_API_URL || 'http://localhost:5038';
