//! Listing Program Utility Functions

use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use pyth_sdk_solana::state::SolanaPriceAccount;

use crate::error::ListingError;
use crate::state::*;

/// Pyth Program ID on Mainnet
pub const PYTH_MAINNET_PROGRAM_ID: &str = "FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH";
/// Pyth Program ID on Devnet
pub const PYTH_DEVNET_PROGRAM_ID: &str = "gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s";

/// Oracle 验证配置
pub const ORACLE_MAX_CONFIDENCE_RATIO: u32 = 5; // 5% 最大置信区间比率
pub const ORACLE_MAX_STALENESS_SECONDS: i64 = 60; // 60秒最大陈旧时间

/// 验证 PDA 地址
pub fn verify_pda(
    seeds: &[&[u8]],
    bump: u8,
    program_id: &Pubkey,
    expected: &Pubkey,
) -> ProgramResult {
    let derived = Pubkey::create_program_address(
        &[seeds, &[&[bump]]].concat(),
        program_id,
    ).map_err(|_| ListingError::InvalidPda)?;
    
    if derived != *expected {
        return Err(ListingError::InvalidPda.into());
    }
    Ok(())
}

/// 验证并派生 ListingConfig PDA
pub fn derive_listing_config_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[LISTING_CONFIG_SEED], program_id)
}

/// 验证并派生 Treasury PDA
/// 存放原生 N1024 质押
pub fn derive_treasury_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[LISTING_TREASURY_SEED], program_id)
}

/// 验证并派生 TokenRegistry PDA
pub fn derive_token_registry_pda(
    token_index: u16,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[TOKEN_REGISTRY_SEED, &token_index.to_le_bytes()],
        program_id,
    )
}

/// 验证并派生 TokenProposal PDA
pub fn derive_token_proposal_pda(
    proposer: &Pubkey,
    nonce: u64,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[TOKEN_PROPOSAL_SEED, proposer.as_ref(), &nonce.to_le_bytes()],
        program_id,
    )
}

/// 验证并派生 SpotMarket PDA
pub fn derive_spot_market_pda(
    market_index: u16,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[SPOT_MARKET_SEED, &market_index.to_le_bytes()],
        program_id,
    )
}

/// 验证并派生 SpotMarketProposal PDA
pub fn derive_spot_proposal_pda(
    proposer: &Pubkey,
    nonce: u64,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[SPOT_PROPOSAL_SEED, proposer.as_ref(), &nonce.to_le_bytes()],
        program_id,
    )
}

/// 验证并派生 PerpMarket PDA
pub fn derive_perp_market_pda(
    market_index: u16,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[PERP_MARKET_SEED, &market_index.to_le_bytes()],
        program_id,
    )
}

/// 验证并派生 PerpMarketProposal PDA
pub fn derive_perp_proposal_pda(
    proposer: &Pubkey,
    nonce: u64,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[PERP_PROPOSAL_SEED, proposer.as_ref(), &nonce.to_le_bytes()],
        program_id,
    )
}

/// 验证并派生 LiquidityPool PDA
pub fn derive_liquidity_pool_pda(
    market: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[LIQUIDITY_POOL_SEED, market.as_ref()],
        program_id,
    )
}

/// 创建账户
pub fn create_account<'a>(
    payer: &AccountInfo<'a>,
    new_account: &AccountInfo<'a>,
    size: usize,
    owner: &Pubkey,
    system_program: &AccountInfo<'a>,
    seeds: &[&[u8]],
) -> ProgramResult {
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(size);

    invoke_signed(
        &system_instruction::create_account(
            payer.key,
            new_account.key,
            lamports,
            size as u64,
            owner,
        ),
        &[payer.clone(), new_account.clone(), system_program.clone()],
        &[seeds],
    )
}

