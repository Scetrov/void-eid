import { createFileRoute, Link, useNavigate } from '@tanstack/react-router'
import { useAuth } from '../../providers/AuthProvider'
import { API_URL } from '../../config'
import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import { ArrowUpDown, Search, ShieldAlert } from 'lucide-react'
import { DashboardLayout } from '../../components/DashboardLayout'

import { useDebounce } from '../../hooks/useDebounce'

export const Route = createFileRoute('/roster/')({
  component: RosterPage,
})

interface RosterMember {
    discordId: string;
    username: string;
    avatar: string | null;
    lastLoginAt: string | null;
    wallets: {
        id: string;
        address: string;
        deletedAt?: string;
        tribes: string[];
    }[];
}

function RosterPage() {
    const { user, token, isAuthenticated, currentTribe } = useAuth()
    const [search, setSearch] = useState('')
    const debouncedSearch = useDebounce(search, 500)
    const [sort, setSort] = useState<'username' | 'wallet_count' | 'last_login'>('username')
    const [order, setOrder] = useState<'asc' | 'desc'>('asc')
    const navigate = useNavigate()

    // Fetch Roster
    const { data: roster, isLoading, error } = useQuery({
        queryKey: ['roster', currentTribe, debouncedSearch, sort, order],
        queryFn: async () => {
            if (!token) throw new Error("No token");

            const params = new URLSearchParams();
            if (currentTribe) params.append('tribe', currentTribe);
            if (debouncedSearch) params.append('search', debouncedSearch);
            params.append('sort', sort);
            params.append('order', order);

            const res = await fetch(`${API_URL}/api/roster?${params.toString()}`, {
                headers: { 'Authorization': `Bearer ${token}` }
            });

            if (!res.ok) {
                 if (res.status === 403) throw new Error("Access Denied: You must be an admin to view this page.");
                 if (res.status === 400) throw new Error("Please select a tribe from the dropdown above.");
                 throw new Error("Failed to fetch roster");
            }
            return res.json() as Promise<RosterMember[]>;
        },
        enabled: !!token && (!!user?.isAdmin || (user?.adminTribes?.length ?? 0) > 0),
        retry: false
    });

    if (!isAuthenticated) {
        return (
             <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: '60vh', flexDirection: 'column', gap: '1rem' }}>
                <h2>Please Login</h2>
                <Link to="/login" className="btn btn-primary">Login</Link>
             </div>
        )
    }

    const hasAdminAccess = user && (user.isAdmin || (user.adminTribes && user.adminTribes.length > 0));
    // If a specific tribe is selected, verify admin status for that tribe
    const isTribeAdmin = currentTribe ? user?.adminTribes?.includes(currentTribe) : true; // If no tribe selected, we might default or show error, but layout handles selection.

    if (user && !hasAdminAccess) {
        return (
            <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: '60vh', flexDirection: 'column', gap: '1rem', textAlign: 'center' }}>
                <ShieldAlert size={64} style={{ color: '#ef4444' }} />
                <h2>Access Denied</h2>
                <p>Only users with the 'Admin' role can view the Roster.</p>
                <Link to="/home" className="btn btn-secondary">Back to Home</Link>
            </div>
        )
    }

    if (currentTribe && !isTribeAdmin && !user?.isAdmin) {
         return (
            <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: '60vh', flexDirection: 'column', gap: '1rem', textAlign: 'center' }}>
                <ShieldAlert size={64} style={{ color: '#ef4444' }} />
                <h2>Access Denied</h2>
                <p>You are not an Admin of the <strong>{currentTribe}</strong> tribe.</p>
                <Link to="/home" className="btn btn-secondary">Back to Home</Link>
            </div>
        )
    }

    const toggleSort = (field: 'username' | 'wallet_count' | 'last_login') => {
        if (sort === field) {
            setOrder(order === 'asc' ? 'desc' : 'asc');
        } else {
            setSort(field);
            setOrder('asc');
        }
    }

    return (
        <DashboardLayout>
            <div className="dashboard-header" style={{ marginBottom: '2rem' }}>
                <h2 style={{ margin: 0 }}>Roster</h2>
            </div>

            <div className="card">
                <div style={{ display: 'flex', gap: '1rem', marginBottom: '1.5rem' }}>
                    <div style={{ position: 'relative', flex: 1 }}>
                        <Search size={18} style={{ position: 'absolute', left: '1rem', top: '50%', transform: 'translateY(-50%)', color: 'var(--text-secondary)' }} />
                        <input
                            type="text"
                            placeholder="Search by username or Discord ID..."
                            value={search}
                            onChange={(e) => setSearch(e.target.value)}
                            style={{
                                width: '100%',
                                padding: '0.75rem 1rem 0.75rem 2.75rem',
                                background: 'var(--bg-secondary)',
                                border: '1px solid var(--glass-border)',
                                borderRadius: 'var(--radius-sm)',
                                color: 'var(--text-primary)',
                                outline: 'none'
                             }}
                        />
                    </div>
                </div>

                {error ? (
                     <div style={{ padding: '2rem', textAlign: 'center', color: '#ef4444' }}>
                        Error: {(error as Error).message}
                     </div>
                ) : isLoading ? (
                    <div style={{ padding: '2rem', textAlign: 'center', color: 'var(--text-secondary)' }}>
                        Loading roster...
                    </div>
                ) : (
                    <div className="roster-table-container">
                        <table className="roster-table" style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
                            <thead>
                                <tr style={{ borderBottom: '1px solid var(--glass-border)' }}>
                                    <th style={{ padding: '1rem', cursor: 'pointer' }} onClick={() => toggleSort('username')}>
                                        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                            Member
                                            {sort === 'username' && <ArrowUpDown size={14} />}
                                        </div>
                                    </th>
                                    <th style={{ padding: '1rem' }}>Discord ID</th>
                                    <th style={{ padding: '1rem', cursor: 'pointer' }} onClick={() => toggleSort('last_login')}>
                                        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                            Last Login
                                            {sort === 'last_login' && <ArrowUpDown size={14} />}
                                        </div>
                                    </th>
                                    <th style={{ padding: '1rem', cursor: 'pointer' }} onClick={() => toggleSort('wallet_count')}>
                                        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                            Wallets
                                            {sort === 'wallet_count' && <ArrowUpDown size={14} />}
                                        </div>
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                {roster?.length === 0 ? (
                                    <tr>
                                        <td colSpan={4} style={{ padding: '2rem', textAlign: 'center', color: 'var(--text-secondary)' }}>
                                            No members found.
                                        </td>
                                    </tr>
                                ) : (
                                    roster?.map((member) => (
                                        <tr
                                            key={member.discordId}
                                            style={{ borderBottom: '1px solid rgba(255,255,255,0.05)', cursor: 'pointer' }}
                                            onClick={() => navigate({ to: '/roster/$id', params: { id: member.discordId } })}
                                            className="roster-row"
                                        >
                                            <td style={{ padding: '1rem' }} data-label="Member">
                                                <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
                                                    {member.avatar ? (
                                                        <img
                                                            src={`https://cdn.discordapp.com/avatars/${member.discordId}/${member.avatar}.png`}
                                                            alt={member.username}
                                                            style={{ width: '32px', height: '32px', borderRadius: '50%' }}
                                                        />
                                                    ) : (
                                                        <div style={{ width: '32px', height: '32px', borderRadius: '50%', background: 'var(--bg-secondary)', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '0.8rem' }}>
                                                            {member.username.charAt(0).toUpperCase()}
                                                        </div>
                                                    )}
                                                    {member.username}
                                                </div>
                                            </td>
                                            <td style={{ padding: '1rem', fontFamily: 'monospace', color: 'var(--text-secondary)' }} data-label="Discord ID">
                                                {member.discordId}
                                            </td>
                                            <td style={{ padding: '1rem', color: 'var(--text-secondary)', fontSize: '0.9rem' }} data-label="Last Login">
                                                {member.lastLoginAt ? new Date(member.lastLoginAt).toLocaleString() : 'Never'}
                                            </td>
                                            <td style={{ padding: '1rem' }} data-label="Wallets">
                                                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem' }}>
                                                    {member.wallets.map(w => (
                                                        <div key={w.id} style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', opacity: w.deletedAt ? 0.5 : 1 }}>
                                                            <code style={{ fontSize: '0.8rem', background: 'rgba(255,255,255,0.05)', padding: '0.2rem 0.4rem', borderRadius: '4px', textDecoration: w.deletedAt ? 'line-through' : 'none' }}>
                                                                {w.address.slice(0, 6)}...{w.address.slice(-4)}
                                                            </code>
                                                            {w.tribes && w.tribes.length > 0 && (
                                                                <span style={{ color: 'var(--brand-orange)', fontSize: '0.7rem' }}>
                                                                    [{w.tribes.join(', ')}]
                                                                </span>
                                                            )}
                                                        </div>
                                                    ))}
                                                    {member.wallets.length === 0 && <span style={{ color: 'var(--text-secondary)', fontSize: '0.9rem' }}>None</span>}
                                                </div>
                                            </td>
                                        </tr>
                                    ))
                                )}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>
        </DashboardLayout>
    )
}
