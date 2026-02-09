import { createFileRoute, Link } from '@tanstack/react-router'
import { useAuth } from '../../providers/AuthProvider'
import { useQuery, useMutation, useQueryClient, keepPreviousData } from '@tanstack/react-query'
import { ShieldAlert, ArrowLeft, ExternalLink, Wallet, ChevronLeft, ChevronRight, LogIn, Link as LinkIcon, Unlink, List, Eye, ShieldPlus, ShieldMinus, UserPlus, UserMinus, FileText, Edit2, Save, X } from 'lucide-react'
import { CopyButton } from '../../components/CopyButton'
import { DashboardLayout } from '../../components/DashboardLayout'
import { useState } from 'react'
import { API_URL } from '../../config';

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
    discordId: string;
    username: string;
    avatar: string | null;
    wallets: {
        id: string;
        address: string;
        deletedAt?: string;
        tribes: string[];
    }[];
    audits?: PaginatedAudits;
}

interface Note {
    id: string;
    targetUserId: string;
    authorId: string;
    tribe: string;
    content: string;
    createdAt: string;
    updatedAt: string;
    authorUsername: string;
    authorDiscriminator: string;
}

// Helper function to format date as YYYY-MM-DD HH:mm
function formatDateTime(dateString: string): string {
    const date = new Date(dateString);
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');
    return `${year}-${month}-${day} ${hours}:${minutes}`;
}

// Helper function to get relative time
function getRelativeTime(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffSecs = Math.floor(diffMs / 1000);
    const diffMins = Math.floor(diffSecs / 60);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);
    const diffWeeks = Math.floor(diffDays / 7);
    const diffMonths = Math.floor(diffDays / 30);
    const diffYears = Math.floor(diffDays / 365);

    if (diffSecs < 60) return 'just now';
    if (diffMins < 60) return `${diffMins} minute${diffMins !== 1 ? 's' : ''} ago`;
    if (diffHours < 24) return `${diffHours} hour${diffHours !== 1 ? 's' : ''} ago`;
    if (diffDays < 7) return `${diffDays} day${diffDays !== 1 ? 's' : ''} ago`;
    if (diffWeeks < 4) return `${diffWeeks} week${diffWeeks !== 1 ? 's' : ''} ago`;
    if (diffMonths < 12) return `${diffMonths} month${diffMonths !== 1 ? 's' : ''} ago`;
    return `${diffYears} year${diffYears !== 1 ? 's' : ''} ago`;
}

// Helper function to get icon for audit action
function getActionIcon(action: string) {
    const iconProps = { size: 16, style: { flexShrink: 0 } };
    switch (action) {
        case 'LOGIN': return <LogIn {...iconProps} />;
        case 'LINK_WALLET': return <LinkIcon {...iconProps} />;
        case 'UNLINK_WALLET': return <Unlink {...iconProps} />;
        case 'VIEW_ROSTER': return <List {...iconProps} />;
        case 'VIEW_MEMBER': return <Eye {...iconProps} />;
        case 'ADMIN_GRANT': return <ShieldPlus {...iconProps} />;
        case 'ADMIN_REVOKE': return <ShieldMinus {...iconProps} />;
        case 'TRIBE_JOIN': return <UserPlus {...iconProps} />;
        case 'TRIBE_LEAVE': return <UserMinus {...iconProps} />;
        case 'NOTE_CREATE': return <FileText {...iconProps} />;
        case 'NOTE_EDIT': return <Edit2 {...iconProps} />;
        default: return <ShieldAlert {...iconProps} />;
    }
}

