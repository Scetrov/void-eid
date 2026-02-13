/**
 * Utility functions shared across the frontend
 */

import { BLOCK_EXPLORER_URL } from './config';

/**
 * Truncate a wallet address to format: 0x1234...5678
 */
export function formatAddress(address: string): string {
    if (!address) return '';
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

/**
 * Format a date string to relative time (e.g., "2d ago")
 */
export function formatTimeAgo(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (diffInSeconds < 60) return `${diffInSeconds}s ago`;
    const diffInMinutes = Math.floor(diffInSeconds / 60);
    if (diffInMinutes < 60) return `${diffInMinutes}m ago`;
    const diffInHours = Math.floor(diffInMinutes / 60);
    if (diffInHours < 24) return `${diffInHours}h ago`;
    const diffInDays = Math.floor(diffInHours / 24);
    return `${diffInDays}d ago`;
}

/**
 * Format a date to the login display format: YYYY.MM.DD at HH:MM UTC
 */
export function formatLoginDate(dateString: string | null | undefined): string {
    if (!dateString) return 'Never';

    const date = new Date(dateString);
    const year = date.getUTCFullYear();
    const month = String(date.getUTCMonth() + 1).padStart(2, '0');
    const day = String(date.getUTCDate()).padStart(2, '0');
    const hours = String(date.getUTCHours()).padStart(2, '0');
    const minutes = String(date.getUTCMinutes()).padStart(2, '0');

    return `${year}.${month}.${day} at ${hours}:${minutes} UTC`;
}

/**
 * Get Explorer URL for a given network and address/object
 */
export function getExplorerUrl(network: string, value: string, type: 'address' | 'object' | 'tx' = 'address'): string {
    // defaults to https://suiscan.xyz/mainnet in config if not set
    let baseUrl = BLOCK_EXPLORER_URL;

    // Attempt to strip path from baseUrl to get the root for dynamic network appending
    // e.g. https://suiscan.xyz/mainnet -> https://suiscan.xyz
    // or https://suiscan.xyz -> https://suiscan.xyz
    try {
        const url = new URL(baseUrl);
        // If the path contains 'mainnet', 'testnet', or 'devnet', strip it
        if (url.pathname.includes('mainnet') || url.pathname.includes('testnet') || url.pathname.includes('devnet')) {
            baseUrl = url.origin;
        }
    } catch {
        // invalid url, ignore
    }

    const networkPath = (network || 'mainnet').toLowerCase();

    let pathType = '';
    switch (type) {
        case 'address': pathType = 'account'; break;
        case 'object': pathType = 'object'; break;
        case 'tx': pathType = 'tx'; break;
    }

    // specific handler for suiscan style
    if (baseUrl.includes('suiscan.xyz')) {
         return `${baseUrl}/${networkPath}/${pathType}/${value}`;
    }

    // Generic fallback: just append /network/type/value ??
    // Or if unknown explorer, maybe it doesn't support multi-network this way.
    // For now, assume the user configured a suiscan-compatible URL or we just use it as is if it doesn't match known patterns.
    return `${baseUrl}/${networkPath}/${pathType}/${value}`;
}

/**
 * Get a display label or icon for the network
 */
export function getNetworkLabel(network: string): { label: string, color: string, bgColor: string } {
    const net = (network || 'mainnet').toLowerCase();
    switch (net) {
        case 'mainnet': return { label: 'Mainnet', color: '#22c55e', bgColor: 'rgba(34, 197, 94, 0.1)' };
        case 'testnet': return { label: 'Testnet', color: '#f59e0b', bgColor: 'rgba(245, 158, 11, 0.1)' };
        case 'devnet': return { label: 'Devnet', color: '#ef4444', bgColor: 'rgba(239, 68, 68, 0.1)' };
        default: return { label: network, color: '#9ca3af', bgColor: 'rgba(156, 163, 175, 0.1)' };
    }
}
