import { useState } from 'react';
import { Check, Copy } from 'lucide-react';

interface CopyButtonProps {
    text: string;
    label?: string;
    className?: string;
}

export function CopyButton({ text, label, className = '' }: CopyButtonProps) {
    const [copied, setCopied] = useState(false);

    const handleCopy = async () => {
        try {
            await navigator.clipboard.writeText(text);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        } catch (err) {
            console.error('Failed to copy text: ', err);
        }
    };

    return (
        <button
            onClick={handleCopy}
            className={`btn btn-ghost ${className}`}
            style={{
                padding: '0.25rem 0.5rem',
                height: 'auto',
                display: 'inline-flex',
                alignItems: 'center',
                gap: '0.5rem',
                fontSize: '0.875rem',
                color: copied ? 'var(--success-color, #4ade80)' : 'var(--text-secondary)'
            }}
            title="Copy to clipboard"
        >
            {copied ? <Check size={16} /> : <Copy size={16} />}
            {label && <span>{copied ? 'Copied!' : label}</span>}
        </button>
    );
}
