import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { ThemeToggle } from '../components/ThemeToggle'
import { useEffect } from 'react'
import { useAuth } from '../providers/AuthProvider'
import DiscordLogo from '../assets/discord.svg'

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
        <h1 style={{ marginBottom: '0.5rem' }}>Void eID</h1>
        <p style={{ color: 'var(--text-secondary)', marginBottom: '2rem' }}>
          Connect with Discord to start managing your Sui wallets.
        </p>

        <button className="btn btn-primary" onClick={login} style={{ width: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5rem' }}>
          <img src={DiscordLogo} alt="Discord" style={{ width: '24px', height: '24px', objectFit: 'contain' }} />
          Login with Discord
        </button>
      </div>
    </div>
  )
}
