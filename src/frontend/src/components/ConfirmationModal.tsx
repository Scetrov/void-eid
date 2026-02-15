import { useState, useEffect } from 'react';
import { AlertTriangle, X } from 'lucide-react';

interface ConfirmationModalProps {
    isOpen: boolean;
    onConfirm: () => void;
    onCancel: () => void;
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    countdownSeconds?: number;
}

export function ConfirmationModal({
    isOpen,
    onConfirm,
    onCancel,
    title,
    message,
    confirmText = 'Confirm',
    cancelText = 'Cancel',
    countdownSeconds = 30
}: ConfirmationModalProps) {
    const [timeLeft, setTimeLeft] = useState(countdownSeconds);
    const [openVersion, setOpenVersion] = useState(0);

    // Track when modal opens to trigger reset
    useEffect(() => {
        if (isOpen) {
            const timer = setTimeout(() => {
                setOpenVersion(v => v + 1);
            }, 0);
            return () => clearTimeout(timer);
        }
    }, [isOpen]);

    // Reset countdown when modal opens (via openVersion change)
    useEffect(() => {
        if (openVersion > 0) {
            const timer = setTimeout(() => {
                setTimeLeft(countdownSeconds);
            }, 0);
            return () => clearTimeout(timer);
        }
    }, [openVersion, countdownSeconds]);

    // Handle Escape key while modal is open
    useEffect(() => {
        if (!isOpen) {
            return;
        }

        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                onCancel();
            }
        };

        window.addEventListener('keydown', handleEscape);

        return () => {
            window.removeEventListener('keydown', handleEscape);
        };
    }, [isOpen, onCancel]);

    // Countdown timer logic
    useEffect(() => {
        if (!isOpen || timeLeft <= 0) {
            return;
        }

        const timer = window.setInterval(() => {
            setTimeLeft((prev) => prev - 1);
        }, 1000);

        return () => {
            window.clearInterval(timer);
        };
    }, [isOpen, timeLeft]);

    if (!isOpen) return null;

    return (
        <div style={{
            position: 'fixed',
            top: 0,
            left: 0,
            width: '100vw',
            height: '100vh',
            backgroundColor: 'rgba(0, 0, 0, 0.85)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 10000,
            backdropFilter: 'blur(4px)',
            padding: '1rem'
        }}>
            <div
                className="card"
                role="dialog"
                aria-modal="true"
                aria-labelledby="modal-title"
                style={{
                    maxWidth: '500px',
                    width: '100%',
                    padding: '2rem',
                    border: '1px solid var(--border-color)',
                    background: 'var(--panel-bg)',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '1.5rem'
                }}
            >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', color: '#ef4444' }}>
                        <AlertTriangle size={24} />
                        <h3 id="modal-title" style={{ margin: 0, color: '#ef4444' }}>{title}</h3>
                    </div>
                    <button
                        onClick={onCancel}
                        aria-label="Close"
                        style={{
                            background: 'none',
                            border: 'none',
                            color: 'var(--text-secondary)',
                            cursor: 'pointer',
                            padding: '4px',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center'
                        }}
                    >
                        <X size={20} />
                    </button>
                </div>

                <div style={{ color: 'var(--text-primary)', lineHeight: '1.6', fontSize: '1rem', whiteSpace: 'pre-wrap' }}>
                    {message}
                </div>

                <div style={{ display: 'flex', gap: '1rem', marginTop: '1rem' }}>
                    <button
                        className="btn btn-secondary"
                        onClick={onCancel}
                        style={{ flex: 1 }}
                    >
                        {cancelText}
                    </button>
                    <button
                        className="btn btn-primary"
                        onClick={onConfirm}
                        disabled={timeLeft > 0}
                        style={{
                            flex: 1,
                            backgroundColor: timeLeft > 0 ? 'rgba(239, 68, 68, 0.1)' : '#ef4444',
                            color: timeLeft > 0 ? '#ef4444' : 'white',
                            borderColor: '#ef4444',
                            opacity: timeLeft > 0 ? 0.7 : 1,
                            cursor: timeLeft > 0 ? 'not-allowed' : 'pointer'
                        }}
                    >
                        {timeLeft > 0 ? `${confirmText} (${timeLeft}s)` : confirmText}
                    </button>
                </div>
            </div>
        </div>
    );
}
