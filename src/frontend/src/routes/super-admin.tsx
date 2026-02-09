import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { DashboardLayout } from '../components/DashboardLayout'
import { useAuth, type User } from '../providers/AuthProvider'
import { useEffect, useState, useCallback } from 'react'
import { API_URL } from '../config'
import { ShieldAlert, Trash2, Edit2, Plus, ChevronDown, ChevronRight, UserPlus, AlertTriangle, CheckCircle, X, Check } from 'lucide-react'
import { CopyButton } from '../components/CopyButton'

export const Route = createFileRoute('/super-admin')({
    component: SuperAdminDashboard,
})

// --- Components ---

const WalletAddress = ({ address }: { address: string }) => {
    return (
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontFamily: '"Diskette Mono", monospace' }}>
            <span title={address}>{address.slice(0, 6)}...{address.slice(-4)}</span>
            <CopyButton text={address} />
        </div>
    );
};

const Modal = ({ children, title, onClose }: { children: React.ReactNode, title: string, onClose: () => void }) => (
    <div style={{
        position: 'fixed', top: 0, left: 0, right: 0, bottom: 0,
        backgroundColor: 'rgba(0,0,0,0.5)',
        backdropFilter: 'blur(4px)',
        display: 'flex', justifyContent: 'center', alignItems: 'center', zIndex: 1000
    }}>
        <div className="card" style={{
            width: '600px',
            maxWidth: '95vw',
            padding: '2rem',
            maxHeight: '90vh',
            overflowY: 'auto',
            scrollbarWidth: 'none', // Firefox
            msOverflowStyle: 'none'  // IE/Edge
        }}>
            <style>{`
                .card::-webkit-scrollbar { display: none; }
            `}</style>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.5rem', borderBottom: '1px solid var(--border-color)', paddingBottom: '0.5rem' }}>
                <h3 style={{ margin: 0, fontFamily: '"Diskette Mono", monospace' }}>{title}</h3>
                <button onClick={onClose} style={{ background: 'none', border: 'none', cursor: 'pointer', color: 'var(--text-secondary)' }}>
                    <div style={{ fontSize: '1.2rem' }}>X</div>
                </button>
            </div>
            {children}
        </div>
    </div>
);

const Notification = ({ message, type, onClose }: { message: string, type: 'error' | 'success', onClose: () => void }) => (
     <div className={`notification-banner notification-banner-${type}`}>
         <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
             {type === 'error' ? <AlertTriangle size={20} color="#ef4444" /> : <CheckCircle size={20} color="#22c55e" />}
             <span>{message}</span>
         </div>
         <button onClick={onClose} style={{ background: 'none', border: 'none', cursor: 'pointer', color: 'var(--text-secondary)' }}>X</button>
     </div>
);

