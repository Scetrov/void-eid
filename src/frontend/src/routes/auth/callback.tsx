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
      // Use navigate instead of window.location.href to avoid hard reload
      // The AuthProvider will pick up the token from localStorage on next render
      navigate({ to: '/home' })
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
