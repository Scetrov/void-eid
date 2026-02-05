import { useState, useEffect } from 'react';
import { useAuth } from '../providers/AuthProvider';
import { Mic, RefreshCw, UserPlus } from 'lucide-react';

interface MumbleStatusResponse {
    username: string | null;
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

    const API_URL = 'http://localhost:5038'; // Matches AuthProvider

    const fetchStatus = async () => {
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
    };

    useEffect(() => {
        fetchStatus();
    }, [token]);

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
        } catch (e) {
            setError('Network error');
        } finally {
            setLoading(false);
        }
    };

    if (!user) return null;

    return (
        <div style={{ maxWidth: '600px', margin: '0 auto', padding: '2rem' }}>
            <div className="card" style={{ 
                background: 'rgba(255, 255, 255, 0.05)', 
                padding: '2rem', 
                borderRadius: '8px', 
                border: '1px solid rgba(255, 255, 255, 0.1)' 
            }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginBottom: '1.5rem' }}>
                    <Mic size={32} style={{ color: 'var(--accent-color, #4ade80)' }} />
                    <h2 style={{ margin: 0 }}>Voice Chat (Mumble)</h2>
                </div>
                
                <p style={{ color: 'var(--text-secondary)', marginBottom: '2rem' }}>
                    Connect to the tribe voice server. You need to be a member of the <strong>Fire</strong> tribe to access voice.
                </p>

                {error && (
                    <div style={{ 
                        padding: '1rem', 
                        background: 'rgba(239, 68, 68, 0.1)', 
                        border: '1px solid rgba(239, 68, 68, 0.2)', 
                        color: '#fca5a5', 
                        borderRadius: '4px',
                        marginBottom: '1rem'
                    }}>
                        {error}
                    </div>
                )}

                {status?.username ? (
                    <div>
                        <div style={{ marginBottom: '1.5rem' }}>
                            <label style={{ display: 'block', color: 'var(--text-secondary)', fontSize: '0.875rem', marginBottom: '0.5rem' }}>Username</label>
                            <div style={{ 
                                padding: '0.75rem', 
                                background: 'rgba(0,0,0,0.2)', 
                                borderRadius: '4px', 
                                fontFamily: 'monospace',
                                fontSize: '1.25rem'
                            }}>
                                {status.username}
                            </div>
                        </div>

                        <button 
                            onClick={handleCreateOrReset}
                            disabled={loading}
                            className="btn"
                            style={{ 
                                display: 'inline-flex', 
                                alignItems: 'center', 
                                gap: '0.5rem',
                                padding: '0.75rem 1.5rem',
                                background: 'transparent',
                                border: '1px solid var(--text-secondary)',
                                color: 'var(--text-primary)',
                                cursor: 'pointer'
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
                                padding: '0.75rem 1.5rem',
                                background: '#4ade80',
                                color: '#000',
                                border: 'none',
                                borderRadius: '4px',
                                fontWeight: 'bold',
                                cursor: 'pointer',
                                fontSize: '1rem'
                            }}
                        >
                            <UserPlus size={20} />
                            {loading ? 'Creating...' : 'Create Account'}
                        </button>
                    </div>
                )}

                {newAccount && (
                    <div style={{ marginTop: '2rem', padding: '1.5rem', background: 'rgba(74, 222, 128, 0.1)', border: '1px solid #4ade80', borderRadius: '8px' }}>
                        <h3 style={{ marginTop: 0, color: '#4ade80' }}>Account Credentials</h3>
                        <p style={{ fontSize: '0.9rem', color: '#fff' }}>
                            <strong>Important:</strong> This password is shown only once. Save it now.
                        </p>
                        
                        <div style={{ display: 'grid', gap: '1rem', marginTop: '1rem' }}>
                            <div>
                                <label style={{ display: 'block', fontSize: '0.8rem', opacity: 0.7 }}>Username</label>
                                <div style={{ fontFamily: 'monospace', fontSize: '1.1rem' }}>{newAccount.username}</div>
                            </div>
                            <div>
                                <label style={{ display: 'block', fontSize: '0.8rem', opacity: 0.7 }}>Password</label>
                                <div style={{ fontFamily: 'monospace', fontSize: '1.1rem', background: 'rgba(0,0,0,0.3)', padding: '0.5rem', borderRadius: '4px', wordBreak: 'break-all' }}>
                                    {newAccount.password}
                                </div>
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
