import { useAuth } from '../providers/AuthProvider';

export function AdminTribeNav() {
    const { user, currentTribe, setCurrentTribe } = useAuth();

    if (!user || !user.adminTribes || user.adminTribes.length <= 1) {
        return null;
    }

    return (
        <div className="secondary-nav" style={{ marginTop: '0.75rem' }}>
            {user.adminTribes.map((tribe) => (
                <button
                    key={tribe}
                    onClick={() => setCurrentTribe(tribe)}
                    className={`secondary-nav-item ${currentTribe === tribe ? 'active' : ''}`}
                >
                    {tribe}
                </button>
            ))}
        </div>
    );
}
