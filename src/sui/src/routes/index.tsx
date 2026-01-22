import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { ThemeToggle } from '../components/ThemeToggle'
import { useEffect } from 'react'
import { useAuth } from '../providers/AuthProvider'

export const Route = createFileRoute('/')({
  component: Index,
})

function Index() {
  const { isAuthenticated, login } = useAuth()
  const navigate = useNavigate()

  useEffect(() => {
    if (isAuthenticated) {
      navigate({ to: '/dashboard' })
    }
  }, [isAuthenticated, navigate])

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      justifyContent: 'center',
      minHeight: '100vh',
      gap: '2rem',
      padding: '1rem',
      width: '100%'
    }}>
      <div style={{ position: 'absolute', top: '1rem', right: '1rem' }}>
          <ThemeToggle />
      </div>

      <div className="card" style={{ maxWidth: '400px', width: '100%', textAlign: 'center' }}>
        <h1 style={{ marginBottom: '0.5rem' }}>Sui Mumble</h1>
        <p style={{ color: 'var(--text-secondary)', marginBottom: '2rem' }}>
          Connect with Discord to start managing your Sui wallets.
        </p>

        <button className="btn btn-primary" onClick={login} style={{ width: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5rem' }}>
          <div style={{ width: '26px', height: '20px', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
            <svg width="26" height="20" viewBox="0 0 26 20" fill="none" xmlns="http://www.w3.org/2000/svg" style={{ display: 'block', width: '100%', height: '100%' }}>
              <path d="M22.25 4.54a19 19 0 0 0-4.75-1.46.12.12 0 0 0-.13.08 13 13 0 0 0-.58 1.18 17.55 17.55 0 0 0-5.26 0 12.87 12.87 0 0 0-.59-1.18.12.12 0 0 0-.13-.08 19 19 0 0 0-4.75 1.46.06.06 0 0 0-.03.05C2.62 10.63 2 16.59 2.53 22.46a.07.07 0 0 0 .03.05 19.34 19.34 0 0 0 5.89 2.94.12.12 0 0 0 .13-.04 13.8 13.8 0 0 0 1.2-1.92.11.11 0 0 0-.06-.15 12.75 12.75 0 0 1-1.87-.88.08.08 0 0 1 0-.14l.36-.27c.05-.03.11-.03.16-.02a14.36 14.36 0 0 0 9.27 0 .1.1 0 0 1 .15.02l.37.28a.08.08 0 0 1 0 .13 12.63 12.63 0 0 1-1.87.89.11.11 0 0 0-.06.15 13.1 13.1 0 0 0 1.21 1.93.12.12 0 0 0 .13.04A19.34 19.34 0 0 0 23.44 22.5c0-.02.01-.04.02-.05.6-6.17-1.12-11.83-5.22-17.9zm-13.8 13.91c-1.7 0-3.1-1.54-3.1-3.42 0-1.89 1.37-3.42 3.1-3.42 1.73 0 3.12 1.54 3.1 3.42 0 1.89-1.37 3.42-3.1 3.42zm9.1 0c-1.7 0-3.1-1.54-3.1-3.42 0-1.89 1.37-3.42 3.1-3.42 1.73 0 3.12 1.54 3.1 3.42 0 1.89-1.37 3.42-3.1 3.42z" fill="currentColor"/>
            </svg>
          </div>
          Login with Discord
        </button>
      </div>
    </div>
  )
}
