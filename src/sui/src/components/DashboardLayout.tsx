import { Link, useLocation } from '@tanstack/react-router'
import { LogOut } from 'lucide-react'
import { ThemeToggle } from './ThemeToggle'
import { TribeSelector } from './TribeSelector'
import { useAuth } from '../providers/AuthProvider'
import type { ReactNode } from 'react'

export function DashboardLayout({ children }: { children: ReactNode }) {
    const { user, logout, currentTribe, setCurrentTribe } = useAuth()
    const location = useLocation()

    // Check if we are on a roster-related page
    const isRosterPage = location.pathname.startsWith('/roster')
    const showSecondaryNav = user?.isAdmin && user.tribes && user.tribes.length > 1 && isRosterPage

    return (
        <div className="dashboard-container">
            <header className="dashboard-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '3rem' }}>
                 <nav style={{ display: 'flex', gap: '2rem' }}>
                    <Link
                        to="/dashboard"
                        activeProps={{ style: { color: 'var(--text-primary)', fontWeight: 'bold' } }}
                        style={{ color: 'var(--text-secondary)', textDecoration: 'none', fontSize: '2rem', fontWeight: 700, fontFamily: "'Diskette Mono', monospace", lineHeight: 1 }}
                    >
                        Dashboard
                    </Link>
                    {user?.isAdmin && (
                        <div style={{ display: 'flex', flexDirection: 'column' }}>
                            <Link
                                to="/roster"
                                activeProps={{ style: { color: 'var(--text-primary)', fontWeight: 'bold' } }}
                                style={{ color: 'var(--text-secondary)', textDecoration: 'none', fontSize: '2rem', fontWeight: 700, fontFamily: "'Diskette Mono', monospace", lineHeight: 1 }}
                            >
                                Tribe Roster
                            </Link>
                            {showSecondaryNav && (
                                <div className="secondary-nav" style={{ marginTop: '0.75rem' }}>
                                    {user.tribes.map((tribe) => (
                                        <button
                                            key={tribe}
                                            onClick={() => setCurrentTribe(tribe)}
                                            className={`secondary-nav-item ${currentTribe === tribe ? 'active' : ''}`}
                                        >
                                            {tribe}
                                        </button>
                                    ))}
                                </div>
                            )}
                        </div>
                    )}
                </nav>
                 <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', paddingTop: '0.25rem' }}>
                    {!showSecondaryNav && <TribeSelector />}
                    <ThemeToggle />
                    <button onClick={logout} className="btn btn-secondary" title="Logout">
                        <LogOut size={18} />
                    </button>
                 </div>
            </header>

            {children}
        </div>
    )
}
