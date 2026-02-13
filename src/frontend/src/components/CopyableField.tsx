import { useState } from 'react';
import { Copy, Check } from 'lucide-react';

interface CopyableFieldProps {
  label?: string;
  value: string;
  className?: string;
  style?: React.CSSProperties;
}

export const CopyableField: React.FC<CopyableFieldProps> = ({ label, value, className = '', style = {} }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy text: ', err);
    }
  };

  return (
    <div style={{
      padding: '1rem',
      backgroundColor: '#000000', // Black BG
      border: '1px solid rgba(255, 255, 255, 0.1)', // Border
      borderRadius: 'var(--radius-sm, 8px)',
      ...style
    }} className={className}>
      {label && <div style={{ color: 'var(--text-secondary)', fontSize: '0.875rem', marginBottom: '0.25rem' }}>{label}</div>}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <code style={{ color: '#ffffff', fontFamily: 'monospace', fontSize: '0.875rem', wordBreak: 'break-all' }}>{value}</code>
        <button
          onClick={handleCopy}
          style={{
            marginLeft: '0.5rem',
            padding: '0.5rem',
            background: 'transparent',
            border: 'none',
            cursor: 'pointer',
            borderRadius: '50%',
            transition: 'background-color 0.2s'
          }}
          title="Copy to clipboard"
        >
          {copied ? <Check size={16} color="#22c55e" /> : <Copy size={16} color="#9ca3af" />}
        </button>
      </div>
    </div>
  );
};
