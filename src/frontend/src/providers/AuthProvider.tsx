import { createContext, useContext, useState, useEffect, useCallback } from 'react';
import type { ReactNode } from 'react';
import { useSignPersonalMessage } from '@mysten/dapp-kit';
import { API_URL } from '../config';

export interface User {
    id: string;
    discordId: string;
    username: string;
    discriminator: string;
    avatar: string | null;
    tribes: string[];
    adminTribes: string[];
    isAdmin: boolean;
    isSuperAdmin: boolean;
    lastLoginAt: string | null;
    wallets: LinkedWallet[];
}

export interface LinkedWallet {
    id: string;
    address: string;
    verifiedAt: string;
    tribes: string[];
}

interface AuthContextType {
  isAuthenticated: boolean;
  user: User | null;
  token: string | null;
  currentTribe: string | null;
  setCurrentTribe: (tribe: string) => void;
  login: () => void;
  logout: () => void;
  linkWallet: (address: string) => Promise<void>;
  unlinkWallet: (id: string) => Promise<void>;
  isLoading: boolean;
  error: string | null;
  setAuthToken: (token: string | null) => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setToken] = useState<string | null>(localStorage.getItem('sui_jwt'));
  const [user, setUser] = useState<User | null>(null);
  const [currentTribe, setCurrentTribeState] = useState<string | null>(
    localStorage.getItem('current_tribe')
  );
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const { mutateAsync: signPersonalMessage } = useSignPersonalMessage();

  const fetchUser = useCallback(async (authToken: string) => {
      try {
          const res = await fetch(`${API_URL}/api/me`, {
              headers: { 'Authorization': `Bearer ${authToken}` }
          });
          if (res.ok) {
              const userData = await res.json();
              setUser(userData);

              // Auto-select tribe strategy:
              // 1. If user is admin of exactly one tribe, select it (prioritize admin context)
              if (userData.adminTribes && userData.adminTribes.length === 1) {
                  const adminTribe = userData.adminTribes[0];
                  setCurrentTribeState(adminTribe);
                  localStorage.setItem('current_tribe', adminTribe);
              }
              // 2. Fallback: If not an admin (or admin of 0), check regular tribes
              else if (userData.tribes && userData.tribes.length === 1) {
                  setCurrentTribeState(userData.tribes[0]);
                  localStorage.setItem('current_tribe', userData.tribes[0]);
              } else if (userData.tribes && userData.tribes.length > 1) {
                  // If user has multiple tribes, check if saved tribe is still valid
                  const savedTribe = localStorage.getItem('current_tribe');
                  if (savedTribe && userData.tribes.includes(savedTribe)) {
                      setCurrentTribeState(savedTribe);
                  } else {
                      // Clear invalid saved tribe
                      setCurrentTribeState(null);
                      localStorage.removeItem('current_tribe');
                  }
              }
          } else {
              // Token invalid
              localStorage.removeItem('sui_jwt');
              setToken(null);
              setUser(null);
          }
      } catch (e) {
          console.error("Failed to fetch user", e);
      }
  }, []);

  useEffect(() => {
      if (token) {
          fetchUser(token);
      }
  }, [token, fetchUser]);

  const login = () => {
    // Redirect to backend Discord Login
    window.location.href = `${API_URL}/api/auth/discord/login`;
  };

  const setCurrentTribe = useCallback((tribe: string) => {
    setCurrentTribeState(tribe);
    localStorage.setItem('current_tribe', tribe);
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem('sui_jwt');
    localStorage.removeItem('current_tribe');
    setToken(null);
    setUser(null);
    setCurrentTribeState(null);
  }, []);

  const setAuthToken = useCallback((newToken: string | null) => {
    if (newToken) {
        localStorage.setItem('sui_jwt', newToken);
    } else {
        localStorage.removeItem('sui_jwt');
    }
    setToken(newToken);
  }, []);

  const linkWallet = async (address: string) => {
    if (!token) {
        setError("You must be logged in to link a wallet.");
        return;
    }
    setError(null);
    setIsLoading(true);
    try {
        // 1. Get Nonce
        const nonceRes = await fetch(`${API_URL}/api/wallets/link-nonce`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({ address })
        });

        if (!nonceRes.ok) throw new Error('Failed to get nonce');
        const { nonce } = await nonceRes.json();

        // 2. Sign
        const message = new TextEncoder().encode(nonce);
        const { signature } = await signPersonalMessage({ message });

        // 3. Verify
        const verifyRes = await fetch(`${API_URL}/api/wallets/link-verify`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({ address, signature })
        });

        if (!verifyRes.ok) {
            const err = await verifyRes.json();
            throw new Error(err.message || 'Verification failed');
        }

        // Refresh user to see new wallet
        await fetchUser(token);

    } catch (err: unknown) {
        console.error(err);
        if (err instanceof Error) {
            setError(err.message || 'Linking failed');
        } else {
            setError('Linking failed');
        }
        throw err; // Re-throw so UI can handle success/fail feedback if needed
    } finally {
        setIsLoading(false);
    }
  };

  const unlinkWallet = async (walletId: string) => {
      if (!token) return;
      setIsLoading(true);
      try {
          const res = await fetch(`${API_URL}/api/wallets/${walletId}`, {
              method: 'DELETE',
              headers: { 'Authorization': `Bearer ${token}` }
          });
          if (!res.ok) throw new Error(`Failed to unlink wallet: ${res.status} ${res.statusText}`);
          await fetchUser(token);
      } catch (err: unknown) {
          console.error(err);
          if (err instanceof Error) {
              setError(err.message);
          } else {
              setError('Failed to unlink wallet');
          }
      } finally {
          setIsLoading(false);
      }
  };

  return (
    <AuthContext.Provider value={{
      isAuthenticated: !!user,
      user,
      token,
      currentTribe,
      setCurrentTribe,
      login,
      logout,
      linkWallet,
      unlinkWallet,
      isLoading,
      error,
      setAuthToken
    }}>
      {children}
    </AuthContext.Provider>
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
