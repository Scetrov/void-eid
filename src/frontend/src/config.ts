// Central configuration for API endpoints
// Check for runtime config injected by the container, then build-time config, then default
declare global {
  interface Window {
    ENV?: {
      VITE_API_URL?: string;
      VITE_BLOCK_EXPLORER_URL?: string;
      VITE_MUMBLE_SERVER_URL?: string;
      VITE_SUI_NETWORK?: string;
    };
  }
}

export const API_URL = window.ENV?.VITE_API_URL || import.meta.env.VITE_API_URL || 'http://localhost:5038';
export const BLOCK_EXPLORER_URL = window.ENV?.VITE_BLOCK_EXPLORER_URL || import.meta.env.VITE_BLOCK_EXPLORER_URL || 'https://suiscan.xyz/mainnet';
export const MUMBLE_SERVER_URL = window.ENV?.VITE_MUMBLE_SERVER_URL || import.meta.env.VITE_MUMBLE_SERVER_URL || 'mumble.void.scetrov.live';
export const SUI_NETWORK = window.ENV?.VITE_SUI_NETWORK || import.meta.env.VITE_SUI_NETWORK || 'testnet';
