import { useEffect, useRef } from 'react';
import { CipherNavText, type CipherNavTextHandle } from './CipherNavText';

interface ApiUnavailableScreenProps {
    statusText: string;
}

export function ApiUnavailableScreen({ statusText }: ApiUnavailableScreenProps) {
    const textRef = useRef<CipherNavTextHandle>(null);

    useEffect(() => {
        // Trigger animation on mount and periodically
        const interval = setInterval(() => {
            textRef.current?.trigger();
        }, 3000);

        // Initial trigger
        setTimeout(() => textRef.current?.trigger(), 100);

        return () => clearInterval(interval);
    }, []);

    return (
        <div style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            height: '100vh',
            width: '100vw',
            backgroundColor: 'var(--bg-primary)',
            color: 'var(--text-primary)',
            gap: '1rem'
        }}>
            <h1 style={{
                fontFamily: "'Diskette Mono', monospace",
                fontSize: '2.5rem',
                fontWeight: 700,
                margin: 0,
                letterSpacing: '0.05em'
            }}>
                <CipherNavText
                    ref={textRef}
                    text="CONNECTING"
                    scrambleDuration={1000}
                    scrambleSpeed={50}
                />
            </h1>
            <p style={{
                fontFamily: "'Inter', sans-serif",
                fontSize: '1rem',
                color: 'var(--text-secondary)',
                margin: 0,
                opacity: 0.8
            }}>
                {statusText}
            </p>
            <div style={{
                marginTop: '2rem',
                width: '40px',
                height: '40px',
                border: '3px solid var(--border-color)',
                borderTop: '3px solid var(--accent-primary)',
                borderRadius: '50%',
                animation: 'spin 1s linear infinite'
            }}>
                <style>{`
                    @keyframes spin {
                        0% { transform: rotate(0deg); }
                        100% { transform: rotate(360deg); }
                    }
                `}</style>
            </div>
        </div>
    );
}
