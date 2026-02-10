import { createFileRoute } from '@tanstack/react-router'
import { useAuth } from '../providers/AuthProvider'
import DiscordLogo from "../assets/discord.svg";

export const Route = createFileRoute('/login')({
  component: Login,
})

function Login() {
  const { login } = useAuth()

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      justifyContent: 'center',
      minHeight: '100vh',
      gap: '2rem',
      padding: '1rem'
    }}>
      <div className="card" style={{ maxWidth: '400px', width: '100%', textAlign: 'center' }}>
        <img src="/logo.png" alt="VoID eID" style={{ maxWidth: '300px', height: 'auto', margin: '0 auto 1.5rem', display: 'block' }} />
        <p style={{ color: 'var(--text-secondary)', marginBottom: '2rem' }}>
          Login with Discord first, you will then be able to connect your EVE Frontier character by linking your wallets.
        </p>

        <button className="btn btn-primary" onClick={login} style={{ width: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5rem' }}>
            {/* Discord Logo PNG */}
            <img src={DiscordLogo} alt="Discord" style={{ width: '24px', height: '24px', objectFit: 'contain' }} />
          Login with Discord
        </button>
      </div>
    </div>
  )
}
