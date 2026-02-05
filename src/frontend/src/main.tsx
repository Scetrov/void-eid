import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { RouterProvider, createRouter } from '@tanstack/react-router'
import { AppProviders } from './providers/AppProviders'
import { ThemeProvider } from './providers/ThemeProvider'
import './index.css'

// Import the generated route tree
import { routeTree } from './routeTree.gen'

// Create a new router instance
// Create a new router instance
const router = createRouter({
  routeTree,
  context: { auth: undefined! },
})

// Register the router instance for type safety
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}

// Render the app
const rootElement = document.getElementById('root')!
if (!rootElement.innerHTML) {
  const root = createRoot(rootElement)
  root.render(
    <StrictMode>
      <ThemeProvider>
        <AppProviders>
          <InnerApp />
        </AppProviders>
      </ThemeProvider>
    </StrictMode>,
  )
}

import { useAuth } from './providers/AuthProvider'

function InnerApp() {
  const auth = useAuth()
  return <RouterProvider router={router} context={{ auth }} />
}
