use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod adevar_security_vault {
    use super::*;

    /// Initialize the Secure Vault with strict Authority & PDA constraints
    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.authority = ctx.accounts.authority.key();
        vault.bump = ctx.bumps.vault;
        vault.total_staked = 0;
        vault.is_locked = false;
        
        msg!("Vault Initialized Safely. Authority: {}", vault.authority);
        Ok(())
    }

    /// Deposit funds into the Vault using Checked Arithmetic
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        // Security Guard: Check reentrancy / lock state
        require!(!vault.is_locked, VaultError::VaultLocked);
        require!(amount > 0, VaultError::InvalidAmount);

        // Security Guard: Overflow protection via checked_add
        vault.total_staked = vault
            .total_staked
            .checked_add(amount)
            .ok_or(VaultError::MathOverflow)?;

        // Lock vault during processing state
        vault.is_locked = true;

        // Transfer SOL safely via System Program CPI
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.depositor.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_context, amount)?;

        // Unlock vault after state completion
        vault.is_locked = false;

        msg!("Deposit successful: {} lamports. Total: {}", amount, vault.total_staked);
        Ok(())
    }

    /// Emergency Withdraw restricted to Vault Authority
    pub fn emergency_withdraw(ctx: Context<EmergencyWithdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        // Access Control Verification
        require_keys_eq!(
            vault.authority,
            ctx.accounts.authority.key(),
            VaultError::Unauthorized
        );

        // Security Guard: Underflow protection via checked_sub
        vault.total_staked = vault
            .total_staked
            .checked_sub(amount)
            .ok_or(VaultError::InsufficientFunds)?;

        **vault.to_account_info().try_borrow_mut_lamports()? = vault
            .to_account_info()
            .lamports()
            .checked_sub(amount)
            .ok_or(VaultError::InsufficientFunds)?;

        **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? = ctx
            .accounts
            .authority
            .to_account_info()
            .lamports()
            .checked_add(amount)
            .ok_or(VaultError::MathOverflow)?;

        msg!("Emergency Withdrawal Executed: {} lamports", amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + VaultState::LEN,
        seeds = [b"vault", authority.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, VaultState>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault.authority.as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, VaultState>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EmergencyWithdraw<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", authority.key().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, VaultState>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct VaultState {
    pub authority: Pubkey, // 32 bytes
    pub bump: u8,          // 1 byte
    pub total_staked: u64, // 8 bytes
    pub is_locked: bool,   // 1 byte
}

impl VaultState {
    pub const LEN: usize = 32 + 1 + 8 + 1;
}

#[error_code]
pub enum VaultError {
    #[msg("Arithmetic Overflow Detected")]
    MathOverflow,
    #[msg("Insufficient Funds in Vault")]
    InsufficientFunds,
    #[msg("Unauthorized Operation Attempted")]
    Unauthorized,
    #[msg("Vault is Currently Locked")]
    VaultLocked,
    #[msg("Deposit Amount Must be Greater Than Zero")]
    InvalidAmount,
}