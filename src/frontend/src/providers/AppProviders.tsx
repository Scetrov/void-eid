import { createNetworkConfig, SuiClientProvider, WalletProvider } from '@mysten/dapp-kit';
import { JsonRpcHTTPTransport } from '@mysten/sui/jsonRpc';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { AuthProvider } from './AuthProvider';
import { ApiGuard } from '../components/ApiGuard';
import '@mysten/dapp-kit/dist/index.css';

const { networkConfig } = createNetworkConfig({
	localnet: {
		transport: new JsonRpcHTTPTransport({ url: 'http://127.0.0.1:9000' }),
		network: 'localnet',
	},
	devnet: {
		transport: new JsonRpcHTTPTransport({ url: 'https://fullnode.devnet.sui.io:443' }),
		network: 'devnet',
	},
	testnet: {
		transport: new JsonRpcHTTPTransport({ url: 'https://fullnode.testnet.sui.io:443' }),
		network: 'testnet',
	},
	mainnet: {
		transport: new JsonRpcHTTPTransport({ url: 'https://fullnode.mainnet.sui.io:443' }),
		network: 'mainnet',
	},
});

const queryClient = new QueryClient();

export function AppProviders({ children }: { children: ReactNode }) {
	return (
		<QueryClientProvider client={queryClient}>
			<SuiClientProvider networks={networkConfig} defaultNetwork="testnet">
				<WalletProvider>
					<ApiGuard>
						<AuthProvider>
							{children}
						</AuthProvider>
					</ApiGuard>
				</WalletProvider>
			</SuiClientProvider>
		</QueryClientProvider>
	);
}
