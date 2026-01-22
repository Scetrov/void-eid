import { Link } from '@tanstack/react-router'
import { LogOut } from 'lucide-react'
import { ThemeToggle } from './ThemeToggle'
import { useAuth } from '../providers/AuthProvider'
import type { ReactNode } from 'react'

export function DashboardLayout({ children }: { children: ReactNode }) {
    const { user, logout } = useAuth()

    return (
        <div className="dashboard-container">
            <header className="dashboard-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '2rem' }}>
                 <nav style={{ display: 'flex', gap: '2rem', alignItems: 'center' }}>
                    <Link
                        to="/dashboard"
                        activeProps={{ style: { color: 'var(--text-primary)', fontWeight: 'bold' } }}
                        style={{ color: 'var(--text-secondary)', textDecoration: 'none', fontSize: '2rem', fontWeight: 700, fontFamily: "'Diskette Mono', monospace" }}
                    >
                        Dashboard
                    </Link>
                    {user?.isAdmin && (
                        <Link
                            to="/roster"
                            activeProps={{ style: { color: 'var(--text-primary)', fontWeight: 'bold' } }}
                            style={{ color: 'var(--text-secondary)', textDecoration: 'none', fontSize: '2rem', fontWeight: 700, fontFamily: "'Diskette Mono', monospace" }}
                        >
                            Tribe Roster
                        </Link>
                    )}
                </nav>
                 <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
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