function SuperAdminDashboard() {
    const { user, token, isLoading: authLoading } = useAuth()
    const navigate = useNavigate()

    // Accordion State
    const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({
        users: true,
        tribes: false,
        wallets: false
    });

    const toggleSection = (section: string) => {
        setExpandedSections(prev => ({
            ...prev,
            [section]: !prev[section]
        }));
    };

    const [isLoading, setIsLoading] = useState(false)
    const [globalError, setGlobalError] = useState<string | null>(null)
    const [globalSuccess, setGlobalSuccess] = useState<string | null>(null)

    // Data
    const [users, setUsers] = useState<User[]>([])
    const [tribes, setTribes] = useState<string[]>([])

    // Search filters
    const [userSearch, setUserSearch] = useState('')
    const [tribeSearch, setTribeSearch] = useState('')
    const [walletSearch, setWalletSearch] = useState('')

    // Modals/Forms state

    // Edit User Modal
    const [editingUser, setEditingUser] = useState<User | null>(null)
    const [editUserError, setEditUserError] = useState<string | null>(null)

    // Create Tribe Modal
    const [isCreateTribeOpen, setIsCreateTribeOpen] = useState(false)
    const [newTribeName, setNewTribeName] = useState('')
    const [createTribeError, setCreateTribeError] = useState<string | null>(null)

    // Rename Tribe Modal
    const [renamingTribe, setRenamingTribe] = useState<string | null>(null)
    const [renameTribeInput, setRenameTribeInput] = useState('')
    const [renameTribeError, setRenameTribeError] = useState<string | null>(null)

    // Add User to Tribe Modal
    const [addingUserToTribe, setAddingUserToTribe] = useState<string | null>(null) // Tribe name
    const [addUserToTribeUsername, setAddUserToTribeUsername] = useState('')
    const [addUserToTribeError, setAddUserToTribeError] = useState<string | null>(null)


    useEffect(() => {
        if (!authLoading && (!user || !user.isSuperAdmin)) {
            navigate({ to: '/home' })
        }
    }, [user, authLoading, navigate])

    const fetchData = useCallback(async () => {
        if (!token) return;
        setIsLoading(true);
        setGlobalError(null);
        try {
            // Fetch Users (includes wallets)
            const resUsers = await fetch(`${API_URL}/api/admin/users`, {
                headers: { 'Authorization': `Bearer ${token}` }
            });
            if (!resUsers.ok) throw new Error("Failed to fetch users");
            const dataUsers = await resUsers.json();
            setUsers(dataUsers);

            // Fetch Tribes
            const resTribes = await fetch(`${API_URL}/api/admin/tribes`, {
                headers: { 'Authorization': `Bearer ${token}` }
            });
            if (!resTribes.ok) throw new Error("Failed to fetch tribes");
            const dataTribes = await resTribes.json();
            setTribes(dataTribes);

        } catch (e: unknown) {
            console.error(e);
            setGlobalError("Failed to load data");
        } finally {
            setIsLoading(false);
        }
    }, [token]);

    useEffect(() => {
        if (user?.isSuperAdmin && token) {
            void fetchData()
        }
    }, [user, token, fetchData])

    // --- Actions ---

    const handleUpdateUser = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!editingUser || !token) return;
        setEditUserError(null);
        setIsLoading(true);
        try {
            const res = await fetch(`${API_URL}/api/admin/users/${editingUser.id}`, {
                method: 'PATCH',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                },
                body: JSON.stringify({
                    is_admin: editingUser.isAdmin,
                    username: editingUser.username,
                    discriminator: editingUser.discriminator,
                    admin_tribes: editingUser.adminTribes
                })
            });
            if (!res.ok) throw new Error("Failed to update user");

            setGlobalSuccess("User updated successfully");
            setEditingUser(null);
            fetchData();
        } catch (e: unknown) {
            if (e instanceof Error) setEditUserError(e.message);
            else setEditUserError("An unknown error occurred");
        } finally {
            setIsLoading(false);
        }
    }

    const handleCreateTribe = async () => {
        if (!newTribeName.trim() || !token) return;
        setCreateTribeError(null);
        setIsLoading(true);
        try {
            const res = await fetch(`${API_URL}/api/admin/tribes`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                },
                body: JSON.stringify({ name: newTribeName })
            });
            if (!res.ok) throw new Error("Failed to create tribe");
            setGlobalSuccess("Tribe created");
            setNewTribeName('');
            setIsCreateTribeOpen(false);
            fetchData();
        } catch (e: unknown) {
            if (e instanceof Error) setCreateTribeError(e.message);
            else setCreateTribeError("An unknown error occurred");
        } finally {
            setIsLoading(false);
        }
    }

    const handleRenameTribe = async () => {
        if (!renamingTribe || !renameTribeInput.trim() || !token) return;
        setRenameTribeError(null);
        setIsLoading(true);
        try {
            const res = await fetch(`${API_URL}/api/admin/tribes/${encodeURIComponent(renamingTribe)}`, {
                method: 'PATCH',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                },
                body: JSON.stringify({ name: renameTribeInput })
            });
            if (!res.ok) throw new Error("Failed to rename tribe");
            setGlobalSuccess("Tribe renamed");
            setRenamingTribe(null);
            setRenameTribeInput('');
            fetchData();
        } catch (e: unknown) {
            if (e instanceof Error) setRenameTribeError(e.message);
            else setRenameTribeError("An unknown error occurred");
        } finally {
            setIsLoading(false);
        }
    }

    const handleAddUserToTribe = async () => {
        if (!addingUserToTribe || !addUserToTribeUsername.trim() || !token) return;
        setAddUserToTribeError(null);
        setIsLoading(true);
        try {
            const res = await fetch(`${API_URL}/api/admin/tribes/${encodeURIComponent(addingUserToTribe)}/users`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                },
                body: JSON.stringify({ username: addUserToTribeUsername })
            });
            if (!res.ok) {
                const txt = await res.text();
                throw new Error(txt || "Failed to add user to tribe");
            }
            setGlobalSuccess(`User added to ${addingUserToTribe}`);
            setAddingUserToTribe(null);
            setAddUserToTribeUsername('');
            fetchData();
        } catch (e: unknown) {
            if (e instanceof Error) setAddUserToTribeError(e.message);
            else setAddUserToTribeError("An unknown error occurred");
        } finally {
            setIsLoading(false);
        }
    }

    const handleDeleteWallet = async (walletId: string, setLocalError?: (err: string) => void) => {
        if (!confirm("Are you sure you want to FORCE DELETE this wallet? This action is audited.") || !token) return;
        console.log("Attempting to delete wallet:", walletId);
        setIsLoading(true);
        if (setLocalError) setLocalError("");
        else setGlobalError(null);

        try {
            const res = await fetch(`${API_URL}/api/admin/wallets/${walletId}`, {
                method: 'DELETE',
                headers: { 'Authorization': `Bearer ${token}` }
            });
            if (!res.ok) {
                 const txt = await res.text();
                 throw new Error(txt || "Failed to delete wallet");
            }
            setGlobalSuccess("Wallet force unlinked");
            // If editing user, remove wallet from local state to reflect immediately or just re-fetch
            if (editingUser) {
                 setEditingUser({
                     ...editingUser,
                     wallets: editingUser.wallets.filter(w => w.id !== walletId)
                 });
            }
            fetchData();
        } catch (e: unknown) {
             const msg = e instanceof Error ? e.message : "An unknown error occurred";
             console.error("Delete Wallet Error:", e);
             if (setLocalError) setLocalError(msg);
             else setGlobalError(msg);
        } finally {
            setIsLoading(false);
        }
    }

    // --- Render Helpers ---

    const filteredUsers = users.filter(u =>
        u.username.toLowerCase().includes(userSearch.toLowerCase()) ||
        u.discordId.includes(userSearch)
    );

    const filteredTribes = tribes.filter(t =>
        t.toLowerCase().includes(tribeSearch.toLowerCase())
    );

    // Derived filtered wallets
    const allWallets = users.flatMap(u => u.wallets.map(w => ({ ...w, username: u.username, userId: u.id })));
    const filteredWallets = allWallets.filter(w =>
        (w.address.toLowerCase().includes(walletSearch.toLowerCase()) ||
        w.username.toLowerCase().includes(walletSearch.toLowerCase()))
    );


    if (authLoading) return null;

    return (
        <DashboardLayout>
            <div className="home-container" style={{ padding: '2rem', maxWidth: '1200px', margin: '0 auto' }}>
                {/* Header - Removed border */}
                <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginBottom: '2rem' }}>
                    <ShieldAlert size={32} color="var(--brand-orange)" />
                    <h1 style={{ margin: 0, fontFamily: '"Diskette Mono", monospace' }}>Super Admin Dashboard</h1>
                </div>

                {/* Global Notifications */}
                {globalError && <Notification message={globalError} type="error" onClose={() => setGlobalError(null)} />}
                {globalSuccess && <Notification message={globalSuccess} type="success" onClose={() => setGlobalSuccess(null)} />}

                {/* Accordion Sections */}

                {/* USERS SECTION */}
                <div className="card" style={{ marginBottom: '1rem', padding: '0', overflow: 'hidden' }}>
                    <div
                        onClick={() => toggleSection('users')}
                        style={{ padding: '1rem', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontWeight: 'bold', backgroundColor: 'var(--bg-secondary)' }}
                    >
                        <span>Users Management</span>
                        {expandedSections['users'] ? <ChevronDown /> : <ChevronRight />}
                    </div>
                    {expandedSections['users'] && (
                        <div style={{ padding: '1rem', borderTop: '1px solid var(--border-color)' }}>
                            <div style={{ marginBottom: '1rem', display: 'flex', justifyContent: 'space-between' }}>
                                <input
                                    type="text"
                                    placeholder="Search users..."
                                    value={userSearch}
                                    onChange={e => setUserSearch(e.target.value)}
                                    style={{ padding: '0.5rem', width: '300px', backgroundColor: 'var(--bg-primary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)', borderRadius: '0' }}
                                />
                                <button className="btn btn-secondary" onClick={() => fetchData()}>Refresh</button>
                            </div>
                            <div style={{ overflowX: 'auto' }}>
                                 <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
                                    <thead>
                                        <tr style={{ borderBottom: '1px solid var(--border-color)' }}>
                                            <th style={{ padding: '0.5rem' }}>ID</th>
                                            <th style={{ padding: '0.5rem' }}>Username</th>
                                            <th style={{ padding: '0.5rem' }}>Discord ID</th>
                                            <th style={{ padding: '0.5rem' }}>Global Admin</th>
                                            <th style={{ padding: '0.5rem' }}>Tribes</th>
                                            <th style={{ padding: '0.5rem' }}>Actions</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {filteredUsers.map(u => (
                                            <tr key={u.id} style={{ borderBottom: '1px solid var(--border-color)' }}>
                                                <td style={{ padding: '0.5rem' }}>{u.id}</td>
                                                <td style={{ padding: '0.5rem' }}>{u.username}#{u.discriminator}</td>
                                                <td style={{ padding: '0.5rem' }}>{u.discordId}</td>
                                                <td style={{ padding: '0.5rem' }}>
                                                    {u.isAdmin ? <Check size={20} color="#22c55e" /> : <X size={20} color="#ef4444" />}
                                                </td>
                                                <td style={{ padding: '0.5rem' }}>
                                                    {u.tribes.join(', ')}
                                                </td>
                                                <td style={{ padding: '0.5rem' }}>
                                                    <button className="btn btn-sm btn-secondary" onClick={() => setEditingUser(u)}>
                                                        <Edit2 size={16} />
                                                    </button>
                                                </td>
                                            </tr>
                                        ))}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    )}
                </div>

                {/* TRIBES SECTION */}
                <div className="card" style={{ marginBottom: '1rem', padding: '0', overflow: 'hidden' }}>
                     <div
                        onClick={() => toggleSection('tribes')}
                        style={{ padding: '1rem', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontWeight: 'bold', backgroundColor: 'var(--bg-secondary)' }}
                    >
                        <span>Tribes Management</span>
                        {expandedSections['tribes'] ? <ChevronDown /> : <ChevronRight />}
                    </div>
                    {expandedSections['tribes'] && (
                        <div style={{ padding: '1rem', borderTop: '1px solid var(--border-color)' }}>
                             <div style={{ marginBottom: '1rem', display: 'flex', justifyContent: 'space-between' }}>
                                <input
                                    type="text"
                                    placeholder="Search tribes..."
                                    value={tribeSearch}
                                    onChange={e => setTribeSearch(e.target.value)}
                                    style={{ padding: '0.5rem', width: '300px', backgroundColor: 'var(--bg-primary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)', borderRadius: '0' }}
                                />
                                <div style={{display:'flex', gap:'0.5rem'}}>
                                     <button className="btn btn-primary" onClick={() => setIsCreateTribeOpen(true)}>
                                        <Plus size={16} style={{marginRight: '0.5rem'}} /> Create Tribe
                                    </button>
                                </div>
                            </div>
                            <div>
                                 {filteredTribes.map(t => (
                                     <div key={t} className="card" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.5rem', padding: '1rem', backgroundColor: 'var(--card-bg)' }}>
                                         <span style={{ fontWeight: 'bold' }}>{t}</span>
                                         <div style={{ display: 'flex', gap: '0.5rem' }}>
                                             <button className="btn btn-sm btn-secondary" onClick={() => {
                                                 setAddingUserToTribe(t);
                                                 setAddUserToTribeUsername('');
                                             }} title="Add User">
                                                 <UserPlus size={16} />
                                             </button>
                                             <button className="btn btn-sm btn-secondary" onClick={() => {
                                                 setRenamingTribe(t);
                                                 setRenameTribeInput(t);
                                             }} title="Rename">
                                                 <Edit2 size={16} />
                                             </button>
                                         </div>
                                     </div>
                                 ))}
                            </div>
                        </div>
                    )}
                </div>

                {/* WALLETS SECTION */}
                 <div className="card" style={{ marginBottom: '1rem', padding: '0', overflow: 'hidden' }}>
                     <div
                        onClick={() => toggleSection('wallets')}
                        style={{ padding: '1rem', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontWeight: 'bold', backgroundColor: 'var(--bg-secondary)' }}
                    >
                        <span>Wallets Management</span>
                        {expandedSections['wallets'] ? <ChevronDown /> : <ChevronRight />}
                    </div>
                    {expandedSections['wallets'] && (
                        <div style={{ padding: '1rem', borderTop: '1px solid var(--border-color)' }}>
                            <div style={{ marginBottom: '1rem', display: 'flex', justifyContent: 'space-between' }}>
                                <input
                                    type="text"
                                    placeholder="Search wallets..."
                                    value={walletSearch}
                                    onChange={e => setWalletSearch(e.target.value)}
                                    style={{ padding: '0.5rem', width: '300px', backgroundColor: 'var(--bg-primary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)', borderRadius: '0' }}
                                />
                            </div>
                             <div style={{ overflowX: 'auto' }}>
                                 <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
                                    <thead>
                                        <tr style={{ borderBottom: '1px solid var(--border-color)' }}>
                                            <th style={{ padding: '0.5rem' }}>Address</th>
                                            <th style={{ padding: '0.5rem' }}>User</th>
                                            <th style={{ padding: '0.5rem' }}>Verified At</th>
                                            <th style={{ padding: '0.5rem' }}>Actions</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {filteredWallets.map(w => (
                                            <tr key={w.id} style={{ borderBottom: '1px solid var(--border-color)', opacity: w.deletedAt ? 0.5 : 1 }}>
                                                <td style={{ padding: '0.5rem', textDecoration: w.deletedAt ? 'line-through' : 'none' }}>
                                                    <WalletAddress address={w.address} />
                                                </td>
                                                <td style={{ padding: '0.5rem' }}>{w.username}</td>
                                                <td style={{ padding: '0.5rem' }}>
                                                    {w.verifiedAt ? new Date(w.verifiedAt).toLocaleString() : '-'}
                                                </td>
                                                <td style={{ padding: '0.5rem' }}>
                                                    <button
                                                        className="btn btn-sm"
                                                        style={{ color: 'var(--text-primary)', border: '1px solid var(--text-primary)', borderRadius: '0' }}
                                                        onClick={() => handleDeleteWallet(w.id)}
                                                        title="Force Unlink"
                                                    >
                                                        <Trash2 size={16} />
                                                    </button>
                                                </td>
                                            </tr>
                                        ))}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    )}
                 </div>

                {/* --- Modals --- */}

                {/* Datalists for Autocomplete */}
                <datalist id="users-datalist">
                    {users.map(u => <option key={u.id} value={u.username} />)}
                </datalist>
                <datalist id="tribes-datalist">
                    {tribes.map(t => <option key={t} value={t} />)}
                </datalist>


                {editingUser && (
                    <Modal title={`Edit User: ${editingUser.username}`} onClose={() => setEditingUser(null)}>
                        {editUserError && <Notification message={editUserError} type="error" onClose={() => setEditUserError(null)} />}
                        <form onSubmit={handleUpdateUser}>
                            <div style={{ marginBottom: '1rem' }}>
                                <h4>Permissions</h4>
                                <div style={{ marginTop: '0.5rem' }}>
                                    <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer' }}>
                                        <input
                                            type="checkbox"
                                            checked={editingUser.isAdmin}
                                            onChange={e => setEditingUser({...editingUser, isAdmin: e.target.checked})}
                                            style={{ width: '20px', height: '20px', borderRadius: '0' }}
                                        />
                                        <span style={{ fontSize: '1.1rem' }}>Is Global Admin</span>
                                    </label>
                                </div>
                            </div>
                            <div style={{ marginBottom: '1rem' }}>
                                <h4>Tribe Administration</h4>
                                <div style={{ marginTop: '0.5rem', display: 'flex', flexDirection: 'column', gap: '0.5rem', maxHeight: '200px', overflowY: 'auto', border: '1px solid var(--border-color)', padding: '0.5rem' }}>
                                    {editingUser.tribes.length === 0 ? (
                                        <p style={{ color: 'var(--text-secondary)', fontStyle: 'italic' }}>User is not in any tribes.</p>
                                    ) : (
                                        editingUser.tribes.map(tribe => (
                                            <label key={tribe} style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer' }}>
                                                <input
                                                    type="checkbox"
                                                    checked={editingUser.adminTribes.includes(tribe)}
                                                    onChange={e => {
                                                        const newAdminTribes = e.target.checked
                                                            ? [...editingUser.adminTribes, tribe]
                                                            : editingUser.adminTribes.filter(t => t !== tribe);
                                                        setEditingUser({ ...editingUser, adminTribes: newAdminTribes });
                                                    }}
                                                    style={{ width: '18px', height: '18px', borderRadius: '0' }}
                                                />
                                                <span>Admin of <strong>{tribe}</strong></span>
                                            </label>
                                        ))
                                    )}
                                </div>
                            </div>
                            <div style={{ marginBottom: '1rem' }}>
                                <h4>Attached Wallets</h4>
                                {editingUser.wallets.length === 0 ? <p style={{color: 'var(--text-secondary)'}}>No wallets attached</p> : (
                                    <ul style={{ listStyle: 'none', padding: 0 }}>
                                        {editingUser.wallets.map(w => (
                                            <li key={w.id} style={{ display: 'flex', justifyContent: 'space-between', padding: '0.5rem', borderBottom: '1px solid var(--border-color)', opacity: w.deletedAt ? 0.5 : 1 }}>
                                                <div style={{ overflow: 'hidden', textOverflow: 'ellipsis', textDecoration: w.deletedAt ? 'line-through' : 'none' }}>
                                                    <WalletAddress address={w.address} />
                                                     <div style={{ fontSize: '0.8rem', color: 'var(--text-secondary)' }}>{new Date(w.verifiedAt).toLocaleString()}</div>
                                                 </div>
                                                 <button
                                                    type="button"
                                                    className="btn btn-sm"
                                                    style={{ color: 'var(--text-primary)', border: '1px solid var(--text-primary)', borderRadius: '0' }}
                                                    onClick={() => handleDeleteWallet(w.id, setEditUserError)}
                                                 >
                                                     <Trash2 size={14} />
                                                 </button>
                                            </li>
                                        ))}
                                    </ul>
                                )}
                            </div>
                            <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end', marginTop: '2rem' }}>
                                <button type="button" className="btn btn-secondary" onClick={() => setEditingUser(null)}>Cancel</button>
                                <button type="submit" className="btn btn-primary" disabled={isLoading}>Save Changes</button>
                            </div>
                        </form>
                    </Modal>
                )}

                {isCreateTribeOpen && (
                    <Modal title="Create New Tribe" onClose={() => setIsCreateTribeOpen(false)}>
                        {createTribeError && <Notification message={createTribeError} type="error" onClose={() => setCreateTribeError(null)} />}
                        <div style={{ marginBottom: '1rem' }}>
                            <label>Tribe Name</label>
                            <input
                                type="text"
                                value={newTribeName}
                                onChange={e => setNewTribeName(e.target.value)}
                                style={{ width: '100%', padding: '0.5rem', marginTop: '0.5rem', backgroundColor: 'var(--bg-primary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)', borderRadius: '0' }}
                            />
                        </div>
                        <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end' }}>
                            <button className="btn btn-secondary" onClick={() => setIsCreateTribeOpen(false)}>Cancel</button>
                            <button className="btn btn-primary" onClick={handleCreateTribe} disabled={isLoading}>Create</button>
                        </div>
                    </Modal>
                )}

                 {renamingTribe && (
                    <Modal title={`Rename Tribe: ${renamingTribe}`} onClose={() => setRenamingTribe(null)}>
                        {renameTribeError && <Notification message={renameTribeError} type="error" onClose={() => setRenameTribeError(null)} />}
                        <div style={{ marginBottom: '1rem' }}>
                            <label>New Name</label>
                            <input
                                type="text"
                                value={renameTribeInput}
                                onChange={e => setRenameTribeInput(e.target.value)}
                                style={{ width: '100%', padding: '0.5rem', marginTop: '0.5rem', backgroundColor: 'var(--bg-primary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)', borderRadius: '0' }}
                            />
                        </div>
                        <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end' }}>
                            <button className="btn btn-secondary" onClick={() => setRenamingTribe(null)}>Cancel</button>
                            <button className="btn btn-primary" onClick={handleRenameTribe} disabled={isLoading}>Rename</button>
                        </div>
                    </Modal>
                )}

                {addingUserToTribe && (
                    <Modal title={`Add User to ${addingUserToTribe}`} onClose={() => setAddingUserToTribe(null)}>
                         {addUserToTribeError && <Notification message={addUserToTribeError} type="error" onClose={() => setAddUserToTribeError(null)} />}
                         <div style={{ marginBottom: '1rem' }}>
                             <label>Username</label>
                             <input
                                 type="text"
                                 list="users-datalist"
                                 value={addUserToTribeUsername}
                                 onChange={e => setAddUserToTribeUsername(e.target.value)}
                                 placeholder="Enter username"
                                 style={{ width: '100%', padding: '0.5rem', marginTop: '0.5rem', backgroundColor: 'var(--bg-primary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)', borderRadius: '0' }}
                                 autoComplete="off"
                             />
                         </div>
                         <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end' }}>
                             <button className="btn btn-secondary" onClick={() => setAddingUserToTribe(null)}>Cancel</button>
                             <button className="btn btn-primary" onClick={handleAddUserToTribe} disabled={isLoading}>Add</button>
                         </div>
                     </Modal>
                )}
            </div>
        </DashboardLayout>
    )
}
