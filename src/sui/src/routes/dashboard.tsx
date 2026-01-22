import { createFileRoute } from '@tanstack/react-router'
import { ConnectButton, useCurrentAccount, useDisconnectWallet } from '@mysten/dapp-kit'
import { DashboardLayout } from '../components/DashboardLayout'
import { useAuth } from '../providers/AuthProvider'
import { Link } from '@tanstack/react-router'
import { useState } from 'react'
import { Calendar, Layers, Wallet, Clock } from 'lucide-react'
import DiscordLogo from "../assets/Discord-Symbol-White.png";
import { formatAddress, formatTimeAgo, formatLoginDate } from '../utils';

export const Route = createFileRoute('/dashboard')({
  component: Dashboard,
})

function Dashboard() {
    const { isAuthenticated, user, linkWallet, unlinkWallet } = useAuth()
    const currentAccount = useCurrentAccount()
    const { mutate: disconnect } = useDisconnectWallet()
    const [isLinking, setIsLinking] = useState(false)
    const [linkError, setLinkError] = useState<string|null>(null)

    const handleLinkWallet = async () => {
        if (!currentAccount) return;
        setIsLinking(true);
        setLinkError(null);
        try {
            await linkWallet(currentAccount.address);
            disconnect(); // Disconnect after successful link
        } catch (e: unknown) {
            if (e instanceof Error) {
                setLinkError(e.message);
            } else {
                setLinkError('An unknown error occurred');
            }
        } finally {
            setIsLinking(false);
        }
    }

    if (!isAuthenticated || !user) {
        return (
            <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: '100vh', flexDirection: 'column', gap: '1rem' }}>
                <h2>Access Denied</h2>
                <p>Please log in with Discord to view the dashboard.</p>
                <Link to="/login" className="btn btn-primary">Go to Login</Link>
            </div>
        )
    }

    const alreadyLinked = user.wallets.some(w => currentAccount && w.address.toLowerCase() === currentAccount.address.toLowerCase());

    return (

        <DashboardLayout>
            {/* Last Login Banner */}
            {user.lastLoginAt && (
                <div style={{
                    background: 'linear-gradient(90deg, rgba(255, 116, 0, 0.1), rgba(34, 197, 94, 0.1))',
                    border: '1px solid rgba(255, 116, 0, 0.3)',
                    borderRadius: 'var(--radius-sm)',
                    padding: '0.75rem 1rem',
                    marginBottom: '1.5rem',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '0.75rem'
                }}>
                    <Clock size={18} style={{ color: 'var(--accent-primary)' }} />
                    <span style={{ color: 'var(--text-secondary)' }}>
                        Last logged in on <strong style={{ color: 'var(--text-primary)' }}>{formatLoginDate(user.lastLoginAt)}</strong>
                    </span>
                </div>
            )}
            <div className="dashboard-grid">
                {/* User Profile Card */}
                <div className="card" style={{ height: 'fit-content' }}>
                    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', textAlign: 'center', gap: '1rem' }}>
                        {/* Avatar Logic: Try to show image, fall back to Initials on error */}
                        <div style={{ position: 'relative', width: '100px', height: '100px' }}>
                            {user.avatar ? (
                                <img
                                    src={`https://cdn.discordapp.com/avatars/${user.discordId}/${user.avatar}.png`}
                                    alt={user.username}
                                    style={{
                                        width: '100%',
                                        height: '100%',
                                        borderRadius: '50%',
                                        border: '4px solid var(--glass-border)',
                                        objectFit: 'cover',
                                        background: 'var(--bg-secondary)' // prevent transparent background issues
                                    }}
                                    onError={(e) => {
                                        // On error, hide this img and let the fallback behind it show?
                                        // Better: Replace with fallback.
                                        e.currentTarget.style.display = 'none';
                                        const fallback = document.getElementById('avatar-fallback');
                                        if (fallback) fallback.style.display = 'flex';
                                    }}
                                />
                            ) : null}

                            {/* Fallback Initials - Hidden by default if avatar exists, shown on error or if no avatar */}
                            <div
                                id="avatar-fallback"
                                style={{
                                    display: user.avatar ? 'none' : 'flex', // If avatar exists, hide initially. OnError will show it.
                                    width: '100%',
                                    height: '100%',
                                    borderRadius: '50%',
                                    background: 'var(--bg-secondary)',
                                    alignItems: 'center',
                                    justifyContent: 'center',
                                    fontSize: '2rem',
                                    border: '4px solid var(--glass-border)',
                                    position: user.avatar ? 'absolute' : 'relative',
                                    top: 0,
                                    left: 0,
                                    zIndex: -1
                                }}
                            >
                                {user.username.charAt(0).toUpperCase()}
                            </div>
                        </div>

                        <div>
                            <h3 style={{ margin: 0 }}>{user.username}</h3>
                            {user.discriminator !== '0' && (
                                <p style={{ color: 'var(--text-secondary)', margin: '0.25rem 0' }}>#{user.discriminator}</p>
                            )}
                        </div>
                        <div style={{
                            padding: '0.75rem 1rem',
                            margin: '0.5rem 0',
                            background: 'rgba(88, 101, 242, 0.1)',
                            color: '#5865F2',
                            borderRadius: 'var(--radius-sm)',
                            fontWeight: 600,
                            display: 'flex',
                            alignItems: 'center',
                            gap: '0.5rem'
                        }}>
                            <div style={{ width: '26px', height: '20px', display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0 }}>
                                <img src={DiscordLogo} alt="Discord" style={{ width: '100%', height: '100%', objectFit: 'contain' }} />
                            </div>
                            Discord Connected
                        </div>
                    </div>
                </div>

                {/* Wallets Section */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>

                    {user.wallets.length === 0 ? (
                        // Onboarding State
                        <div className="card" style={{ border: '2px dashed var(--accent-secondary)', background: 'rgba(255, 116, 0, 0.05)' }}>
                            <div style={{ textAlign: 'center', padding: '2rem 1rem' }}>
                                <Wallet size={48} style={{ color: 'var(--accent-primary)', marginBottom: '1rem' }} />
                                <h3>Let's get you set up</h3>
                                <p style={{ color: 'var(--text-secondary)', marginBottom: '2rem' }}>
                                    Connect your first Sui Wallet to link it to your account.
                                </p>

                                <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '1rem' }}>
                                    <ConnectButton />
                                    {currentAccount && (
                                        <button
                                            className="btn btn-primary"
                                            onClick={handleLinkWallet}
                                            disabled={isLinking}
                                            style={{ marginTop: '1rem' }}
                                        >
                                            {isLinking ? 'Verifying...' : 'Link Connected Wallet'}
                                        </button>
                                    )}
                                </div>
                                {linkError && (
                                    <p style={{ color: '#ef4444', marginTop: '1rem' }}>{linkError}</p>
                                )}
                            </div>
                        </div>
                    ) : (
                        // Standard Dashboard State
                        <>
                             {/* Link New Wallet */}
                            <div className="card">
                                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.5rem' }}>
                                    <h3 style={{ margin: 0, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                        <Wallet size={20} />
                                        Link Another Wallet
                                    </h3>
                                    <ConnectButton />
                                </div>

                                {currentAccount ? (
                                    <div style={{ background: 'var(--bg-secondary)', padding: '1.5rem', borderRadius: 'var(--radius-md)' }}>
                                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                            <div>
                                                <p style={{ margin: '0 0 0.5rem 0', fontSize: '0.875rem', color: 'var(--text-secondary)' }}>Active Wallet</p>
                                                <code style={{ fontSize: '1rem' }}>{formatAddress(currentAccount.address)}</code>
                                            </div>
                                            {alreadyLinked ? (
                                                <span style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>✓ Linked</span>
                                            ) : (
                                                <button
                                                    className="btn btn-primary"
                                                    onClick={handleLinkWallet}
                                                    disabled={isLinking}
                                                >
                                                    {isLinking ? 'Verifying...' : 'Link this Wallet'}
                                                </button>
                                            )}
                                        </div>
                                        {linkError && (
                                            <div style={{ marginTop: '1rem', color: '#ef4444', background: 'rgba(239, 68, 68, 0.1)', padding: '0.75rem', borderRadius: 'var(--radius-sm)' }}>
                                                Error: {linkError}
                                            </div>
                                        )}
                                    </div>
                                ) : (
                                    <p style={{ color: 'var(--text-secondary)', textAlign: 'center', padding: '2rem', border: '2px dashed var(--glass-border)', borderRadius: 'var(--radius-md)' }}>
                                        Connect a new wallet to link it.
                                    </p>
                                )}
                            </div>

                            {/* Linked Wallets List */}
                            <div className="card">
                                <h3 style={{ margin: '0 0 1.5rem 0', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                    <Layers size={20} />
                                    Linked Wallets ({user.wallets.length})
                                </h3>

                                <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                                    {user.wallets.map(wallet => (
                                        <div key={wallet.id} style={{
                                            padding: '1rem',
                                            borderRadius: 'var(--radius-sm)',
                                            border: '1px solid var(--glass-border)',
                                            display: 'flex',
                                            justifyContent: 'space-between',
                                            alignItems: 'center'
                                        }}>
                                            <code style={{ background: 'transparent', padding: 0 }}>{formatAddress(wallet.address)}</code>
                                            <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', color: 'var(--text-secondary)', fontSize: '0.875rem' }}>
                                                <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                                    <Calendar size={14} />
                                                    {formatTimeAgo(wallet.verifiedAt)}
                                                </div>
                                                <button
                                                    className="btn btn-secondary"
                                                    style={{ padding: '0.25rem 0.5rem', fontSize: '0.75rem', height: 'auto', border: '1px solid var(--text-secondary)' }}
                                                    onClick={() => unlinkWallet(wallet.id)}
                                                    title="Unlink Wallet"
                                                >
                                                    Unlink
                                                </button>
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        </>
                    )}
                </div>
            </div>
        </DashboardLayout>
    )
}
