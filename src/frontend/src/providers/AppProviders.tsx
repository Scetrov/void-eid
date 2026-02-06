import { createNetworkConfig, SuiClientProvider, WalletProvider } from '@mysten/dapp-kit';
import { getJsonRpcFullnodeUrl, JsonRpcHTTPTransport } from '@mysten/sui/jsonRpc';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { AuthProvider } from './AuthProvider';
import { ApiGuard } from '../components/ApiGuard';
import '@mysten/dapp-kit/dist/index.css';

const { networkConfig } = createNetworkConfig({
	localnet: {
		transport: new JsonRpcHTTPTransport({ url: getJsonRpcFullnodeUrl('localnet') }),
		network: 'localnet',
	},
	devnet: {
		transport: new JsonRpcHTTPTransport({ url: getJsonRpcFullnodeUrl('devnet') }),
		network: 'devnet',
	},
	testnet: {
		transport: new JsonRpcHTTPTransport({ url: getJsonRpcFullnodeUrl('testnet') }),
		network: 'testnet',
	},
	mainnet: {
		transport: new JsonRpcHTTPTransport({ url: getJsonRpcFullnodeUrl('mainnet') }),
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
