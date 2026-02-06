import { useState, useRef, useImperativeHandle, forwardRef, useEffect, useCallback } from 'react';

const CHARACTERS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';

export interface CipherNavTextHandle {
    trigger: () => void;
}

interface CipherNavTextProps {
    text: string;
    scrambleSpeed?: number; // ms per frame during scramble
    scrambleDuration?: number; // ms to scramble before resolving
    className?: string;
    style?: React.CSSProperties;
}

export const CipherNavText = forwardRef<CipherNavTextHandle, CipherNavTextProps>(({
    text,
    scrambleSpeed = 100, // Slightly faster than "half a second" for better feel, but adjustable
    scrambleDuration = 1000,
    className,
    style
}, ref) => {
    const [displayText, setDisplayText] = useState(text);
    const isAnimating = useRef(false);
    const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

    const trigger = useCallback(() => {
        if (isAnimating.current) return;
        isAnimating.current = true;

        const startTime = Date.now();

        if (intervalRef.current) clearInterval(intervalRef.current);

        intervalRef.current = setInterval(() => {
            const now = Date.now();
            const elapsed = now - startTime;

            // Phase 1: Keep scrambling everything
            if (elapsed < scrambleDuration) {
                // To respect spaces or special chars (though request said restricted to A-Z, original text might have spaces?)
                // Assuming Nav headings are mostly A-Z but might have spaces.
                // We'll preserve spaces.
                const splitText = text.split('');
                const randomText = splitText.map(char => {
                    if (char === ' ') return ' ';
                    return CHARACTERS.charAt(Math.floor(Math.random() * CHARACTERS.length));
                }).join('');
                setDisplayText(randomText);
            }
            // Phase 2: Sequential Resolve
            else {
                const resolveElapsed = elapsed - scrambleDuration;
                // We want to resolve one character every X ms
                // Let's say we resolve one character every 150ms
                const resolveSpeed = scrambleSpeed;
                const charsResolved = Math.floor(resolveElapsed / resolveSpeed);

                if (charsResolved >= text.length) {
                    setDisplayText(text);
                    isAnimating.current = false;
                    if (intervalRef.current) clearInterval(intervalRef.current);
                    intervalRef.current = null;
                } else {
                    const splitText = text.split('');
                    const mixedText = splitText.map((char, index) => {
                        if (index < charsResolved) {
                            return text[index]; // Resolved
                        }
                        if (char === ' ') return ' ';
                        return CHARACTERS.charAt(Math.floor(Math.random() * CHARACTERS.length));
                    }).join('');
                    setDisplayText(mixedText);
                }
            }
        }, scrambleSpeed);

    }, [text, scrambleDuration, scrambleSpeed]);

    useImperativeHandle(ref, () => ({
        trigger
    }));

    // Cleanup
    useEffect(() => {
        return () => {
            if (intervalRef.current) clearInterval(intervalRef.current);
        };
    }, []);

    return (
        <span className={className} style={{ display: 'inline-block', minWidth: '4ch', ...style }}>
            {displayText}
        </span>
    );
});

CipherNavText.displayName = 'CipherNavText';
