import { createRootRouteWithContext, Outlet } from '@tanstack/react-router'
// import { TanStackRouterDevtools } from '@tanstack/router-devtools'

interface MyRouterContext {
  auth: {
    isAuthenticated: boolean
  }
}

export const Route = createRootRouteWithContext<MyRouterContext>()({
  component: RootComponent,
})

function RootComponent() {
  return (
    <>
        <div className="app-container">
            <Outlet />
        </div>
        {/* <TanStackRouterDevtools /> */}
    </>
  )
}
