import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useEffect } from 'react'
import z from 'zod'
import { useAuth } from '../../providers/AuthProvider'

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
  const { setAuthToken } = useAuth()

  useEffect(() => {
    if (token) {
      setAuthToken(token)
      navigate({ to: '/home' })
    } else {
      navigate({ to: '/login' })
    }
  }, [token, navigate, setAuthToken])

  return (
    <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}>
      <p>Authenticating...</p>
    </div>
  )
}
