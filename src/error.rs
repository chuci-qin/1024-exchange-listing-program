//! Listing Program Error Types

use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum ListingError {
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,

    /// Invalid admin
    #[error("Invalid admin")]
    InvalidAdmin,

    /// Account already initialized
    #[error("Account already initialized")]
    AlreadyInitialized,

    /// Account not initialized
    #[error("Account not initialized")]
    NotInitialized,

    /// Invalid PDA
    #[error("Invalid PDA")]
    InvalidPda,

    /// Invalid symbol format
    #[error("Invalid symbol format")]
    InvalidSymbol,

    /// Symbol too long (max 8 chars)
    #[error("Symbol too long")]
    SymbolTooLong,

    /// Invalid decimals (must be 0-18)
    #[error("Invalid decimals")]
    InvalidDecimals,

    /// Insufficient stake amount
    #[error("Insufficient stake amount")]
    InsufficientStake,

    /// Proposal not in pending status
    #[error("Proposal not pending")]
    ProposalNotPending,

    /// Proposal review deadline not reached
    #[error("Review deadline not reached")]
    ReviewDeadlineNotReached,

    /// Proposal review deadline passed
    #[error("Review deadline passed")]
    ReviewDeadlinePassed,

    /// Not the proposer
    #[error("Not the proposer")]
    NotProposer,

    /// Token not registered
    #[error("Token not registered")]
    TokenNotRegistered,

    /// Token not active
    #[error("Token not active")]
    TokenNotActive,

    /// Market already exists
    #[error("Market already exists")]
    MarketAlreadyExists,

    /// Invalid market type
    #[error("Invalid market type")]
    InvalidMarketType,

    /// Invalid tick size
    #[error("Invalid tick size")]
    InvalidTickSize,

    /// Invalid lot size
    #[error("Invalid lot size")]
    InvalidLotSize,

    /// Invalid fee rate
    #[error("Invalid fee rate")]
    InvalidFeeRate,

    /// Invalid oracle
    #[error("Invalid oracle")]
    InvalidOracle,

    /// Oracle required for perp market
    #[error("Oracle required for perp market")]
    OracleRequired,

    /// Invalid leverage (must be 1-100)
    #[error("Invalid leverage")]
    InvalidLeverage,

    /// Invalid initial margin rate
    #[error("Invalid initial margin rate")]
    InvalidInitialMarginRate,

    /// Invalid maintenance margin rate
    #[error("Invalid maintenance margin rate")]
    InvalidMaintenanceMarginRate,

    /// Stake lock period not ended
    #[error("Stake lock period not ended")]
    StakeLockPeriodNotEnded,

    /// Stake already claimed
    #[error("Stake already claimed")]
    StakeAlreadyClaimed,

    /// Market is paused
    #[error("Market is paused")]
    MarketPaused,

    /// Listing is paused
    #[error("Listing is paused")]
    ListingPaused,

    /// Liquidity pool not initialized
    #[error("Liquidity pool not initialized")]
    LiquidityPoolNotInitialized,

    /// Liquidity pool already initialized
    #[error("Liquidity pool already initialized")]
    LiquidityPoolAlreadyInitialized,

    /// Insufficient liquidity
    #[error("Insufficient liquidity")]
    InsufficientLiquidity,

    /// Invalid price range
    #[error("Invalid price range")]
    InvalidPriceRange,

    /// Numerical overflow
    #[error("Numerical overflow")]
    Overflow,

    /// Invalid account owner
    #[error("Invalid account owner")]
    InvalidAccountOwner,

    /// Quote token must be USDC
    #[error("Quote token must be USDC")]
    QuoteTokenMustBeUsdc,

    /// Same token pair
    #[error("Base and quote token cannot be the same")]
    SameTokenPair,

    /// Invalid account
    #[error("Invalid account")]
    InvalidAccount,

    /// Invalid amount
    #[error("Invalid amount")]
    InvalidAmount,

    /// Unauthorized action
    #[error("Unauthorized")]
    Unauthorized,

    /// Market not found
    #[error("Market not found")]
    MarketNotFound,

    /// Invalid order density
    #[error("Invalid order density (must be 1-100)")]
    InvalidOrderDensity,

    /// Invalid spread
    #[error("Invalid spread")]
    InvalidSpread,

    /// Invalid lock period
    #[error("Invalid lock period")]
    InvalidLockPeriod,

    /// Pool not active
    #[error("Pool not active")]
    PoolNotActive,

    /// Insufficient balance
    #[error("Insufficient balance")]
    InsufficientBalance,

    /// Numerical underflow
    #[error("Numerical underflow")]
    Underflow,

    /// Pool has remaining funds
    #[error("Pool has remaining funds - withdraw first")]
    PoolHasRemainingFunds,
}

impl From<ListingError> for ProgramError {
    fn from(e: ListingError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

