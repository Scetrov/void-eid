import { createContext, useContext, useState, useEffect, useCallback } from 'react';
import type { ReactNode } from 'react';
import { useSignPersonalMessage } from '@mysten/dapp-kit';

export interface User {
    id: string;
    discordId: string;
    username: string;
    discriminator: string;
    avatar: string | null;
    tribe: string | null;
    isAdmin: boolean;
    lastLoginAt: string | null;
    wallets: LinkedWallet[];
}

export interface LinkedWallet {
    id: string;
    address: string;
    verifiedAt: string;
}

interface AuthContextType {
  isAuthenticated: boolean;
  user: User | null;
  token: string | null;
  login: () => void;
  logout: () => void;
  linkWallet: (address: string) => Promise<void>;
  unlinkWallet: (id: string) => Promise<void>;
  isLoading: boolean;
  error: string | null;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setToken] = useState<string | null>(localStorage.getItem('sui_jwt'));
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const { mutateAsync: signPersonalMessage } = useSignPersonalMessage();

  const fetchUser = useCallback(async (authToken: string) => {
      try {
          const res = await fetch('http://localhost:5038/api/me', {
              headers: { 'Authorization': `Bearer ${authToken}` }
          });
          if (res.ok) {
              const userData = await res.json();
              setUser(userData);
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
    window.location.href = 'http://localhost:5038/api/auth/discord/login';
  };

  const logout = useCallback(() => {
    localStorage.removeItem('sui_jwt');
    setToken(null);
    setUser(null);
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
        const nonceRes = await fetch('http://localhost:5038/api/wallets/link-nonce', {
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
        const verifyRes = await fetch('http://localhost:5038/api/wallets/link-verify', {
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
          const res = await fetch(`http://localhost:5038/api/wallets/${walletId}`, {
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
    <AuthContext.Provider value={{ isAuthenticated: !!user, user, token, login, logout, linkWallet, unlinkWallet, isLoading, error }}>
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
