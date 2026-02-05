import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useEffect } from 'react'
import z from 'zod'

const searchSchema = z.object({
  token: z.string().optional(),
})

export const Route = createFileRoute('/auth/callback')({
  validateSearch: searchSchema,
  component: AuthCallback,
})

function AuthCallback() {
  const { token } = Route.useSearch()
  const navigate = useNavigate()

  useEffect(() => {
    if (token) {
      localStorage.setItem('sui_jwt', token)
      // Small delay or direct redirect. AuthProvider will pick it up on mount/update but we might need a reload or context update.
      // Since AuthProvider reads from localStorage on mount, we should trigger a window reload or use a setter if exposed (we exposed setToken via login, but here we are outside context initially?)
      // Actually AuthCallback is inside AuthProvider (if configured in main/root).
      // Let's force reload to be safe and simple for now, or just navigate to dashboard and let AuthProvider fetchUser.

      // Better: navigate to dashboard. But AuthProvider needs to know token changed if it doesn't poll localStorage.
      // Our AuthProvider `useEffect` depends on `token` state, which is initialized from localStorage.
      // We need to update the state.
      // Ideally we use a `useAuth` hook here and call a `setToken` method, but `setToken` is internal.
      // Let's redirect to dashboard with a hard reload to be sure.
      window.location.href = '/home'
    } else {
      navigate({ to: '/login' })
    }
  }, [token, navigate])

  return (
    <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}>
      <p>Authenticating...</p>
    </div>
  )
}
