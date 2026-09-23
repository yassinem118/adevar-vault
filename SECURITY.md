# Security Architecture & Threat Model

## Overview
`adevar-security-vault` is a high-assurance Solana smart contract engineered with strict defensive programming principles to minimize attack vectors.

## Invariants & Security Guarantees
1. **Checked Arithmetic Operations:** All balance changes utilize native Rust checked math (`checked_add`, `checked_sub`) to prevent integer overflows and underflows.
2. **Deterministic PDA Derivation:** Vault state accounts are strictly derived using fixed seeds `[b"vault", authority]` and validated on-chain via Canonical Bump checks.
3. **Reentrancy Protection Lock:** Reentrancy guard flags (`is_locked`) ensure atomic execution state transitions during CPI and lamport transfers.
4. **Role-Based Access Control:** Critical instructions (such as `emergency_withdraw`) validate Signer Identity matching the Vault Authority pubkey.

## Testing Strategy
Integration security testing is implemented using `LiteSVM` for lightweight, fast-execution verification of state transitions, edge-case failure modes, and unauthorized access attempts.