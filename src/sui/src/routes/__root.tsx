import { createRootRoute, Outlet } from '@tanstack/react-router'
// import { TanStackRouterDevtools } from '@tanstack/router-devtools'

export const Route = createRootRoute({
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
