import { Link, useLocation } from '@tanstack/react-router'
import { LogOut, Home, Mic, Users } from 'lucide-react'
import { ThemeToggle } from './ThemeToggle'
import { AdminTribeNav } from './AdminTribeNav'
import { useAuth } from '../providers/AuthProvider'
import type { ReactNode } from 'react'

export function DashboardLayout({ children }: { children: ReactNode }) {
    const { user, logout } = useAuth()
    const location = useLocation()
    const isRosterPage = location.pathname.startsWith('/roster')

    return (
        <div className="dashboard-container">
            <header className="dashboard-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '3rem' }}>
                  <nav style={{ display: 'flex', gap: '2rem', alignItems: 'flex-start' }}>
                    <Link
                        to="/home"
                        activeProps={{ style: { color: 'var(--text-primary)', fontWeight: 'bold' } }}
                        className="nav-link"
                        style={{ color: 'var(--text-secondary)', textDecoration: 'none', fontSize: '2rem', fontWeight: 700, fontFamily: "'Diskette Mono', monospace", lineHeight: 1, display: 'flex', alignItems: 'center', gap: '0.75rem' }}
                    >
                        <Home size={28} />
                        Home
                    </Link>
                    <Link
                        to="/voice"
                        activeProps={{ style: { color: 'var(--text-primary)', fontWeight: 'bold' } }}
                        className="nav-link"
                        style={{ color: 'var(--text-secondary)', textDecoration: 'none', fontSize: '2rem', fontWeight: 700, fontFamily: "'Diskette Mono', monospace", lineHeight: 1, display: 'flex', alignItems: 'center', gap: '0.75rem' }}
                    >
                        <Mic size={28} />
                        Voice
                    </Link>
                    {(user?.adminTribes?.length ?? 0) > 0 && (
                        <div style={{ display: 'flex', flexDirection: 'column' }}>
                            <Link
                                to="/roster"
                                activeProps={{ style: { color: 'var(--text-primary)', fontWeight: 'bold' } }}
                                className="nav-link"
                                style={{ color: 'var(--text-secondary)', textDecoration: 'none', fontSize: '2rem', fontWeight: 700, fontFamily: "'Diskette Mono', monospace", lineHeight: 1, display: 'flex', alignItems: 'center', gap: '0.75rem' }}
                            >
                                <Users size={28} />
                                Roster
                            </Link>
                            {/* Secondary Nav for Multi-Tribe Admins */}
                            {isRosterPage && <AdminTribeNav />}
                        </div>
                    )}
                </nav>
                 <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', paddingTop: '0.25rem' }}>
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