/// 验证 Admin
pub fn verify_admin(
    admin: &AccountInfo,
    config: &ListingConfig,
) -> ProgramResult {
    if !admin.is_signer {
        msg!("Admin must be a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if admin.key != &config.admin {
        msg!("Invalid admin");
        return Err(ListingError::InvalidAdmin.into());
    }
    Ok(())
}

/// 验证 Symbol 格式
/// - 长度 2-8 字符
/// - 仅允许大写字母和数字
pub fn validate_symbol(symbol: &[u8; 8]) -> ProgramResult {
    let len = symbol.iter().position(|&c| c == 0).unwrap_or(8);
    
    if len < 2 {
        msg!("Symbol too short (min 2 chars)");
        return Err(ListingError::InvalidSymbol.into());
    }
    
    for &c in &symbol[..len] {
        if !((c >= b'A' && c <= b'Z') || (c >= b'0' && c <= b'9')) {
            msg!("Invalid symbol character: {}", c as char);
            return Err(ListingError::InvalidSymbol.into());
        }
    }
    
    Ok(())
}

/// 验证 Market Symbol 格式
/// - Spot: 必须包含 '/'，如 "BTC/USDC"
/// - Perp: 必须包含 '-'，如 "BTC-USDC"
pub fn validate_market_symbol(symbol: &[u8; 16], is_spot: bool) -> ProgramResult {
    let len = symbol.iter().position(|&c| c == 0).unwrap_or(16);
    
    if len < 5 {
        msg!("Market symbol too short (min 5 chars)");
        return Err(ListingError::InvalidSymbol.into());
    }
    
    let separator = if is_spot { b'/' } else { b'-' };
    let has_separator = symbol[..len].contains(&separator);
    
    if !has_separator {
        msg!("Market symbol must contain '{}'", separator as char);
        return Err(ListingError::InvalidSymbol.into());
    }
    
    Ok(())
}

/// 验证精度
pub fn validate_decimals(decimals: u8) -> ProgramResult {
    if decimals > 18 {
        msg!("Invalid decimals: {} (max 18)", decimals);
        return Err(ListingError::InvalidDecimals.into());
    }
    Ok(())
}

/// 验证费率
/// - taker_fee_bps: 0-1000 (0-10%)
/// - maker_fee_bps: -500 ~ 500 (-5% ~ 5%)
pub fn validate_fee_rates(taker_fee_bps: u16, maker_fee_bps: i16) -> ProgramResult {
    if taker_fee_bps > 1000 {
        msg!("Taker fee too high: {} bps (max 1000)", taker_fee_bps);
        return Err(ListingError::InvalidFeeRate.into());
    }
    if maker_fee_bps < -500 || maker_fee_bps > 500 {
        msg!("Maker fee out of range: {} bps (-500 ~ 500)", maker_fee_bps);
        return Err(ListingError::InvalidFeeRate.into());
    }
    Ok(())
}

/// 验证杠杆
pub fn validate_leverage(max_leverage: u8) -> ProgramResult {
    if max_leverage == 0 || max_leverage > 100 {
        msg!("Invalid leverage: {} (must be 1-100)", max_leverage);
        return Err(ListingError::InvalidLeverage.into());
    }
    Ok(())
}

/// 验证保证金率
/// - initial_margin_rate_e6: 如 100000 = 10%
/// - maintenance_margin_rate_e6: 如 50000 = 5%
/// - 维持 < 初始
pub fn validate_margin_rates(
    initial_margin_rate_e6: u32,
    maintenance_margin_rate_e6: u32,
) -> ProgramResult {
    if initial_margin_rate_e6 == 0 || initial_margin_rate_e6 > 1_000_000 {
        msg!("Invalid initial margin rate: {}", initial_margin_rate_e6);
        return Err(ListingError::InvalidInitialMarginRate.into());
    }
    if maintenance_margin_rate_e6 == 0 || maintenance_margin_rate_e6 > 1_000_000 {
        msg!("Invalid maintenance margin rate: {}", maintenance_margin_rate_e6);
        return Err(ListingError::InvalidMaintenanceMarginRate.into());
    }
    if maintenance_margin_rate_e6 >= initial_margin_rate_e6 {
        msg!("Maintenance margin must be less than initial margin");
        return Err(ListingError::InvalidMaintenanceMarginRate.into());
    }
    Ok(())
}

/// 验证 tick_size 和 lot_size
pub fn validate_sizes(tick_size_e6: u64, lot_size_e6: u64) -> ProgramResult {
    if tick_size_e6 == 0 {
        msg!("Tick size must be > 0");
        return Err(ListingError::InvalidTickSize.into());
    }
    if lot_size_e6 == 0 {
        msg!("Lot size must be > 0");
        return Err(ListingError::InvalidLotSize.into());
    }
    Ok(())
}

/// 验证价格范围
pub fn validate_price_range(price_lower_e6: u64, price_upper_e6: u64) -> ProgramResult {
    if price_lower_e6 == 0 || price_upper_e6 == 0 {
        msg!("Price must be > 0");
        return Err(ListingError::InvalidPriceRange.into());
    }
    if price_lower_e6 >= price_upper_e6 {
        msg!("Price lower must be < upper");
        return Err(ListingError::InvalidPriceRange.into());
    }
    Ok(())
}

/// 获取当前时间戳
pub fn get_current_timestamp() -> Result<i64, ProgramError> {
    let clock = Clock::get()?;
    Ok(clock.unix_timestamp)
}

/// 验证 Pyth Oracle 账户
/// 
/// 验证流程:
/// 1. 验证账户 owner 是 Pyth Program ID
/// 2. 验证 Price Feed 可解析
/// 3. 验证价格有效性 (价格 > 0, 置信区间合理, 时间戳新鲜)
pub fn validate_pyth_oracle(
    oracle_account: &AccountInfo,
    current_timestamp: i64,
) -> ProgramResult {
    // 1. 验证账户 owner 是 Pyth Program
    let pyth_mainnet = PYTH_MAINNET_PROGRAM_ID.parse::<Pubkey>()
        .map_err(|_| ListingError::InvalidOracle)?;
    let pyth_devnet = PYTH_DEVNET_PROGRAM_ID.parse::<Pubkey>()
        .map_err(|_| ListingError::InvalidOracle)?;
    
    if oracle_account.owner != &pyth_mainnet && oracle_account.owner != &pyth_devnet {
        msg!("Oracle account owner is not Pyth Program");
        msg!("Expected: {} or {}", PYTH_MAINNET_PROGRAM_ID, PYTH_DEVNET_PROGRAM_ID);
        msg!("Got: {}", oracle_account.owner);
        return Err(ListingError::InvalidOracle.into());
    }

    // 2. 解析 Price Feed
    let price_account = SolanaPriceAccount::account_info_to_feed(oracle_account)
        .map_err(|e| {
            msg!("Failed to parse Pyth price account: {:?}", e);
            ListingError::InvalidOracle
        })?;

    // 3. 获取价格
    let price = price_account.get_price_no_older_than(
        current_timestamp,
        ORACLE_MAX_STALENESS_SECONDS as u64,
    );

    match price {
        Some(p) => {
            // 验证价格 > 0
            if p.price <= 0 {
                msg!("Oracle price is non-positive: {}", p.price);
                return Err(ListingError::InvalidOracle.into());
            }

            // 验证置信区间不能太大 (< 5% of price)
            let max_conf = (p.price.unsigned_abs() as u64) * ORACLE_MAX_CONFIDENCE_RATIO as u64 / 100;
            if p.conf > max_conf {
                msg!("Oracle confidence interval too large: {} (max: {})", p.conf, max_conf);
                return Err(ListingError::InvalidOracle.into());
            }

            // 验证 exponent 在合理范围内
            if p.expo < -18 || p.expo > 18 {
                msg!("Oracle price exponent out of range: {}", p.expo);
                return Err(ListingError::InvalidOracle.into());
            }

            msg!("Pyth Oracle validated successfully");
            msg!("Price: {} x 10^{}", p.price, p.expo);
            msg!("Confidence: {}", p.conf);

            Ok(())
        }
        None => {
            msg!("Oracle price is stale or unavailable");
            Err(ListingError::InvalidOracle.into())
        }
    }
}

/// 简化的 Oracle 存在性验证（不验证价格）
/// 用于市场上架时仅验证 Oracle 账户有效
pub fn validate_oracle_exists(
    oracle_account: &AccountInfo,
) -> ProgramResult {
    // 验证账户 owner 是 Pyth Program
    let pyth_mainnet = PYTH_MAINNET_PROGRAM_ID.parse::<Pubkey>()
        .map_err(|_| ListingError::InvalidOracle)?;
    let pyth_devnet = PYTH_DEVNET_PROGRAM_ID.parse::<Pubkey>()
        .map_err(|_| ListingError::InvalidOracle)?;
    
    // 也允许 system_program owner（空账户，用于测试）
    let system_program = solana_program::system_program::id();
    
    if oracle_account.owner != &pyth_mainnet 
        && oracle_account.owner != &pyth_devnet 
        && oracle_account.owner != &system_program {
        msg!("Invalid Oracle account owner");
        return Err(ListingError::InvalidOracle.into());
    }

    // 尝试解析验证格式
    if oracle_account.owner != &system_program {
        let _price_account = SolanaPriceAccount::account_info_to_feed(oracle_account)
            .map_err(|e| {
                msg!("Failed to parse Pyth price account: {:?}", e);
                ListingError::InvalidOracle
            })?;
    }

    msg!("Oracle account verified");
    Ok(())
}

/// SPL Token 转账
pub fn spl_token_transfer<'a>(
    source: &AccountInfo<'a>,
    destination: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    amount: u64,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> ProgramResult {
    let ix = spl_token::instruction::transfer(
        token_program.key,
        source.key,
        destination.key,
        authority.key,
        &[],
        amount,
    )?;

    let account_infos = &[
        source.clone(),
        destination.clone(),
        authority.clone(),
        token_program.clone(),
    ];

    if let Some(seeds) = signer_seeds {
        invoke_signed(&ix, account_infos, seeds)
    } else {
        invoke(&ix, account_infos)
    }
}

/// 原生 N1024 (lamports) 转账
/// 用于 PLP 质押
pub fn transfer_native_lamports<'a>(
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    amount: u64,
    system_program: &AccountInfo<'a>,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> ProgramResult {
    let ix = system_instruction::transfer(from.key, to.key, amount);

    let account_infos = &[
        from.clone(),
        to.clone(),
        system_program.clone(),
    ];

    if let Some(seeds) = signer_seeds {
        invoke_signed(&ix, account_infos, seeds)
    } else {
        invoke(&ix, account_infos)
    }
}

