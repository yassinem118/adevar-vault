import './globals.css';
import { WalletContextProvider } from './components/WalletContextProvider';

export const metadata = {
  title: 'Adevar Security Vault',
  description: 'High-Assurance Solana Security Vault',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>
        <WalletContextProvider>
          {children}
        </WalletContextProvider>
      </body>
    </html>
  );
}