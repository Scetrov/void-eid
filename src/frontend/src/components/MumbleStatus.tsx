import { useState, useEffect, useCallback } from 'react';
import { useAuth } from '../providers/AuthProvider';
import { Mic, RefreshCw, UserPlus } from 'lucide-react';
import { API_URL, MUMBLE_SERVER_URL } from '../config';
import { CopyableField } from './CopyableField';

interface MumbleStatusResponse {
    username: string | null;
    required_tribe: string;
}

interface CreateAccountResponse {
    username: string;
    password: string;
}

export function MumbleStatus() {
    const { token, user } = useAuth();
    const [status, setStatus] = useState<MumbleStatusResponse | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [newAccount, setNewAccount] = useState<CreateAccountResponse | null>(null);

    const fetchStatus = useCallback(async () => {
        if (!token) return;
        try {
            const res = await fetch(`${API_URL}/api/mumble/status`, {
                headers: { 'Authorization': `Bearer ${token}` }
            });
            if (res.ok) {
                const data = await res.json();
                setStatus(data);
            }
        } catch (e) {
            console.error(e);
        }
    }, [token]);

    useEffect(() => {
        fetchStatus();
    }, [token, fetchStatus]);

    const handleCreateOrReset = async () => {
        if (!token) return;
        setLoading(true);
        setError(null);
        setNewAccount(null);

        try {
            const res = await fetch(`${API_URL}/api/mumble/account`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({}) // Backend extracts ID from token
            });

            if (res.ok) {
                const data = await res.json();
                setNewAccount(data);
                fetchStatus();
            } else {
                const err = await res.json();
                setError(err.error || 'Failed to create account');
            }
        } catch {
            setError('Network error');
        } finally {
            setLoading(false);
        }
    };

    if (!user) return null;

    return (
        <div style={{ maxWidth: '600px', margin: '0 auto', padding: '2rem' }}>
            <div className="card">
                <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginBottom: '1.5rem' }}>
                    <Mic size={32} style={{ color: 'var(--brand-orange)' }} />
                    <h2 style={{ margin: 0 }}>Voice Chat (Mumble)</h2>
                </div>

                <p style={{ color: 'var(--text-secondary)', marginBottom: '2rem' }}>
                    Connect to the tribe voice server at <strong style={{ color: 'var(--text-primary)' }}>{MUMBLE_SERVER_URL}</strong>. You need to be a member of the <strong>{status?.required_tribe || '...'}</strong> tribe to access voice.
                </p>

                {error && (
                    <div style={{
                        padding: '1rem',
                        background: 'rgba(239, 68, 68, 0.1)',
                        border: '1px solid rgba(239, 68, 68, 0.2)',
                        color: '#ef4444',
                        borderRadius: 'var(--radius-sm)',
                        marginBottom: '1rem'
                    }}>
                        {error}
                    </div>
                )}

                {status?.username ? (
                    <div>
                        <div style={{ marginBottom: '1.5rem' }}>
                            <CopyableField
                                label="Username"
                                value={status.username}
                            />
                        </div>

                        <button
                            onClick={handleCreateOrReset}
                            disabled={loading}
                            className="btn btn-secondary"
                            style={{
                                display: 'inline-flex',
                                alignItems: 'center',
                                gap: '0.5rem',
                                width: '100%'
                            }}
                        >
                            <RefreshCw size={18} />
                            {loading ? 'Resetting...' : 'Reset Password'}
                        </button>
                    </div>
                ) : (
                    <div>
                         <button
                            onClick={handleCreateOrReset}
                            disabled={loading}
                            className="btn btn-primary"
                            style={{
                                display: 'inline-flex',
                                alignItems: 'center',
                                gap: '0.5rem',
                                width: '100%',
                                justifyContent: 'center'
                            }}
                        >
                            <UserPlus size={20} />
                            {loading ? 'Creating...' : 'Create Account'}
                        </button>
                    </div>
                )}

                {newAccount && (
                    <div style={{ marginTop: '2rem', padding: '1.5rem', background: 'rgba(74, 222, 128, 0.1)', border: '1px solid #4ade80', borderRadius: 'var(--radius-sm)' }}>
                        <h3 style={{ marginTop: 0, color: '#4ade80' }}>Account Credentials</h3>
                        <p style={{ fontSize: '0.9rem', color: 'var(--text-primary)' }}>
                            <strong>Important:</strong> This password is shown only once. Save it now.
                        </p>

                        <div style={{ display: 'grid', gap: '1rem', marginTop: '1rem' }}>
                            <CopyableField label="Username" value={newAccount.username} />
                            <CopyableField label="Password" value={newAccount.password} />
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
