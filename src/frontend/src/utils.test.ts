import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { formatAddress, formatTimeAgo, formatLoginDate } from './utils';

describe('formatAddress', () => {
    it('should truncate a valid address to 0x1234...5678 format', () => {
        const address = '0x1234567890abcdef1234567890abcdef12345678';
        expect(formatAddress(address)).toBe('0x1234...5678');
    });

    it('should return empty string for empty input', () => {
        expect(formatAddress('')).toBe('');
    });

    it('should handle short addresses gracefully', () => {
        const shortAddress = '0x1234';
        // With 6 chars from start and 4 from end, this will overlap but still work
        expect(formatAddress(shortAddress)).toBe('0x1234...1234');
    });

    it('should handle null-ish values that evaluate to falsy', () => {
        // TypeScript would catch this, but testing runtime behavior
        expect(formatAddress(null as unknown as string)).toBe('');
        expect(formatAddress(undefined as unknown as string)).toBe('');
    });
});

describe('formatTimeAgo', () => {
    beforeEach(() => {
        // Mock Date.now() to return a fixed time
        vi.useFakeTimers();
        vi.setSystemTime(new Date('2026-02-05T10:00:00Z'));
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it('should format seconds ago', () => {
        const thirtySecondsAgo = '2026-02-05T09:59:30Z';
        expect(formatTimeAgo(thirtySecondsAgo)).toBe('30s ago');
    });

    it('should format minutes ago', () => {
        const fiveMinutesAgo = '2026-02-05T09:55:00Z';
        expect(formatTimeAgo(fiveMinutesAgo)).toBe('5m ago');
    });

    it('should format hours ago', () => {
        const threeHoursAgo = '2026-02-05T07:00:00Z';
        expect(formatTimeAgo(threeHoursAgo)).toBe('3h ago');
    });

    it('should format days ago', () => {
        const twoDaysAgo = '2026-02-03T10:00:00Z';
        expect(formatTimeAgo(twoDaysAgo)).toBe('2d ago');
    });

    it('should handle edge case at minute boundary', () => {
        const exactlyOneMinuteAgo = '2026-02-05T09:59:00Z';
        expect(formatTimeAgo(exactlyOneMinuteAgo)).toBe('1m ago');
    });

    it('should handle edge case at hour boundary', () => {
        const exactlyOneHourAgo = '2026-02-05T09:00:00Z';
        expect(formatTimeAgo(exactlyOneHourAgo)).toBe('1h ago');
    });

    it('should handle edge case at day boundary', () => {
        const exactlyOneDayAgo = '2026-02-04T10:00:00Z';
        expect(formatTimeAgo(exactlyOneDayAgo)).toBe('1d ago');
    });
});

describe('formatLoginDate', () => {
    it('should format a valid date string to YYYY.MM.DD at HH:MM UTC', () => {
        const dateString = '2026-01-21T10:30:00Z';
        expect(formatLoginDate(dateString)).toBe('2026.01.21 at 10:30 UTC');
    });

    it('should return "Never" for null input', () => {
        expect(formatLoginDate(null)).toBe('Never');
    });

    it('should return "Never" for undefined input', () => {
        expect(formatLoginDate(undefined)).toBe('Never');
    });

    it('should handle dates with single-digit months/days/hours/minutes', () => {
        const dateString = '2026-02-05T09:05:00Z';
        expect(formatLoginDate(dateString)).toBe('2026.02.05 at 09:05 UTC');
    });

    it('should handle midnight UTC', () => {
        const dateString = '2026-12-31T00:00:00Z';
        expect(formatLoginDate(dateString)).toBe('2026.12.31 at 00:00 UTC');
    });

    it('should handle end of day UTC', () => {
        const dateString = '2026-12-31T23:59:00Z';
        expect(formatLoginDate(dateString)).toBe('2026.12.31 at 23:59 UTC');
    });
});
