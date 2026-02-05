import { createFileRoute, redirect } from '@tanstack/react-router'
import { MumbleStatus } from '../components/MumbleStatus'
import { DashboardLayout } from '../components/DashboardLayout'

export const Route = createFileRoute('/mumble')({
    beforeLoad: ({ context }) => {
        if (!context.auth.isAuthenticated) {
            throw redirect({
                to: '/login',
            })
        }
    },
    component: MumblePage,
})

function MumblePage() {
    return (
        <DashboardLayout>
            <MumbleStatus />
        </DashboardLayout>
    )
}
