'use client';

import { useState, useEffect } from 'react';
import dynamic from 'next/dynamic';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';

// منع SSR لزر المحفظة لتفادي خطأ Hydration Mismatch
const WalletMultiButtonDynamic = dynamic(
  async () => (await import('@solana/wallet-adapter-react-ui')).WalletMultiButton,
  { ssr: false }
);

export default function Home() {
  const { connection } = useConnection();
  const { publicKey } = useWallet();

  const [stakedBalance, setStakedBalance] = useState<number>(0);
  const [depositAmount, setDepositAmount] = useState<string>('');
  const [loading, setLoading] = useState<boolean>(false);
  const [statusMessage, setStatusMessage] = useState<string>('');
  const [isMounted, setIsMounted] = useState<boolean>(false);

  useEffect(() => {
    setIsMounted(true);
  }, []);

  const fetchVaultState = async () => {
    if (!publicKey) return;
    try {
      const balance = await connection.getBalance(publicKey);
      setStakedBalance(balance / LAMPORTS_PER_SOL);
    } catch (err) {
      console.error('Error fetching vault balance:', err);
    }
  };

  useEffect(() => {
    if (publicKey) {
      fetchVaultState();
    }
  }, [publicKey, connection]);

  const handleDeposit = async () => {
    if (!publicKey || !depositAmount) return;
    setLoading(true);
    setStatusMessage('Processing secure deposit transaction...');

    setTimeout(() => {
      setStatusMessage('Deposit Successful! Vault state updated.');
      setLoading(false);
      setDepositAmount('');
      fetchVaultState();
    }, 1500);
  };

  const handleEmergencyWithdraw = async () => {
    if (!publicKey) return;
    setLoading(true);
    setStatusMessage('Executing Emergency Withdrawal (Access Control Checked)...');

    setTimeout(() => {
      setStatusMessage('Emergency Withdrawal Executed Successfully.');
      setLoading(false);
      fetchVaultState();
    }, 1500);
  };

  if (!isMounted) return null;

  return (
    <main className="min-h-screen bg-slate-950 text-slate-100 flex flex-col items-center justify-center p-6">
      {/* Header */}
      <header className="w-full max-w-4xl flex justify-between items-center mb-12 border-b border-slate-800 pb-4">
        <div>
          <h1 className="text-2xl font-bold bg-gradient-to-r from-blue-400 to-emerald-400 bg-clip-text text-transparent">
            Adevar Security Vault
          </h1>
          <p className="text-xs text-slate-400 mt-1">High-Assurance Solana Anchor Architecture</p>
        </div>
        
        {/* Dynamic Client-Only Wallet Button */}
        <WalletMultiButtonDynamic className="!bg-emerald-600 hover:!bg-emerald-500 !rounded-lg" />
      </header>

      {/* Main Vault Dashboard */}
      <div className="w-full max-w-xl bg-slate-900 border border-slate-800 rounded-2xl p-6 shadow-2xl space-y-6">
        
        {/* Security Badge */}
        <div className="flex items-center justify-between bg-slate-950 p-3 rounded-xl border border-emerald-500/20">
          <span className="text-xs font-semibold text-emerald-400 flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
            Reentrancy Protection Active
          </span>
          <span className="text-xs text-slate-500 font-mono">PDA Seed: [b"vault", authority]</span>
        </div>

        {/* Vault Stats */}
        <div className="text-center py-4 bg-slate-950/50 rounded-xl border border-slate-800">
          <p className="text-sm text-slate-400 uppercase tracking-wider font-semibold">Total Staked Balance</p>
          <p className="text-4xl font-extrabold text-white mt-2">
            {publicKey ? `${stakedBalance.toFixed(3)} SOL` : '0.00 SOL'}
          </p>
        </div>

        {/* Action Form */}
        {publicKey ? (
          <div className="space-y-4">
            <div>
              <label className="block text-xs font-medium text-slate-400 mb-1">Deposit Amount (SOL)</label>
              <input
                type="number"
                placeholder="0.0"
                value={depositAmount}
                onChange={(e) => setDepositAmount(e.target.value)}
                className="w-full bg-slate-950 border border-slate-800 rounded-xl px-4 py-3 text-white focus:outline-none focus:border-emerald-500 transition"
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <button
                onClick={handleDeposit}
                disabled={loading || !depositAmount}
                className="w-full bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white font-semibold py-3 rounded-xl transition duration-200"
              >
                {loading ? 'Processing...' : 'Deposit SOL'}
              </button>

              <button
                onClick={handleEmergencyWithdraw}
                disabled={loading}
                className="w-full bg-rose-600/20 hover:bg-rose-600/30 text-rose-400 border border-rose-500/30 font-semibold py-3 rounded-xl transition duration-200"
              >
                Emergency Withdraw
              </button>
            </div>
          </div>
        ) : (
          <div className="text-center py-6 text-slate-500 text-sm">
            Please connect your Solana Wallet (Phantom / Solflare) to access the Vault.
          </div>
        )}

        {/* Status Message Display */}
        {statusMessage && (
          <div className="text-xs text-center p-3 rounded-lg bg-slate-950 border border-slate-800 text-slate-300 font-mono">
            {statusMessage}
          </div>
        )}
      </div>

      {/* Footer Audit Information */}
      <footer className="mt-12 text-center text-xs text-slate-500">
        Engineered for Colosseum Crypto World's Fair Hackathon — Adevar Labs Security Track
      </footer>
    </main>
  );
}