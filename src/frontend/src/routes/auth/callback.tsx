import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import z from 'zod'
import { useAuth } from '../../providers/AuthProvider'
import { API_URL } from '../../config'

const searchSchema = z.object({
  code: z.string().optional(),
})

export const Route = createFileRoute('/auth/callback')({
  validateSearch: searchSchema,
  component: AuthCallback,
})

function AuthCallback() {
  const { code } = Route.useSearch()
  const navigate = useNavigate()
  const { setAuthToken } = useAuth()
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!code) {
      navigate({ to: '/login' })
      return
    }

    // Exchange code for JWT token
    fetch(`${API_URL}/api/auth/exchange`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ code }),
    })
      .then((res) => {
        if (!res.ok) {
          throw new Error('Failed to exchange auth code')
        }
        return res.json()
      })
      .then((data: { token: string }) => {
        setAuthToken(data.token)
        navigate({ to: '/home' })
      })
      .catch((err) => {
        console.error('Auth exchange error:', err)
        setError('Authentication failed. Please try again.')
        setTimeout(() => navigate({ to: '/login' }), 2000)
      })
  }, [code, navigate, setAuthToken])

  if (error) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}>
        <p style={{ color: 'red' }}>{error}</p>
      </div>
    )
  }

  return (
    <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}>
      <p>Authenticating...</p>
    </div>
  )
}
