import { createFileRoute, Link } from '@tanstack/react-router'
import { ShieldAlert, LogOut } from 'lucide-react'

export const Route = createFileRoute('/deleted')({
  component: AccountDeleted,
})

function AccountDeleted() {
  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      justifyContent: 'center',
      alignItems: 'center',
      minHeight: '100vh',
      padding: '2rem',
      background: 'radial-gradient(circle at 50% -20%, #1a0a0a 0%, #050505 100%)',
      color: 'var(--text-primary)',
      textAlign: 'center',
      fontFamily: 'var(--font-heading)'
    }}>
      <div className="card card-static" style={{
        maxWidth: '600px',
        width: '100%',
        padding: '3rem',
        border: '1px solid #ef4444',
        background: 'rgba(239, 68, 68, 0.05)',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: '2rem'
      }}>
        <ShieldAlert size={64} color="#ef4444" />

        <div>
          <h1 style={{ color: '#ef4444', marginBottom: '1rem' }}>Account Terminated</h1>
          <div style={{
            color: 'var(--text-secondary)',
            lineHeight: '1.8',
            fontSize: '1.1rem',
            fontFamily: 'var(--font-body)',
            textTransform: 'none',
            letterSpacing: 'normal'
          }}>
            <p style={{ marginBottom: '1.5rem' }}>
              Your account has been permanently deleted at your request to fulfill your <a href="https://ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/individual-rights/individual-rights/right-to-erasure/" style={{ color: '#ef4444', textDecoration: 'underline', textUnderlineOffset: '4px', transition: 'opacity 0.2s' }} onMouseOver={(e) => e.currentTarget.style.opacity = '0.8'} onMouseOut={(e) => e.currentTarget.style.opacity = '1'}><strong>Right to be Forgotten / Right to erasure</strong></a> under GDPR.
            </p>
            <p>
              As part of our commitment to your privacy, your Discord ID and associated Wallet Addresses have been completely delisted and barred from future use to ensure that this data cannot be re-processed or used to recreate a profile on this platform.
            </p>
            <p>
              If you believe this to be in-error then please contact your tribe admins, however, please note there is no facility to restore a deleted account.
            </p>
          </div>
        </div>

        <div className="dashed-line"></div>

        <Link to="/login" className="btn btn-primary" style={{ gap: '0.75rem' }}>
          <LogOut size={18} />
          Back to Login
        </Link>
      </div>
    </div>
  )
}
