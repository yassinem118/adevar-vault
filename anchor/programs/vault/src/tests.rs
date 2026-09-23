#[cfg(test)]
mod tests {
    use crate::ID as PROGRAM_ID;
    use anchor_lang::system_program;
    use litesvm::LiteSVM;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        transaction::Transaction,
    };

    const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

    fn get_vault_pda(signer: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"vault", signer.as_ref()], &PROGRAM_ID)
    }

    fn create_initialize_ix(signer: &Pubkey, vault: &Pubkey) -> Instruction {
        // Anchor discriminator for "initialize_vault" = hash("global:initialize_vault")[0..8]
        let discriminator: [u8; 8] = [48, 191, 163, 44, 72, 122, 100, 182];

        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*signer, true),
                AccountMeta::new(*vault, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: discriminator.to_vec(),
        }
    }

    fn create_deposit_ix(signer: &Pubkey, vault: &Pubkey, amount: u64) -> Instruction {
        // Anchor discriminator for "deposit" = hash("global:deposit")[0..8]
        let discriminator: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
        let mut data = discriminator.to_vec();
        data.extend_from_slice(&amount.to_le_bytes());

        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*signer, true),
                AccountMeta::new(*vault, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data,
        }
    }

    fn create_emergency_withdraw_ix(authority: &Pubkey, vault: &Pubkey, amount: u64) -> Instruction {
        // Anchor discriminator for "emergency_withdraw" = hash("global:emergency_withdraw")[0..8]
        let discriminator: [u8; 8] = [238, 138, 226, 172, 12, 10, 163, 178];
        let mut data = discriminator.to_vec();
        data.extend_from_slice(&amount.to_le_bytes());

        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*authority, true),
                AccountMeta::new(*vault, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data,
        }
    }

    #[test]
    fn test_vault_lifecycle_success() {
        let mut svm = LiteSVM::new();

        // Load compiled binary
        let program_bytes = include_bytes!("../../../target/deploy/adevar_security_vault.so");
        svm.add_program(PROGRAM_ID, program_bytes);

        // Setup user with SOL
        let user = Keypair::new();
        svm.airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

        let (vault_pda, _bump) = get_vault_pda(&user.pubkey());

        // 1. Initialize Vault
        let init_ix = create_initialize_ix(&user.pubkey(), &vault_pda);
        let blockhash = svm.latest_blockhash();
        let init_tx = Transaction::new_signed_with_payer(
            &[init_ix],
            Some(&user.pubkey()),
            &[&user],
            blockhash,
        );
        assert!(svm.send_transaction(init_tx).is_ok(), "Initialization failed");

        // 2. Deposit 2 SOL
        let deposit_amount = 2 * LAMPORTS_PER_SOL;
        let deposit_ix = create_deposit_ix(&user.pubkey(), &vault_pda, deposit_amount);
        let blockhash = svm.latest_blockhash();
        let deposit_tx = Transaction::new_signed_with_payer(
            &[deposit_ix],
            Some(&user.pubkey()),
            &[&user],
            blockhash,
        );
        assert!(svm.send_transaction(deposit_tx).is_ok(), "Deposit failed");

        // Verify Vault Balance
        let vault_account = svm.get_account(&vault_pda).unwrap();
        assert!(vault_account.lamports >= deposit_amount);

        // 3. Emergency Withdraw 1 SOL
        let withdraw_amount = 1 * LAMPORTS_PER_SOL;
        let withdraw_ix = create_emergency_withdraw_ix(&user.pubkey(), &vault_pda, withdraw_amount);
        let blockhash = svm.latest_blockhash();
        let withdraw_tx = Transaction::new_signed_with_payer(
            &[withdraw_ix],
            Some(&user.pubkey()),
            &[&user],
            blockhash,
        );
        assert!(svm.send_transaction(withdraw_tx).is_ok(), "Emergency withdraw failed");
    }

    #[test]
    fn test_unauthorized_withdraw_fails() {
        let mut svm = LiteSVM::new();

        let program_bytes = include_bytes!("../../../target/deploy/adevar_security_vault.so");
        svm.add_program(PROGRAM_ID, program_bytes);

        let owner = Keypair::new();
        let attacker = Keypair::new();

        svm.airdrop(&owner.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();
        svm.airdrop(&attacker.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

        let (vault_pda, _bump) = get_vault_pda(&owner.pubkey());

        // Initialize by owner
        let init_ix = create_initialize_ix(&owner.pubkey(), &vault_pda);
        let blockhash = svm.latest_blockhash();
        let init_tx = Transaction::new_signed_with_payer(
            &[init_ix],
            Some(&owner.pubkey()),
            &[&owner],
            blockhash,
        );
        svm.send_transaction(init_tx).unwrap();

        // Attacker attempts emergency withdraw
        let withdraw_ix = create_emergency_withdraw_ix(&attacker.pubkey(), &vault_pda, 1 * LAMPORTS_PER_SOL);
        let blockhash = svm.latest_blockhash();
        let attacker_tx = Transaction::new_signed_with_payer(
            &[withdraw_ix],
            Some(&attacker.pubkey()),
            &[&attacker],
            blockhash,
        );

        let result = svm.send_transaction(attacker_tx);
        assert!(result.is_err(), "Attacker should NOT be able to withdraw");
    }
}