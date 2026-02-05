import { Link, useLocation } from '@tanstack/react-router'
import { LogOut } from 'lucide-react'
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
                 <nav style={{ display: 'flex', gap: '2rem' }}>
                    <Link
                        to="/dashboard"
                        activeProps={{ style: { color: 'var(--text-primary)', fontWeight: 'bold' } }}
                        style={{ color: 'var(--text-secondary)', textDecoration: 'none', fontSize: '2rem', fontWeight: 700, fontFamily: "'Diskette Mono', monospace", lineHeight: 1 }}
                    >
                        Dashboard
                    </Link>
                    <Link
                        to="/mumble"
                        activeProps={{ style: { color: 'var(--text-primary)', fontWeight: 'bold' } }}
                        style={{ color: 'var(--text-secondary)', textDecoration: 'none', fontSize: '2rem', fontWeight: 700, fontFamily: "'Diskette Mono', monospace", lineHeight: 1 }}
                    >
                        Voice
                    </Link>
                    {(user?.adminTribes?.length ?? 0) > 0 && (
                        <div style={{ display: 'flex', flexDirection: 'column' }}>
                            <Link
                                to="/roster"
                                activeProps={{ style: { color: 'var(--text-primary)', fontWeight: 'bold' } }}
                                style={{ color: 'var(--text-secondary)', textDecoration: 'none', fontSize: '2rem', fontWeight: 700, fontFamily: "'Diskette Mono', monospace", lineHeight: 1 }}
                            >
                                Tribe Roster
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
