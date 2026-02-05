import { createFileRoute } from '@tanstack/react-router'
import { MumbleStatus } from '../components/MumbleStatus'
import { DashboardLayout } from '../components/DashboardLayout'

export const Route = createFileRoute('/voice')({
    component: MumblePage,
})

function MumblePage() {
    return (
        <DashboardLayout>
            <MumbleStatus />
        </DashboardLayout>
    )
}
