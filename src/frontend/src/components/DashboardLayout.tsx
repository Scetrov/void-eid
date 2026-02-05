import { Link, useLocation } from '@tanstack/react-router'
import { LogOut, Home, Mic, Users } from 'lucide-react'
import { ThemeToggle } from './ThemeToggle'
import { AdminTribeNav } from './AdminTribeNav'
import { useAuth } from '../providers/AuthProvider'
import { CipherNavText, type CipherNavTextHandle } from './CipherNavText'
import { useRef, type ReactNode } from 'react'

export function DashboardLayout({ children }: { children: ReactNode }) {
    const { user, logout } = useAuth()
    const location = useLocation()
    const isRosterPage = location.pathname.startsWith('/roster')

    const homeRef = useRef<CipherNavTextHandle>(null)
    const voiceRef = useRef<CipherNavTextHandle>(null)
    const rosterRef = useRef<CipherNavTextHandle>(null)

    return (
        <div className="dashboard-container">
            <header className="dashboard-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '3rem' }}>
                  <nav style={{ display: 'flex', gap: '2rem', alignItems: 'flex-start' }}>
                    <Link
                        to="/home"
                        activeProps={{ style: { color: 'var(--text-primary)', fontWeight: 'bold' } }}
                        className="nav-link"
                        style={{ color: 'var(--text-secondary)', textDecoration: 'none', fontSize: '2rem', fontWeight: 700, fontFamily: "'Diskette Mono', monospace", lineHeight: 1, display: 'flex', alignItems: 'center', gap: '0.75rem' }}
                        onMouseEnter={() => homeRef.current?.trigger()}
                    >
                        <Home size={28} />
                        <CipherNavText ref={homeRef} text="Home" scrambleDuration={500} scrambleSpeed={75} />
                    </Link>
                    <Link
                        to="/voice"
                        activeProps={{ style: { color: 'var(--text-primary)', fontWeight: 'bold' } }}
                        className="nav-link"
                        style={{ color: 'var(--text-secondary)', textDecoration: 'none', fontSize: '2rem', fontWeight: 700, fontFamily: "'Diskette Mono', monospace", lineHeight: 1, display: 'flex', alignItems: 'center', gap: '0.75rem' }}
                        onMouseEnter={() => voiceRef.current?.trigger()}
                    >
                        <Mic size={28} />
                        <CipherNavText ref={voiceRef} text="Voice" scrambleDuration={500} scrambleSpeed={75} />
                    </Link>
                    {(user?.adminTribes?.length ?? 0) > 0 && (
                        <div style={{ display: 'flex', flexDirection: 'column' }}>
                            <Link
                                to="/roster"
                                activeProps={{ style: { color: 'var(--text-primary)', fontWeight: 'bold' } }}
                                className="nav-link"
                                style={{ color: 'var(--text-secondary)', textDecoration: 'none', fontSize: '2rem', fontWeight: 700, fontFamily: "'Diskette Mono', monospace", lineHeight: 1, display: 'flex', alignItems: 'center', gap: '0.75rem' }}
                                onMouseEnter={() => rosterRef.current?.trigger()}
                            >
                                <Users size={28} />
                                <CipherNavText ref={rosterRef} text="Roster" scrambleDuration={500} scrambleSpeed={75} />
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