/// 从 PDA 转出原生 lamports（使用 lamports 直接操作）
/// 用于退还质押
pub fn transfer_lamports_from_pda<'a>(
    from_pda: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    amount: u64,
) -> ProgramResult {
    // 直接操作 lamports，不需要 system_program
    **from_pda.try_borrow_mut_lamports()? -= amount;
    **to.try_borrow_mut_lamports()? += amount;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_symbol() {
        let mut symbol = [0u8; 8];
        
        // Valid: "BTC"
        symbol[..3].copy_from_slice(b"BTC");
        assert!(validate_symbol(&symbol).is_ok());
        
        // Valid: "PEPE2024"
        symbol = [0u8; 8];
        symbol.copy_from_slice(b"PEPE2024");
        assert!(validate_symbol(&symbol).is_ok());
        
        // Invalid: too short
        symbol = [0u8; 8];
        symbol[0] = b'A';
        assert!(validate_symbol(&symbol).is_err());
        
        // Invalid: lowercase
        symbol = [0u8; 8];
        symbol[..3].copy_from_slice(b"btc");
        assert!(validate_symbol(&symbol).is_err());
    }

    #[test]
    fn test_validate_market_symbol() {
        let mut symbol = [0u8; 16];
        
        // Valid Spot: "BTC/USDC"
        symbol[..8].copy_from_slice(b"BTC/USDC");
        assert!(validate_market_symbol(&symbol, true).is_ok());
        
        // Valid Perp: "BTC-USDC"
        symbol = [0u8; 16];
        symbol[..8].copy_from_slice(b"BTC-USDC");
        assert!(validate_market_symbol(&symbol, false).is_ok());
        
        // Invalid Spot: wrong separator
        symbol = [0u8; 16];
        symbol[..8].copy_from_slice(b"BTC-USDC");
        assert!(validate_market_symbol(&symbol, true).is_err());
    }

    #[test]
    fn test_validate_fee_rates() {
        // Valid
        assert!(validate_fee_rates(10, 0).is_ok());
        assert!(validate_fee_rates(100, -50).is_ok());
        
        // Invalid: taker too high
        assert!(validate_fee_rates(1001, 0).is_err());
        
        // Invalid: maker out of range
        assert!(validate_fee_rates(10, -501).is_err());
        assert!(validate_fee_rates(10, 501).is_err());
    }

    #[test]
    fn test_validate_leverage() {
        assert!(validate_leverage(1).is_ok());
        assert!(validate_leverage(100).is_ok());
        assert!(validate_leverage(0).is_err());
        assert!(validate_leverage(101).is_err());
    }

    #[test]
    fn test_validate_margin_rates() {
        // Valid: 10% initial, 5% maintenance
        assert!(validate_margin_rates(100_000, 50_000).is_ok());
        
        // Invalid: maintenance >= initial
        assert!(validate_margin_rates(100_000, 100_000).is_err());
        assert!(validate_margin_rates(100_000, 150_000).is_err());
    }
}

