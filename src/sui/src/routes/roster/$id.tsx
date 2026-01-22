import { createFileRoute, Link } from '@tanstack/react-router'
import { useAuth } from '../../providers/AuthProvider'
import { useQuery } from '@tanstack/react-query'
import { ShieldAlert, ArrowLeft, Copy, ExternalLink, Wallet, ChevronLeft, ChevronRight } from 'lucide-react'
import { DashboardLayout } from '../../components/DashboardLayout'
import { useState } from 'react'

export const Route = createFileRoute('/roster/$id')({
  component: RosterMemberPage,
})

interface AuditLog {
    id: string;
    action: string;
    actorId: string;
    targetId: string | null;
    details: string;
    createdAt: string;
    actorUsername: string;
    actorDiscriminator: string;
}

interface PaginatedAudits {
    items: AuditLog[];
    total: number;
    page: number;
    perPage: number;
    totalPages: number;
}

interface RosterMember {
    discord_id: string;
    username: string;
    avatar: string | null;
    wallets: string[];
    audits?: PaginatedAudits;
}

function RosterMemberPage() {
    const { id } = Route.useParams()
    const { user, token, isAuthenticated } = useAuth()
    const [auditPage, setAuditPage] = useState(1)

    const { data: member, isLoading, error } = useQuery({
        queryKey: ['rosterMember', id, auditPage],
        queryFn: async () => {
            if (!token) throw new Error("No token");

            const params = new URLSearchParams();
            params.append('audit_page', auditPage.toString());
            params.append('audit_per_page', '10');

            const res = await fetch(`http://localhost:5038/api/roster/${id}?${params.toString()}`, {
                headers: { 'Authorization': `Bearer ${token}` }
            });

            if (!res.ok) {
                 if (res.status === 403) throw new Error("Access Denied: You do not have permission to view this member.");
                 if (res.status === 404) throw new Error("Member not found.");
                 throw new Error("Failed to fetch member details");
            }
            return res.json() as Promise<RosterMember>;
        },
        enabled: !!token && !!user?.isAdmin,
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

    if (user && !user.isAdmin) {
        return (
            <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: '60vh', flexDirection: 'column', gap: '1rem', textAlign: 'center' }}>
                <ShieldAlert size={64} style={{ color: '#ef4444' }} />
                <h2>Access Denied</h2>
                <p>Only users with the 'Admin' role can view Member Details.</p>
                <Link to="/dashboard" className="btn btn-secondary">Back to Dashboard</Link>
            </div>
        )
    }

    if (error) {
         return (
            <div className="dashboard-container">
                <header className="dashboard-header" style={{ marginBottom: '2rem' }}>
                     <Link to="/roster" className="btn btn-secondary" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                        <ArrowLeft size={16} /> Back to Roster
                    </Link>
                </header>
                <div className="card" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '1rem', padding: '3rem', textAlign: 'center' }}>
                     <ShieldAlert size={48} style={{ color: '#ef4444' }} />
                     <h3>Error Loading Member</h3>
                     <p style={{ color: '#ef4444' }}>{(error as Error).message}</p>
                </div>
            </div>
         )
    }

    if (isLoading) {
        return (
            <div className="dashboard-container">
                 <header className="dashboard-header" style={{ marginBottom: '2rem' }}>
                     <Link to="/roster" className="btn btn-secondary" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                        <ArrowLeft size={16} /> Back to Roster
                    </Link>
                </header>
                <div className="card" style={{ padding: '3rem', textAlign: 'center', color: 'var(--text-secondary)' }}>
                    Loading member details...
                </div>
            </div>
        )
    }

    return (
        <DashboardLayout>
            <div className="dashboard-header" style={{ marginBottom: '2rem' }}>
                 <div>
                    <h2 style={{ margin: 0 }}>Member Details</h2>
                    <p style={{ color: 'var(--text-secondary)', margin: '0.5rem 0 0 0' }}>
                        Viewing details for <strong style={{ color: 'var(--accent-primary)' }}>{member?.username}</strong>
                    </p>
                </div>
                <Link to="/roster" className="btn btn-secondary" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                    <ArrowLeft size={16} /> Back to Roster
                </Link>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: '2rem' }}>

                {/* Profile Card */}
                <div className="card">
                    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', textAlign: 'center', gap: '1rem', padding: '1rem' }}>
                        {member?.avatar ? (
                            <img
                                src={`https://cdn.discordapp.com/avatars/${member.discord_id}/${member.avatar}.png?size=256`}
                                alt={member.username}
                                style={{ width: '128px', height: '128px', borderRadius: '50%', border: '4px solid var(--glass-border)' }}
                            />
                        ) : (
                             <div style={{ width: '128px', height: '128px', borderRadius: '50%', background: 'var(--bg-secondary)', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '3rem', border: '4px solid var(--glass-border)' }}>
                                {member?.username.charAt(0).toUpperCase()}
                            </div>
                        )}

                        <div>
                            <h3 style={{ margin: 0, fontSize: '1.5rem' }}>{member?.username}</h3>
                            <code style={{ color: 'var(--text-secondary)', background: 'rgba(255,255,255,0.05)', padding: '0.2rem 0.5rem', borderRadius: '4px', marginTop: '0.5rem', display: 'inline-block' }}>
                                {member?.discord_id}
                            </code>
                        </div>
                    </div>
                </div>

                {/* Wallets Card */}
                <div className="card">
                    <h3 style={{ marginTop: 0, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                        <Wallet size={20} color="var(--accent-primary)" />
                        Linked Wallets
                    </h3>

                    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', marginTop: '1.5rem' }}>
                        {member?.wallets.length === 0 ? (
                             <div style={{ color: 'var(--text-secondary)', fontStyle: 'italic' }}>
                                No wallets linked.
                             </div>
                        ) : (
                            member?.wallets.map((wallet) => (
                                <div key={wallet} style={{
                                    background: 'var(--bg-secondary)',
                                    padding: '1rem',
                                    borderRadius: 'var(--radius-sm)',
                                    display: 'flex',
                                    justifyContent: 'space-between',
                                    alignItems: 'center',
                                    border: '1px solid var(--glass-border)'
                                }}>
                                    <code style={{ fontSize: '0.9rem', wordBreak: 'break-all' }}>{wallet}</code>
                                    <div style={{ display: 'flex', gap: '0.5rem' }}>
                                        <button
                                            onClick={() => navigator.clipboard.writeText(wallet)}
                                            style={{ background: 'transparent', border: 'none', color: 'var(--text-secondary)', cursor: 'pointer', padding: '0.25rem' }}
                                            title="Copy Address"
                                        >
                                            <Copy size={16} />
                                        </button>
                                        <a
                                            href={`https://suiscan.xyz/mainnet/account/${wallet}`}
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            style={{ color: 'var(--text-secondary)', padding: '0.25rem' }}
                                            title="View on Explorer"
                                        >
                                            <ExternalLink size={16} />
                                        </a>
                                    </div>
                                </div>
                            ))
                        )}
                    </div>
                </div>

                {/* Audit Logs Card */}
                <div className="card" style={{ gridColumn: '1 / -1' }}>
                     <h3 style={{ marginTop: 0, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                        <ShieldAlert size={20} color="var(--accent-primary)" />
                        Audit History
                    </h3>

                    <div style={{ marginTop: '1.5rem', overflowX: 'auto' }}>
                        <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
                            <thead>
                                <tr style={{ borderBottom: '1px solid var(--glass-border)' }}>
                                    <th style={{ padding: '0.75rem', color: 'var(--text-secondary)' }}>Actor</th>
                                    <th style={{ padding: '0.75rem', color: 'var(--text-secondary)' }}>Action</th>
                                    <th style={{ padding: '0.75rem', color: 'var(--text-secondary)' }}>Details</th>
                                    <th style={{ padding: '0.75rem', color: 'var(--text-secondary)' }}>Date</th>
                                </tr>
                            </thead>
                            <tbody>
                                {member?.audits?.items?.map((audit) => (
                                    <tr key={audit.id} style={{ borderBottom: '1px solid rgba(255,255,255,0.05)' }}>
                                        <td style={{ padding: '0.75rem' }}>
                                            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                                <span style={{ fontWeight: 'bold' }}>{audit.actorUsername}</span>
                                                {audit.actorDiscriminator !== '0' && (
                                                    <span style={{ color: 'var(--text-secondary)', fontSize: '0.8rem' }}>#{audit.actorDiscriminator}</span>
                                                )}
                                            </div>
                                        </td>
                                        <td style={{ padding: '0.75rem' }}>
                                            <code style={{ color: 'var(--accent-primary)', fontSize: '0.8rem' }}>{audit.action}</code>
                                        </td>
                                        <td style={{ padding: '0.75rem' }}>{audit.details}</td>
                                        <td style={{ padding: '0.75rem', color: 'var(--text-secondary)', fontSize: '0.9rem' }}>
                                            {new Date(audit.createdAt).toLocaleString()}
                                        </td>
                                    </tr>
                                )) || (
                                    <tr>
                                        <td colSpan={4} style={{ padding: '2rem', textAlign: 'center', color: 'var(--text-secondary)' }}>No audit history found.</td>
                                    </tr>
                                )}
                            </tbody>
                        </table>

                        {/* Pagination Controls */}
                        {member?.audits && member.audits.totalPages > 1 && (
                            <div style={{
                                display: 'flex',
                                justifyContent: 'space-between',
                                alignItems: 'center',
                                marginTop: '1.5rem',
                                padding: '1rem',
                                background: 'var(--bg-secondary)',
                                borderRadius: 'var(--radius-sm)'
                            }}>
                                <span style={{ color: 'var(--text-secondary)', fontSize: '0.9rem' }}>
                                    Showing {((member.audits.page - 1) * member.audits.perPage) + 1} - {Math.min(member.audits.page * member.audits.perPage, member.audits.total)} of {member.audits.total} entries
                                </span>
                                <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                                    <button
                                        className="btn btn-secondary"
                                        onClick={() => setAuditPage(p => Math.max(1, p - 1))}
                                        disabled={auditPage <= 1}
                                        style={{ padding: '0.5rem', display: 'flex', alignItems: 'center' }}
                                    >
                                        <ChevronLeft size={18} />
                                    </button>
                                    <span style={{ padding: '0 0.75rem', color: 'var(--text-primary)' }}>
                                        Page {member.audits.page} of {member.audits.totalPages}
                                    </span>
                                    <button
                                        className="btn btn-secondary"
                                        onClick={() => setAuditPage(p => Math.min(member.audits!.totalPages, p + 1))}
                                        disabled={auditPage >= member.audits.totalPages}
                                        style={{ padding: '0.5rem', display: 'flex', alignItems: 'center' }}
                                    >
                                        <ChevronRight size={18} />
                                    </button>
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </DashboardLayout>
    )
}
