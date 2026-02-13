import { createLazyFileRoute } from '@tanstack/react-router'
import { ConnectButton, useCurrentAccount, useDisconnectWallet } from '@mysten/dapp-kit'
import { DashboardLayout } from '../components/DashboardLayout'
import { useAuth } from '../providers/AuthProvider'
import { Link } from '@tanstack/react-router'
import { useState } from 'react'
import { Calendar, Layers, Wallet, Clock, ShieldAlert } from 'lucide-react'
import DiscordLogo from "../assets/discord.svg";
import { formatAddress, formatTimeAgo, formatLoginDate, getExplorerUrl, getNetworkLabel } from '../utils';
import { SUI_NETWORK } from '../config';
import { ConfirmationModal } from '../components/ConfirmationModal';

export const Route = createLazyFileRoute('/home')({
  component: Home,
})

function Home() {
    const { isAuthenticated, user, linkWallet, unlinkWallet, deleteAccount } = useAuth()
    const realAccount = useCurrentAccount()
    // Allow E2E tests to override the connected account
    const mockAccount = (window as unknown as { __MOCK_ACCOUNT__: unknown }).__MOCK_ACCOUNT__;
    const currentAccount = (mockAccount || realAccount) as (typeof realAccount);
    const { mutate: disconnect } = useDisconnectWallet()
    const [isLinking, setIsLinking] = useState(false)
    const [linkError, setLinkError] = useState<string|null>(null)
    const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false)

    if (!isAuthenticated || !user) {
        return (
            <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: '100vh', flexDirection: 'column', gap: '1rem' }}>
                <h2>Access Denied</h2>
                <p>Please log in with Discord to view the dashboard.</p>
                <Link to="/login" className="btn btn-primary">Go to Login</Link>
            </div>
        )
    }

    // Validation Logic:
    // targetNetwork is the expected chain ID (e.g. 'sui:testnet')
    const targetNetwork = `sui:${SUI_NETWORK}`;

    // Use account metadata to detect network
    const walletChain = currentAccount?.chains && currentAccount.chains.length > 0 ? currentAccount.chains[0] : null;
    const isWrongNetwork = !!(currentAccount && walletChain && walletChain !== targetNetwork);
    const currentNetworkLabel = walletChain ? walletChain.split(':')[1] : 'unknown';

    const handleLinkWallet = async () => {
        if (!currentAccount) return;
        if (isWrongNetwork) {
            setLinkError(`Please switch your wallet to ${SUI_NETWORK}. Currently on ${currentNetworkLabel}.`);
            return;
        }
        setIsLinking(true);
        setLinkError(null);
        try {
            // Extract network from walletChain or default
            let network = SUI_NETWORK;
            if (walletChain) {
                 const parts = walletChain.split(':');
                 if (parts.length > 1) network = parts[1];
            }

            await linkWallet(currentAccount.address, network);
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

    const alreadyLinked = user.wallets.some(w => currentAccount && w.address.toLowerCase() === currentAccount.address.toLowerCase());

    return (
        <DashboardLayout>
            {/* Last Login Banner */}
            {user.lastLoginAt && (
                <div style={{
                    background: 'linear-gradient(90deg, rgba(255, 116, 0, 0.1), rgba(255, 116, 0, 0.2))',
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
                        (<div className="card" style={{ border: "2px dashed var(--accent-secondary)", background: "rgba(255, 116, 0, 0.05)" }}>
                            <div style={{ textAlign: 'center', padding: '2rem 1rem' }}>
                                <Wallet size={48} style={{ color: 'var(--accent-primary)', marginBottom: '1rem' }} />
                                <h3>Let's get you set up</h3>
                                <p style={{ color: 'var(--text-secondary)', marginBottom: '2rem' }}>
                                    Connect your first Sui Wallet to link it to your account.
                                </p>

                                <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '1rem' }}>
                                    <ConnectButton />
                                    {currentAccount && (
                                        <>
                                            {!isWrongNetwork && (
                                                <button
                                                    className="btn btn-primary"
                                                    onClick={handleLinkWallet}
                                                    disabled={isLinking}
                                                    style={{ marginTop: '1rem' }}
                                                >
                                                    {isLinking ? 'Verifying...' : 'Link Connected Wallet'}
                                                </button>
                                            )}
                                            {isWrongNetwork && (
                                                <div style={{
                                                    marginTop: '1.5rem',
                                                    color: '#eab308',
                                                    background: 'rgba(234, 179, 8, 0.1)',
                                                    padding: '1.5rem',
                                                    borderRadius: 'var(--radius-md)',
                                                    border: '1px solid rgba(234, 179, 8, 0.2)',
                                                    display: 'flex',
                                                    flexDirection: 'column',
                                                    alignItems: 'center',
                                                    gap: '1rem',
                                                    width: '100%'
                                                }}>
                                                    <ShieldAlert size={32} />
                                                    <div style={{ textAlign: 'center', lineHeight: '1.8' }}>
                                                        <strong style={{ display: 'block', marginBottom: '0.5rem', fontSize: '1.1rem' }}>Wrong Network</strong>
                                                        Your wallet is currently on <code style={{background:'rgba(0,0,0,0.2)', padding: '0.2rem 0.4rem', borderRadius: '4px'}}>{currentNetworkLabel}</code>.<br/>
                                                        Please switch to <code style={{background:'rgba(0,0,0,0.2)', padding: '0.2rem 0.4rem', borderRadius: '4px'}}>{SUI_NETWORK}</code> to continue.
                                                    </div>
                                                </div>
                                            )}
                                        </>
                                    )}
                                </div>
                                {linkError && (
                                    <p style={{ color: '#ef4444', marginTop: '1rem' }}>{linkError}</p>
                                )}
                            </div>
                        </div>)
                    ) : (
                        // Standard Dashboard State
                        (<>
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
                                                !isWrongNetwork && (
                                                    <button
                                                        className="btn btn-primary"
                                                        onClick={handleLinkWallet}
                                                        disabled={isLinking}
                                                    >
                                                        {isLinking ? 'Verifying...' : 'Link this Wallet'}
                                                    </button>
                                                )
                                            )}
                                        </div>
                                        {isWrongNetwork && (
                                            <div style={{
                                                marginTop: '1.5rem',
                                                color: '#eab308',
                                                background: 'rgba(234, 179, 8, 0.1)',
                                                padding: '1.25rem',
                                                borderRadius: 'var(--radius-md)',
                                                border: '1px solid rgba(234, 179, 8, 0.2)',
                                                display: 'flex',
                                                alignItems: 'center',
                                                gap: '1rem'
                                            }}>
                                                <ShieldAlert size={24} style={{ flexShrink: 0 }} />
                                                <div style={{ fontSize: '0.95rem', lineHeight: '1.8' }}>
                                                    <strong>Wrong Network:</strong> Your wallet is on <code style={{background:'rgba(0,0,0,0.2)', padding: '0.1rem 0.3rem', borderRadius: '3px'}}>{currentNetworkLabel}</code>.
                                                    Please switch to <code style={{background:'rgba(0,0,0,0.2)', padding: '0.1rem 0.3rem', borderRadius: '3px'}}>{SUI_NETWORK}</code> inside your wallet.
                                                </div>
                                            </div>
                                        )}

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
                                            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                                <a
                                                    href={getExplorerUrl(wallet.network || 'mainnet', wallet.address, 'address')}
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    style={{ textDecoration: 'none', color: 'inherit' }}
                                                >
                                                    <code style={{ background: 'transparent', padding: 0, cursor: 'pointer' }}>{formatAddress(wallet.address)}</code>
                                                </a>
                                                {(() => {
                                                    const { label, color, bgColor } = getNetworkLabel(wallet.network || 'mainnet');
                                                    return (
                                                        <span style={{ fontSize: '0.7rem', padding: '0.1rem 0.4rem', borderRadius: '4px', backgroundColor: bgColor, color: color, marginLeft: '0.5rem' }}>
                                                            {label}
                                                        </span>
                                                    );
                                                })()}
                                                {wallet.tribes && wallet.tribes.length > 0 && (
                                                    <span style={{
                                                        color: 'var(--brand-orange)',
                                                        fontSize: '0.75rem',
                                                        fontFamily: 'var(--font-heading)',
                                                        letterSpacing: '1px'
                                                    }}>
                                                        [{wallet.tribes.join(', ')}]
                                                    </span>
                                                )}
                                            </div>
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
                        </>)
                    )}
                </div>
            </div>
            {/* Danger Zone */}
            <div style={{ marginTop: '3rem', borderTop: '1px solid var(--glass-border)', paddingTop: '2rem' }}>
                <h3 style={{ color: '#ef4444', display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '1rem' }}>
                    <ShieldAlert size={20} />
                    Danger Zone
                </h3>
                <div className="card" style={{ border: '1px solid rgba(239, 68, 68, 0.2)', background: 'rgba(239, 68, 68, 0.05)' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <div>
                            <h4 style={{ margin: 0, color: 'var(--text-primary)' }}>Delete Profile</h4>
                            <p style={{ margin: '0.5rem 0 0 0', color: 'var(--text-secondary)', fontSize: '0.875rem' }}>
                                Permanently delete your account and all associated data. This action cannot be undone.
                            </p>
                        </div>
                        <button
                            className="btn"
                            style={{ backgroundColor: '#ef4444', color: 'white', border: 'none' }}
                            onClick={() => setIsDeleteModalOpen(true)}
                        >
                            Delete Profile
                        </button>
                    </div>
                </div>
            </div>

            <ConfirmationModal
                isOpen={isDeleteModalOpen}
                title="Delete Profile"
                message={`This action is irreversible, and designed to enable your Right to be Forgotten / Erasure under the General Data Protection Regulation (GDPR).

Are you absolutely sure you want to delete your profile? This will permanently block your Discord ID and Wallet Addresses from this platform and as such is irreversible.`}
                confirmText="Delete Profile"
                onConfirm={() => {
                    deleteAccount();
                    setIsDeleteModalOpen(false);
                }}
                onCancel={() => setIsDeleteModalOpen(false)}
                countdownSeconds={30}
            />
        </DashboardLayout>
    )
}
