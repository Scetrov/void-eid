import { Moon, Sun, Laptop } from 'lucide-react'
import { useTheme } from '../providers/ThemeProvider'

export function ThemeToggle() {
  const { theme, setTheme } = useTheme()

  const buttonStyle = (isActive: boolean) => ({
    background: isActive ? 'var(--card-bg)' : 'transparent',
    border: isActive ? '1px solid var(--accent-primary)' : '1px solid transparent',
    padding: '0.5rem',
    cursor: 'pointer',
    color: isActive ? 'var(--accent-primary)' : 'var(--text-secondary)',
    transition: 'all 0.2s ease',
  })

  return (
    <div style={{
      display: 'flex',
      gap: '0.25rem',
      background: 'var(--bg-secondary)',
      padding: '0.25rem',
      border: '1px solid var(--glass-border)',
    }}>
      <button
        className="btn-icon"
        onClick={() => setTheme("light")}
        title="Light Mode"
        style={buttonStyle(theme === 'light')}
      >
        <Sun size={18} />
      </button>
      <button
        className="btn-icon"
        onClick={() => setTheme("dark")}
        title="Dark Mode"
        style={buttonStyle(theme === 'dark')}
      >
        <Moon size={18} />
      </button>
      <button
        className="btn-icon"
        onClick={() => setTheme("system")}
        title="System Preference"
        style={buttonStyle(theme === 'system')}
      >
        <Laptop size={18} />
      </button>
    </div>
  )
}
