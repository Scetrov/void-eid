import { useState, useEffect, type ReactNode } from 'react';
import { ApiUnavailableScreen } from './ApiUnavailableScreen';
import { API_URL } from '../config';


interface ApiGuardProps {
    children: ReactNode;
}

export function ApiGuard({ children }: ApiGuardProps) {
    const [isAvailable, setIsAvailable] = useState<boolean | null>(null);
    const [retryCount, setRetryCount] = useState(0);
    const [statusText, setStatusText] = useState("Checking API status...");

    useEffect(() => {
        let isMounted = true;
        const checkApi = async () => {
            try {
                // Try fetching the /docs endpoint which should be publicly available
                // and returns 200 OK HTML content
                const controller = new AbortController();
                const timeoutId = setTimeout(() => controller.abort(), 5000); // 5s timeout

                const res = await fetch(`${API_URL}/docs`, {
                    method: 'GET',
                    signal: controller.signal
                });
                
                clearTimeout(timeoutId);

                if (res.ok) {
                    if (isMounted) {
                        setIsAvailable(true);
                        setStatusText("Connected.");
                    }
                } else {
                    throw new Error(`Status: ${res.status}`);
                }
            } catch {
                if (isMounted) {
                    setIsAvailable(false);
                    setStatusText(`Connection failed. Retrying... (${retryCount + 1})`);
                    
                    // Schedule retry
                    setTimeout(() => {
                        if (isMounted) {
                            setRetryCount(c => c + 1);
                        }
                    }, 3000);
                }
            }
        };

        checkApi();

        return () => {
            isMounted = false;
        };
    }, [retryCount]);

    if (isAvailable === null) {
        // Initial loading state
        return <ApiUnavailableScreen statusText={statusText} />;
    }

    if (!isAvailable) {
        return <ApiUnavailableScreen statusText={statusText} />;
    }

    return <>{children}</>;
}