function RosterMemberPage() {
    const { id } = Route.useParams()
    const { user, token, isAuthenticated, currentTribe } = useAuth()
    const [auditPage, setAuditPage] = useState(1)
    const [expandedDetails, setExpandedDetails] = useState<Set<string>>(new Set())
    const [newNoteContent, setNewNoteContent] = useState('')
    const [editingNoteId, setEditingNoteId] = useState<string | null>(null)
    const [editNoteContent, setEditNoteContent] = useState('')
    const queryClient = useQueryClient()

    const { data: member, isLoading, error } = useQuery({
        queryKey: ['rosterMember', id, currentTribe, auditPage],
        queryFn: async () => {
            if (!token) throw new Error("No token");

            const params = new URLSearchParams();
            if (currentTribe) params.append('tribe', currentTribe);
            params.append('audit_page', auditPage.toString());
            params.append('audit_per_page', '10');

            const res = await fetch(`${API_URL}/api/roster/${id}?${params.toString()}`, {
                headers: { 'Authorization': `Bearer ${token}` }
            });

            if (!res.ok) {
                 if (res.status === 403) throw new Error("Access Denied: You do not have permission to view this member.");
                 if (res.status === 404) throw new Error("Member not found.");
                 if (res.status === 400) throw new Error("Please select a tribe from the dropdown above.");
                 throw new Error("Failed to fetch member details");
            }
            return res.json() as Promise<RosterMember>;
        },
        enabled: !!token && !!user?.isAdmin,
        retry: false,
        placeholderData: keepPreviousData,
    });

    // Notes query
    const { data: notes = [] } = useQuery({
        queryKey: ['notes', id, currentTribe],
        queryFn: async () => {
            if (!token || !currentTribe) return [];
            const params = new URLSearchParams();
            params.append('tribe', currentTribe);
            const res = await fetch(`${API_URL}/api/roster/${id}/notes?${params.toString()}`, {
                headers: { 'Authorization': `Bearer ${token}` }
            });
            if (!res.ok) return [];
            return res.json() as Promise<Note[]>;
        },
        enabled: !!token && !!user?.isAdmin && !!currentTribe
    });

    // Grant admin mutation
    const grantAdminMutation = useMutation({
        mutationFn: async (walletId: string) => {
            if (!token || !currentTribe) throw new Error("Missing token or tribe");
            const params = new URLSearchParams();
            params.append('tribe', currentTribe);
            const res = await fetch(`${API_URL}/api/roster/${id}/grant-admin?${params.toString()}`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ wallet_id: walletId })
            });
            if (!res.ok) throw new Error("Failed to grant admin");
            return res.json();
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['rosterMember', id] });
        }
    });

    // Create note mutation
    const createNoteMutation = useMutation({
        mutationFn: async (content: string) => {
            if (!token || !currentTribe) throw new Error("Missing token or tribe");
            const params = new URLSearchParams();
            params.append('tribe', currentTribe);
            const res = await fetch(`${API_URL}/api/roster/${id}/notes?${params.toString()}`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ content })
            });
            if (!res.ok) throw new Error("Failed to create note");
            return res.json();
        },
        onSuccess: () => {
            setNewNoteContent('');
            queryClient.invalidateQueries({ queryKey: ['notes', id] });
            queryClient.invalidateQueries({ queryKey: ['rosterMember', id] });
        }
    });

    // Edit note mutation
    const editNoteMutation = useMutation({
        mutationFn: async ({ noteId, content }: { noteId: string; content: string }) => {
            if (!token) throw new Error("Missing token");
            const res = await fetch(`${API_URL}/api/notes/${noteId}`, {
                method: 'PUT',
                headers: {
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ content })
            });
            if (!res.ok) throw new Error("Failed to edit note");
            return res.json();
        },
        onSuccess: () => {
            setEditingNoteId(null);
            setEditNoteContent('');
            queryClient.invalidateQueries({ queryKey: ['notes', id] });
            queryClient.invalidateQueries({ queryKey: ['rosterMember', id] });
        }
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
                <Link to="/home" className="btn btn-secondary">Back to Home</Link>
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
                                src={`https://cdn.discordapp.com/avatars/${member.discordId}/${member.avatar}.png?size=256`}
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
                                {member?.discordId}
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
                                <div key={wallet.id} style={{
                                    background: 'var(--bg-secondary)',
                                    padding: '1rem',
                                    borderRadius: 'var(--radius-sm)',
                                    display: 'flex',
                                    justifyContent: 'space-between',
                                    alignItems: 'center',
                                    border: '1px solid var(--glass-border)',
                                    gap: '1rem',
                                    opacity: wallet.deletedAt ? 0.5 : 1
                                }}>
                                    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', flex: 1, textDecoration: wallet.deletedAt ? 'line-through' : 'none' }}>
                                        <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', flexWrap: 'wrap' }}>
                                            <code style={{ fontSize: '0.9rem', wordBreak: 'break-all' }}>{wallet.address}</code>
                                            {wallet.tribes && wallet.tribes.length > 0 && (
                                                <span style={{ color: 'var(--brand-orange)', fontSize: '0.8rem', fontFamily: 'var(--font-heading)', letterSpacing: '1px' }}>
                                                    [{wallet.tribes.join(', ')}]
                                                </span>
                                            )}
                                        </div>
                                         {/* Grant Admin button - only show if active, if user is admin of current tribe and wallet doesn't already have this tribe */}
                                        {currentTribe && !wallet.deletedAt && user?.adminTribes?.includes(currentTribe) && !wallet.tribes.includes(currentTribe) && (
                                            <button
                                                onClick={() => grantAdminMutation.mutate(wallet.id)}
                                                disabled={grantAdminMutation.isPending}
                                                className="btn btn-primary"
                                                style={{ fontSize: '0.75rem', padding: '0.4rem 0.75rem', alignSelf: 'flex-start' }}
                                            >
                                                <ShieldPlus size={14} style={{ marginRight: '0.25rem' }} />
                                                {grantAdminMutation.isPending ? 'Granting...' : `Grant ${currentTribe} Admin`}
                                            </button>
                                        )}
                                    </div>
                                    <div style={{ display: 'flex', gap: '0.5rem' }}>
                                        <CopyButton text={wallet.address} />
                                        <a
                                            href={`https://suiscan.xyz/mainnet/account/${wallet.address}`}
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

                {/* Notes Panel - Full Width */}
                <div className="card" style={{ gridColumn: '1 / -1' }}>
                    <h3 style={{ marginTop: 0, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                        <FileText size={20} color="var(--accent-primary)" />
                        Notes
                    </h3>

                    {/* New Note Form */}
                    <div style={{ marginTop: '1.5rem', marginBottom: '2rem' }}>
                        <textarea
                            value={newNoteContent}
                            onChange={(e) => setNewNoteContent(e.target.value)}
                            placeholder="Add a note about this member..."
                            style={{
                                width: '100%',
                                minHeight: '100px',
                                padding: '0.75rem',
                                background: 'var(--bg-secondary)',
                                border: '1px solid var(--glass-border)',
                                color: 'var(--text-primary)',
                                fontFamily: 'var(--font-body)',
                                fontSize: '0.95rem',
                                resize: 'vertical'
                            }}
                        />
                        <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: '0.5rem' }}>
                            <button
                                onClick={() => createNoteMutation.mutate(newNoteContent)}
                                disabled={!newNoteContent.trim() || createNoteMutation.isPending}
                                className="btn btn-primary"
                                style={{ fontSize: '0.85rem' }}
                            >
                                <Save size={14} style={{ marginRight: '0.25rem' }} />
                                {createNoteMutation.isPending ? 'Adding...' : 'Add Note'}
                            </button>
                        </div>
                    </div>

                    {/* Notes List */}
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                        {notes.length === 0 ? (
                            <div style={{ color: 'var(--text-secondary)', fontStyle: 'italic', textAlign: 'center', padding: '2rem' }}>
                                No notes yet. Add one above to start tracking information about this member.
                            </div>
                        ) : (
                            notes.map((note) => (
                                <div key={note.id} style={{
                                    background: 'var(--bg-secondary)',
                                    padding: '1rem',
                                    border: '1px solid var(--glass-border)',
                                    borderRadius: 'var(--radius-sm)'
                                }}>
                                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '0.75rem' }}>
                                        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem' }}>
                                            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                                <span style={{ fontWeight: 'bold', fontSize: '0.9rem' }}>{note.authorUsername}</span>
                                                {note.authorDiscriminator !== '0' && (
                                                    <span style={{ color: 'var(--text-secondary)', fontSize: '0.75rem' }}>#{note.authorDiscriminator}</span>
                                                )}
                                            </div>
                                            <div style={{ fontSize: '0.8rem', color: 'var(--text-secondary)' }}>
                                                {formatDateTime(note.createdAt)}
                                                {note.updatedAt !== note.createdAt && ' (edited)'}
                                            </div>
                                        </div>
                                        {/* Only show edit button if current user is the author */}
                                        {user?.id === note.authorId && editingNoteId !== note.id && (
                                            <button
                                                onClick={() => {
                                                    setEditingNoteId(note.id);
                                                    setEditNoteContent(note.content);
                                                }}
                                                style={{
                                                    background: 'transparent',
                                                    border: 'none',
                                                    color: 'var(--text-secondary)',
                                                    cursor: 'pointer',
                                                    padding: '0.25rem'
                                                }}
                                                title="Edit Note"
                                            >
                                                <Edit2 size={14} />
                                            </button>
                                        )}
                                    </div>

                                    {editingNoteId === note.id ? (
                                        <div>
                                            <textarea
                                                value={editNoteContent}
                                                onChange={(e) => setEditNoteContent(e.target.value)}
                                                style={{
                                                    width: '100%',
                                                    minHeight: '80px',
                                                    padding: '0.75rem',
                                                    background: 'var(--bg-primary)',
                                                    border: '1px solid var(--glass-border)',
                                                    color: 'var(--text-primary)',
                                                    fontFamily: 'var(--font-body)',
                                                    fontSize: '0.95rem',
                                                    resize: 'vertical',
                                                    marginBottom: '0.5rem'
                                                }}
                                            />
                                            <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' }}>
                                                <button
                                                    onClick={() => {
                                                        setEditingNoteId(null);
                                                        setEditNoteContent('');
                                                    }}
                                                    className="btn btn-secondary"
                                                    style={{ fontSize: '0.75rem', padding: '0.4rem 0.75rem' }}
                                                >
                                                    <X size={12} style={{ marginRight: '0.25rem' }} />
                                                    Cancel
                                                </button>
                                                <button
                                                    onClick={() => editNoteMutation.mutate({ noteId: note.id, content: editNoteContent })}
                                                    disabled={!editNoteContent.trim() || editNoteMutation.isPending}
                                                    className="btn btn-primary"
                                                    style={{ fontSize: '0.75rem', padding: '0.4rem 0.75rem' }}
                                                >
                                                    <Save size={12} style={{ marginRight: '0.25rem' }} />
                                                    {editNoteMutation.isPending ? 'Saving...' : 'Save'}
                                                </button>
                                            </div>
                                        </div>
                                    ) : (
                                        <p style={{
                                            margin: 0,
                                            whiteSpace: 'pre-wrap',
                                            wordBreak: 'break-word',
                                            lineHeight: 1.6
                                        }}>
                                            {note.content}
                                        </p>
                                    )}
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
                                    <th style={{ padding: '0.75rem', color: 'var(--text-secondary)', width: '40px' }}></th>
                                    <th style={{ padding: '0.75rem', color: 'var(--text-secondary)' }}>Actor</th>
                                    <th style={{ padding: '0.75rem', color: 'var(--text-secondary)' }}>Action</th>
                                    <th style={{ padding: '0.75rem', color: 'var(--text-secondary)' }}>Details</th>
                                    <th style={{ padding: '0.75rem', color: 'var(--text-secondary)' }}>Date</th>
                                </tr>
                            </thead>
                            <tbody>
                                {member?.audits?.items?.map((audit) => {
                                    const isExpanded = expandedDetails.has(audit.id);
                                    const shouldTruncate = audit.details.length > 100;
                                    const displayDetails = shouldTruncate && !isExpanded
                                        ? audit.details.slice(0, 100) + '...'
                                        : audit.details;

                                    return (
                                        <tr key={audit.id} style={{ borderBottom: '1px solid rgba(255,255,255,0.05)' }}>
                                            <td style={{ padding: '0.75rem', color: 'var(--accent-primary)' }}>
                                                {getActionIcon(audit.action)}
                                            </td>
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
                                            <td
                                                style={{
                                                    padding: '0.75rem',
                                                    cursor: shouldTruncate ? 'pointer' : 'default',
                                                    userSelect: shouldTruncate ? 'none' : 'auto'
                                                }}
                                                onClick={() => {
                                                    if (shouldTruncate) {
                                                        setExpandedDetails(prev => {
                                                            const next = new Set(prev);
                                                            if (next.has(audit.id)) {
                                                                next.delete(audit.id);
                                                            } else {
                                                                next.add(audit.id);
                                                            }
                                                            return next;
                                                        });
                                                    }
                                                }}
                                            >
                                                {displayDetails}
                                                {shouldTruncate && (
                                                    <span style={{
                                                        color: 'var(--accent-primary)',
                                                        fontSize: '0.85rem',
                                                        marginLeft: '0.5rem',
                                                        fontStyle: 'italic'
                                                    }}>
                                                        {isExpanded ? '(click to collapse)' : '(click to expand)'}
                                                    </span>
                                                )}
                                            </td>
                                            <td style={{ padding: '0.75rem', color: 'var(--text-secondary)', fontSize: '0.9rem' }}>
                                                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem' }}>
                                                    <span>{formatDateTime(audit.createdAt)}</span>
                                                    <span style={{ fontSize: '0.8rem', color: 'var(--text-tertiary)' }}>
                                                        ({getRelativeTime(audit.createdAt)})
                                                    </span>
                                                </div>
                                            </td>
                                        </tr>
                                    );
                                }) || (
                                    <tr>
                                        <td colSpan={5} style={{ padding: '2rem', textAlign: 'center', color: 'var(--text-secondary)' }}>No audit history found.</td>
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
