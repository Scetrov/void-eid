import { useAuth } from '../providers/AuthProvider';

export function TribeSelector() {
  const { user, currentTribe, setCurrentTribe } = useAuth();

  if (!user || !user.tribes || user.tribes.length === 0) {
    return null;
  }

  // If user has only one tribe, just display it
  if (user.tribes.length === 1) {
    return (
      <div className="tribe-display">
        <span className="tribe-label">Tribe:</span>
        <span className="tribe-name">{user.tribes[0]}</span>
      </div>
    );
  }

  // If user has multiple tribes, show a selector
  return (
    <div className="tribe-selector">
      <label htmlFor="tribe-select" className="tribe-label">
        Tribe:
      </label>
      <select
        id="tribe-select"
        value={currentTribe || ''}
        onChange={(e) => setCurrentTribe(e.target.value)}
        className="tribe-select"
      >
        {!currentTribe && (
          <option value="" disabled>
            Select a tribe...
          </option>
        )}
        {user.tribes.map((tribe) => (
          <option key={tribe} value={tribe}>
            {tribe}
          </option>
        ))}
      </select>
    </div>
  );
}
