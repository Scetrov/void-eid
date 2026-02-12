import { Link, useLocation } from '@tanstack/react-router'
import { LogOut, Home, Mic, Users, ShieldAlert } from 'lucide-react'
import { ThemeToggle } from './ThemeToggle'
import { AdminTribeNav } from './AdminTribeNav'
import { useAuth } from '../providers/AuthProvider'
import { CipherNavText, type CipherNavTextHandle } from './CipherNavText'
import { useRef, type ReactNode, useState, useEffect } from 'react'

export function DashboardLayout({ children }: { children: ReactNode }) {
    const { user, logout } = useAuth()
    const location = useLocation()
    const isRosterPage = location.pathname.startsWith('/roster')

    const homeRef = useRef<CipherNavTextHandle>(null)
    const voiceRef = useRef<CipherNavTextHandle>(null)
    const rosterRef = useRef<CipherNavTextHandle>(null)

    const mobileHomeRef = useRef<CipherNavTextHandle>(null)
    const mobileVoiceRef = useRef<CipherNavTextHandle>(null)
    const mobileRosterRef = useRef<CipherNavTextHandle>(null)
    const mobileSuperAdminRef = useRef<CipherNavTextHandle>(null)

    const [isMenuOpen, setIsMenuOpen] = useState(false)

    // Close menu when location changes
    useEffect(() => {
        // Use requestAnimationFrame to avoid "setState synchronously in effect" lint error
        // and ensure the menu closes after the navigation render is committed.
        const frame = requestAnimationFrame(() => setIsMenuOpen(false))
        return () => cancelAnimationFrame(frame)
    }, [location.pathname])

    // Staggered animation for mobile menu
    useEffect(() => {
        if (isMenuOpen) {
            const timeout1 = setTimeout(() => mobileHomeRef.current?.trigger(), 100)
            const timeout2 = setTimeout(() => mobileVoiceRef.current?.trigger(), 200)
            const timeout3 = setTimeout(() => mobileRosterRef.current?.trigger(), 300)
            const timeout4 = setTimeout(() => mobileSuperAdminRef.current?.trigger(), 400)

            return () => {
                clearTimeout(timeout1)
                clearTimeout(timeout2)
                clearTimeout(timeout3)
                clearTimeout(timeout4)
            }
        }
    }, [isMenuOpen])

    return (
        <div className="dashboard-container">
            <header className="dashboard-header">
                {/* Mobile Menu Trigger - Visible only on mobile via CSS */}
                <div className="mobile-menu-trigger">
                    <button
                        className="frontier-burger-btn"
                        onClick={() => setIsMenuOpen(true)}
                        aria-label="Open Menu"
                    >
                        [ :: MENU :: ]
                    </button>
                </div>

                {/* Desktop Navigation - Hidden on mobile via CSS */}
                <nav className="desktop-nav" style={{ display: 'flex', gap: '2rem', alignItems: 'flex-start' }}>
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

                {/* Desktop Actions - Hidden on mobile via CSS, moved to overlay */}
                    <div className="desktop-actions" style={{ display: 'flex', alignItems: 'center', gap: '1rem', paddingTop: '0.25rem' }}>
                    {user?.isSuperAdmin && (
                        <Link
                            to="/super-admin"
                            className="btn btn-secondary"
                            style={{ color: 'var(--accent-primary)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}
                            title="Super Admin Dashboard"
                        >
                            <ShieldAlert size={18} />
                        </Link>
                    )}
                    <ThemeToggle />
                    <button onClick={logout} className="btn btn-secondary" title="Logout">
                        <LogOut size={18} />
                    </button>
                </div>
            </header>

            {/* Mobile Menu Overlay */}
            {isMenuOpen && (
                <div className="mobile-menu-overlay">
                    <div className="mobile-menu-header">
                        <button
                            className="frontier-close-btn"
                            onClick={() => setIsMenuOpen(false)}
                            aria-label="Close Menu"
                        >
                            [ X ]
                        </button>
                    </div>

                    <nav className="mobile-nav">
                         <Link
                            to="/home"
                            className="mobile-nav-link"
                            activeProps={{ className: 'active' }}
                         >
                            <Home size={32} />
                            <CipherNavText ref={mobileHomeRef} text="Home" scrambleDuration={600} scrambleSpeed={50} />
                        </Link>
                        <Link
                            to="/voice"
                            className="mobile-nav-link"
                            activeProps={{ className: 'active' }}
                        >
                            <Mic size={32} />
                            <CipherNavText ref={mobileVoiceRef} text="Voice" scrambleDuration={600} scrambleSpeed={50} />
                        </Link>
                         {(user?.adminTribes?.length ?? 0) > 0 && (
                            <>
                                <Link
                                    to="/roster"
                                    className="mobile-nav-link"
                                    activeProps={{ className: 'active' }}
                                >
                                    <Users size={32} />
                                    <CipherNavText ref={mobileRosterRef} text="Roster" scrambleDuration={600} scrambleSpeed={50} />
                                </Link>
                                <div className="mobile-subnav">
                                     <AdminTribeNav />
                                </div>
                            </>
                        )}
                        {user?.isSuperAdmin && (
                            <Link
                                to="/super-admin"
                                className="mobile-nav-link"
                                activeProps={{ className: 'active' }}
                                style={{ color: 'var(--accent-primary)' }}
                            >
                                <ShieldAlert size={32} />
                                <CipherNavText ref={mobileSuperAdminRef} text="Super Admin" scrambleDuration={600} scrambleSpeed={50} />
                            </Link>
                        )}
                    </nav>

                    <div className="mobile-menu-footer">
                         <div className="mobile-theme-toggle">
                            <ThemeToggle />
                        </div>
                        <button onClick={logout} className="btn btn-primary btn-block" style={{ width: '100%' }}>
                            <LogOut size={18} style={{ marginRight: '0.5rem' }} />
                            LOGOUT
                        </button>
                    </div>
                </div>
            )}

            {children}

            <footer className="dashboard-footer">
                <div className="dashed-line" />
                <div className="footer-links">
                    {Object.entries(__MARKDOWN_METADATA__).map(([slug, { title }]) => (
                        <Link
                            key={slug}
                            to="/$page"
                            params={{ page: slug }}
                            className="footer-link"
                        >
                            {title}
                        </Link>
                    ))}
                    <div className="footer-copyright">
                        &copy; {new Date().getFullYear()} SCETROV
                    </div>
                </div>
            </footer>
        </div>
    )
}
