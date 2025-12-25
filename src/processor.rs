//! Listing Program Instruction Processor
//!
//! PLP (Permissionless Listing Protocol) 指令处理器

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use crate::error::ListingError;
use crate::instruction::ListingInstruction;
use crate::state::*;
use crate::utils::*;

/// 主处理入口
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = ListingInstruction::try_from_slice(instruction_data)
        .map_err(|_| ListingError::InvalidInstruction)?;

    match instruction {
        // =====================================================================
        // Admin 指令
        // =====================================================================
        ListingInstruction::Initialize {
            vault_program,
            fund_program,
            ledger_program,
        } => {
            msg!("Instruction: Initialize");
            process_initialize(program_id, accounts, vault_program, fund_program, ledger_program)
        }

        ListingInstruction::UpdateAdmin { new_admin } => {
            msg!("Instruction: UpdateAdmin");
            process_update_admin(program_id, accounts, new_admin)
        }

        ListingInstruction::UpdateStakeConfig {
            token_stake_amount,
            spot_stake_amount,
            perp_stake_amount,
        } => {
            msg!("Instruction: UpdateStakeConfig");
            process_update_stake_config(
                program_id,
                accounts,
                token_stake_amount,
                spot_stake_amount,
                perp_stake_amount,
            )
        }

        ListingInstruction::UpdateReviewPeriods {
            token_review_period,
            spot_review_period,
            perp_review_period,
            stake_lock_period,
        } => {
            msg!("Instruction: UpdateReviewPeriods");
            process_update_review_periods(
                program_id,
                accounts,
                token_review_period,
                spot_review_period,
                perp_review_period,
                stake_lock_period,
            )
        }

        ListingInstruction::SetPaused { paused } => {
            msg!("Instruction: SetPaused");
            process_set_paused(program_id, accounts, paused)
        }

        // =====================================================================
        // PLP-1: Token 注册指令
        // =====================================================================
        ListingInstruction::ProposeToken {
            nonce,
            symbol,
            mint,
            decimals,
            oracle,
        } => {
            msg!("Instruction: ProposeToken");
            process_propose_token(program_id, accounts, nonce, symbol, mint, decimals, oracle)
        }

        ListingInstruction::ObjectToken { stake_amount } => {
            msg!("Instruction: ObjectToken");
            process_object_token(program_id, accounts, stake_amount)
        }

        ListingInstruction::ApproveToken => {
            msg!("Instruction: ApproveToken");
            process_approve_token(program_id, accounts)
        }

        ListingInstruction::RejectToken {
            reason_code,
            slash_percentage,
        } => {
            msg!("Instruction: RejectToken");
            process_reject_token(program_id, accounts, reason_code, slash_percentage)
        }

        ListingInstruction::CancelTokenProposal => {
            msg!("Instruction: CancelTokenProposal");
            process_cancel_token_proposal(program_id, accounts)
        }

        ListingInstruction::FinalizeToken => {
            msg!("Instruction: FinalizeToken");
            process_finalize_token(program_id, accounts)
        }

        ListingInstruction::ClaimTokenStake => {
            msg!("Instruction: ClaimTokenStake");
            process_claim_token_stake(program_id, accounts)
        }

        ListingInstruction::UpdateTokenStatus { is_active } => {
            msg!("Instruction: UpdateTokenStatus");
            process_update_token_status(program_id, accounts, is_active)
        }

        // =====================================================================
        // PLP-2: Spot 市场上架指令
        // =====================================================================
        ListingInstruction::ProposeSpotMarket {
            nonce,
            symbol,
            base_token_index,
            quote_token_index,
            tick_size_e6,
            lot_size_e6,
            taker_fee_bps,
            maker_fee_bps,
            min_order_size_e6,
            max_order_size_e6,
        } => {
            msg!("Instruction: ProposeSpotMarket");
            process_propose_spot_market(
                program_id,
                accounts,
                nonce,
                symbol,
                base_token_index,
                quote_token_index,
                tick_size_e6,
                lot_size_e6,
                taker_fee_bps,
                maker_fee_bps,
                min_order_size_e6,
                max_order_size_e6,
            )
        }

        ListingInstruction::ObjectSpotMarket { stake_amount } => {
            msg!("Instruction: ObjectSpotMarket");
            process_object_spot_market(program_id, accounts, stake_amount)
        }

        ListingInstruction::ApproveSpotMarket => {
            msg!("Instruction: ApproveSpotMarket");
            process_approve_spot_market(program_id, accounts)
        }

        ListingInstruction::RejectSpotMarket {
            reason_code,
            slash_percentage,
        } => {
            msg!("Instruction: RejectSpotMarket");
            process_reject_spot_market(program_id, accounts, reason_code, slash_percentage)
        }

        ListingInstruction::CancelSpotMarketProposal => {
            msg!("Instruction: CancelSpotMarketProposal");
            process_cancel_spot_proposal(program_id, accounts)
        }

        ListingInstruction::FinalizeSpotMarket => {
            msg!("Instruction: FinalizeSpotMarket");
            process_finalize_spot_market(program_id, accounts)
        }

        ListingInstruction::ClaimSpotMarketStake => {
            msg!("Instruction: ClaimSpotMarketStake");
            process_claim_spot_stake(program_id, accounts)
        }

        ListingInstruction::UpdateSpotMarketStatus { is_active, is_paused } => {
            msg!("Instruction: UpdateSpotMarketStatus");
            process_update_spot_status(program_id, accounts, is_active, is_paused)
        }

        ListingInstruction::UpdateSpotMarketParams {
            taker_fee_bps,
            maker_fee_bps,
            min_order_size_e6,
            max_order_size_e6,
        } => {
            msg!("Instruction: UpdateSpotMarketParams");
            process_update_spot_params(
                program_id,
                accounts,
                taker_fee_bps,
                maker_fee_bps,
                min_order_size_e6,
                max_order_size_e6,
            )
        }

        // =====================================================================
        // PLP-3: Perp 市场上架指令
        // =====================================================================
        ListingInstruction::ProposePerpMarket {
            nonce,
            symbol,
            base_token_index,
            quote_token_index,
            oracle,
            tick_size_e6,
            lot_size_e6,
            max_leverage,
            initial_margin_rate_e6,
            maintenance_margin_rate_e6,
            taker_fee_bps,
            maker_fee_bps,
            min_order_size_e6,
            max_order_size_e6,
            max_open_interest_e6,
            insurance_fund_deposit_e6,
        } => {
            msg!("Instruction: ProposePerpMarket");
            process_propose_perp_market(
                program_id,
                accounts,
                nonce,
                symbol,
                base_token_index,
                quote_token_index,
                oracle,
                tick_size_e6,
                lot_size_e6,
                max_leverage,
                initial_margin_rate_e6,
                maintenance_margin_rate_e6,
                taker_fee_bps,
                maker_fee_bps,
                min_order_size_e6,
                max_order_size_e6,
                max_open_interest_e6,
                insurance_fund_deposit_e6,
            )
        }

        ListingInstruction::ObjectPerpMarket { stake_amount } => {
            msg!("Instruction: ObjectPerpMarket");
            process_object_perp_market(program_id, accounts, stake_amount)
        }

        ListingInstruction::ApprovePerpMarket => {
            msg!("Instruction: ApprovePerpMarket");
            process_approve_perp_market(program_id, accounts)
        }

        ListingInstruction::RejectPerpMarket {
            reason_code,
            slash_percentage,
        } => {
            msg!("Instruction: RejectPerpMarket");
            process_reject_perp_market(program_id, accounts, reason_code, slash_percentage)
        }

        ListingInstruction::CancelPerpMarketProposal => {
            msg!("Instruction: CancelPerpMarketProposal");
            process_cancel_perp_proposal(program_id, accounts)
        }

        ListingInstruction::FinalizePerpMarket => {
            msg!("Instruction: FinalizePerpMarket");
            process_finalize_perp_market(program_id, accounts)
        }

        ListingInstruction::ClaimPerpMarketStake => {
            msg!("Instruction: ClaimPerpMarketStake");
            process_claim_perp_stake(program_id, accounts)
        }

        ListingInstruction::UpdatePerpMarketStatus { is_active, is_paused } => {
            msg!("Instruction: UpdatePerpMarketStatus");
            process_update_perp_status(program_id, accounts, is_active, is_paused)
        }

        ListingInstruction::UpdatePerpMarketParams {
            max_leverage,
            initial_margin_rate_e6,
            maintenance_margin_rate_e6,
            taker_fee_bps,
            maker_fee_bps,
            max_open_interest_e6,
        } => {
            msg!("Instruction: UpdatePerpMarketParams");
            process_update_perp_params(
                program_id,
                accounts,
                max_leverage,
                initial_margin_rate_e6,
                maintenance_margin_rate_e6,
                taker_fee_bps,
                maker_fee_bps,
                max_open_interest_e6,
            )
        }

        // =====================================================================
        // PLP-4: 初始流动性池指令
        // =====================================================================
        ListingInstruction::InitializeLiquidityPool {
            market_type,
            price_lower_e6,
            price_upper_e6,
            order_density,
            spread_bps,
        } => {
            msg!("Instruction: InitializeLiquidityPool");
            process_initialize_liquidity_pool(
                program_id,
                accounts,
                market_type,
                price_lower_e6,
                price_upper_e6,
                order_density,
                spread_bps,
            )
        }

        ListingInstruction::FundLiquidityPool {
            base_amount_e6,
            quote_amount_e6,
        } => {
            msg!("Instruction: FundLiquidityPool");
            process_fund_liquidity_pool(program_id, accounts, base_amount_e6, quote_amount_e6)
        }

        ListingInstruction::AdjustLiquidityPoolParams {
            price_lower_e6,
            price_upper_e6,
            order_density,
            spread_bps,
        } => {
            msg!("Instruction: AdjustLiquidityPoolParams");
            process_adjust_liquidity_pool(
                program_id,
                accounts,
                price_lower_e6,
                price_upper_e6,
                order_density,
                spread_bps,
            )
        }

        ListingInstruction::RefreshLiquidityPoolOrders => {
            msg!("Instruction: RefreshLiquidityPoolOrders");
            process_refresh_liquidity_orders(program_id, accounts)
        }

        ListingInstruction::WithdrawLiquidityPoolProfit {
            base_amount_e6,
            quote_amount_e6,
        } => {
            msg!("Instruction: WithdrawLiquidityPoolProfit");
            process_withdraw_liquidity_profit(
                program_id,
                accounts,
                base_amount_e6,
                quote_amount_e6,
            )
        }

        ListingInstruction::RetireLiquidityPool => {
            msg!("Instruction: RetireLiquidityPool");
            process_retire_liquidity_pool(program_id, accounts)
        }

        // =====================================================================
        // Query 指令
        // =====================================================================
        ListingInstruction::QueryToken { token_index } => {
            msg!("Query: Token {}", token_index);
            Ok(())
        }

        ListingInstruction::QuerySpotMarket { market_index } => {
            msg!("Query: SpotMarket {}", market_index);
            Ok(())
        }

        ListingInstruction::QueryPerpMarket { market_index } => {
            msg!("Query: PerpMarket {}", market_index);
            Ok(())
        }
    }
}

// =============================================================================
// Admin 指令处理
// =============================================================================

fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vault_program: Pubkey,
    fund_program: Pubkey,
    ledger_program: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let treasury_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    // 验证 Admin 签名
    if !admin.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 验证 Config PDA
    let (expected_config, config_bump) = derive_listing_config_pda(program_id);
    if config_account.key != &expected_config {
        return Err(ListingError::InvalidPda.into());
    }

    // 验证 Treasury PDA
    let (expected_treasury, treasury_bump) = derive_treasury_pda(program_id);
    if treasury_account.key != &expected_treasury {
        msg!("Invalid treasury PDA");
        return Err(ListingError::InvalidPda.into());
    }

    // 检查是否已初始化
    if !config_account.data_is_empty() {
        return Err(ListingError::AlreadyInitialized.into());
    }

    // 创建 Config 账户
    create_account(
        admin,
        config_account,
        LISTING_CONFIG_SIZE,
        program_id,
        system_program,
        &[LISTING_CONFIG_SEED, &[config_bump]],
    )?;

    // 创建 Treasury PDA 账户（用于接收原生 N1024 质押）
    // Treasury 是一个空的 PDA 账户，只存储 lamports
    if treasury_account.data_is_empty() {
        // Treasury 只需要最小租金豁免余额
        let rent = Rent::get()?;
        let min_balance = rent.minimum_balance(0);
        
        create_account(
            admin,
            treasury_account,
            0, // 不需要存储数据，只存 lamports
            program_id,
            system_program,
            &[LISTING_TREASURY_SEED, &[treasury_bump]],
        )?;
        
        msg!("Treasury PDA created with {} lamports", min_balance);
    }

    // 初始化数据
    let config = ListingConfig {
        discriminator: ListingConfig::DISCRIMINATOR,
        version: 1,
        admin: *admin.key,
        treasury: *treasury_account.key,
        vault_program,
        fund_program,
        ledger_program,
        token_stake_amount: ListingConfig::DEFAULT_TOKEN_STAKE,
        spot_stake_amount: ListingConfig::DEFAULT_SPOT_STAKE,
        perp_stake_amount: ListingConfig::DEFAULT_PERP_STAKE,
        token_review_period_seconds: ListingConfig::DEFAULT_TOKEN_REVIEW_PERIOD,
        spot_review_period_seconds: ListingConfig::DEFAULT_SPOT_REVIEW_PERIOD,
        perp_review_period_seconds: ListingConfig::DEFAULT_PERP_REVIEW_PERIOD,
        stake_lock_period_seconds: ListingConfig::DEFAULT_STAKE_LOCK_PERIOD,
        total_tokens: 0,
        total_spot_markets: 0,
        total_perp_markets: 0,
        total_staked_lamports: 0,
        is_paused: false,
        bump: config_bump,
        reserved: [0u8; 64],
    };

    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("ListingConfig initialized (Native N1024 Staking)");
    msg!("Admin: {}", admin.key);
    msg!("Treasury: {}", treasury_account.key);
    msg!("Token Stake: {} N1024", ListingConfig::DEFAULT_TOKEN_STAKE / 1_000_000_000);
    msg!("Spot Stake: {} N1024", ListingConfig::DEFAULT_SPOT_STAKE / 1_000_000_000);
    msg!("Perp Stake: {} N1024", ListingConfig::DEFAULT_PERP_STAKE / 1_000_000_000);

    Ok(())
}

fn process_update_admin(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_admin: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;

    // 反序列化
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    
    // 验证 Admin
    verify_admin(admin, &config)?;

    // 更新 Admin
    config.admin = new_admin;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("Admin updated to: {}", new_admin);

    Ok(())
}

fn process_update_stake_config(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_stake_amount: Option<u64>,
    spot_stake_amount: Option<u64>,
    perp_stake_amount: Option<u64>,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;

    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    verify_admin(admin, &config)?;

    if let Some(amount) = token_stake_amount {
        config.token_stake_amount = amount;
    }
    if let Some(amount) = spot_stake_amount {
        config.spot_stake_amount = amount;
    }
    if let Some(amount) = perp_stake_amount {
        config.perp_stake_amount = amount;
    }

    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("Stake config updated");

    Ok(())
}

fn process_update_review_periods(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_review_period: Option<u32>,
    spot_review_period: Option<u32>,
    perp_review_period: Option<u32>,
    stake_lock_period: Option<u32>,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;

    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    verify_admin(admin, &config)?;

    if let Some(period) = token_review_period {
        config.token_review_period_seconds = period;
    }
    if let Some(period) = spot_review_period {
        config.spot_review_period_seconds = period;
    }
    if let Some(period) = perp_review_period {
        config.perp_review_period_seconds = period;
    }
    if let Some(period) = stake_lock_period {
        config.stake_lock_period_seconds = period;
    }

    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("Review periods updated");

    Ok(())
}

fn process_set_paused(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    paused: bool,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;

    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    verify_admin(admin, &config)?;

    config.is_paused = paused;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("Listing paused: {}", paused);

    Ok(())
}

// =============================================================================
// PLP-1: Token 注册指令处理
// =============================================================================

fn process_propose_token(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    nonce: u64,
    symbol: [u8; 8],
    mint: Pubkey,
    decimals: u8,
    oracle: Option<Pubkey>,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let proposer = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let treasury_account = next_account_info(account_iter)?;
    let token_mint = next_account_info(account_iter)?;
    let _oracle_account = next_account_info(account_iter).ok(); // 可选
    let system_program = next_account_info(account_iter)?;

    // 验证签名
    if !proposer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    
    // 检查是否暂停
    if config.is_paused {
        return Err(ListingError::ListingPaused.into());
    }

    // 验证 Treasury PDA
    if treasury_account.key != &config.treasury {
        msg!("Invalid treasury account");
        return Err(ListingError::InvalidAccount.into());
    }

    // 验证 Symbol
    validate_symbol(&symbol)?;
    validate_decimals(decimals)?;

    // 验证 Mint 存在
    if token_mint.key != &mint {
        return Err(ListingError::InvalidAccount.into());
    }

    // 验证 Proposal PDA
    let (expected_proposal, bump) = derive_token_proposal_pda(proposer.key, nonce, program_id);
    if proposal_account.key != &expected_proposal {
        return Err(ListingError::InvalidPda.into());
    }

    // 检查是否已初始化
    if !proposal_account.data_is_empty() {
        return Err(ListingError::AlreadyInitialized.into());
    }

    // 检查 proposer 余额是否足够
    let stake_amount = config.token_stake_amount;
    if proposer.lamports() < stake_amount {
        msg!("Insufficient balance. Required: {} lamports ({} N1024)", 
             stake_amount, stake_amount / 1_000_000_000);
        return Err(ListingError::InsufficientStake.into());
    }

    // 转移原生 N1024 质押 (lamports)
    transfer_native_lamports(
        proposer,
        treasury_account,
        stake_amount,
        system_program,
        None,
    )?;

    // 更新配置中的总质押额
    config.total_staked_lamports = config.total_staked_lamports
        .checked_add(stake_amount)
        .ok_or(ListingError::Overflow)?;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("Stake transferred: {} N1024", stake_amount / 1_000_000_000);

    // 创建 Proposal 账户
    create_account(
        proposer,
        proposal_account,
        TOKEN_PROPOSAL_SIZE,
        program_id,
        system_program,
        &[
            TOKEN_PROPOSAL_SEED,
            proposer.key.as_ref(),
            &nonce.to_le_bytes(),
            &[bump],
        ],
    )?;

    // 初始化数据
    let current_ts = get_current_timestamp()?;
    let review_deadline = current_ts + config.token_review_period_seconds as i64;

    let proposal = TokenProposal {
        discriminator: TokenProposal::DISCRIMINATOR,
        version: 1,
        proposer: *proposer.key,
        nonce,
        symbol,
        mint,
        decimals,
        oracle,
        stake_amount: config.token_stake_amount,
        status: ProposalStatus::Pending,
        created_at: current_ts,
        review_deadline,
        objection_count: 0,
        objection_stake: 0,
        stake_claimed: false,
        bump,
        reserved: [0u8; 64],
    };

    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    msg!("Token proposal created");
    msg!("Proposer: {}", proposer.key);
    msg!("Symbol: {:?}", std::str::from_utf8(&symbol).unwrap_or(""));
    msg!("Mint: {}", mint);
    msg!("Stake: {} 1024", config.token_stake_amount / 1_000_000);
    msg!("Review deadline: {}", review_deadline);

    Ok(())
}

fn process_object_token(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    stake_amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let objector = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let treasury_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    // 验证签名
    if !objector.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;

    // 验证 Treasury
    if treasury_account.key != &config.treasury {
        return Err(ListingError::InvalidAccount.into());
    }

    // 加载提案
    let mut proposal = TokenProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证状态
    if proposal.status != ProposalStatus::Pending {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 验证未过期
    let current_ts = get_current_timestamp()?;
    if current_ts > proposal.review_deadline {
        return Err(ListingError::ReviewDeadlinePassed.into());
    }

    // 检查余额
    if objector.lamports() < stake_amount {
        msg!("Insufficient balance for objection stake");
        return Err(ListingError::InsufficientStake.into());
    }

    // 转移原生 N1024 反对质押
    transfer_native_lamports(
        objector,
        treasury_account,
        stake_amount,
        system_program,
        None,
    )?;

    // 更新总质押
    config.total_staked_lamports = config.total_staked_lamports
        .checked_add(stake_amount)
        .ok_or(ListingError::Overflow)?;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    // 更新反对信息
    proposal.objection_count = proposal.objection_count.checked_add(1)
        .ok_or(ListingError::Overflow)?;
    proposal.objection_stake = proposal.objection_stake.checked_add(stake_amount)
        .ok_or(ListingError::Overflow)?;

    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    msg!("Token objection recorded");
    msg!("Objector: {}", objector.key);
    msg!("Stake: {} N1024", stake_amount / 1_000_000_000);
    msg!("Total objections: {}", proposal.objection_count);

    Ok(())
}

fn process_approve_token(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let registry_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    // 加载配置并验证 Admin
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    verify_admin(admin, &config)?;

    // 加载提案
    let mut proposal = TokenProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证状态
    if proposal.status != ProposalStatus::Pending {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 分配 token_index
    let token_index = config.total_tokens;
    config.total_tokens = config.total_tokens.checked_add(1)
        .ok_or(ListingError::Overflow)?;

    // 验证 Registry PDA
    let (expected_registry, bump) = derive_token_registry_pda(token_index, program_id);
    if registry_account.key != &expected_registry {
        return Err(ListingError::InvalidPda.into());
    }

    // 创建 Registry 账户
    create_account(
        admin,
        registry_account,
        TOKEN_REGISTRY_SIZE,
        program_id,
        system_program,
        &[TOKEN_REGISTRY_SEED, &token_index.to_le_bytes(), &[bump]],
    )?;

    // 初始化 Registry
    let current_ts = get_current_timestamp()?;
    let registry = TokenRegistry {
        discriminator: TokenRegistry::DISCRIMINATOR,
        version: 1,
        token_index,
        symbol: proposal.symbol,
        mint: proposal.mint,
        decimals: proposal.decimals,
        oracle: proposal.oracle,
        is_active: true,
        proposer: proposal.proposer,
        approved_at: current_ts,
        bump,
        reserved: [0u8; 64],
    };

    registry.serialize(&mut &mut registry_account.data.borrow_mut()[..])?;

    // 更新提案状态
    proposal.status = ProposalStatus::Approved;
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    // 保存配置
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("Token approved");
    msg!("Token index: {}", token_index);
    msg!("Symbol: {:?}", std::str::from_utf8(&proposal.symbol).unwrap_or(""));
    msg!("Mint: {}", proposal.mint);

    Ok(())
}

fn process_reject_token(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    reason_code: u8,
    slash_percentage: u8,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let _treasury_account = next_account_info(account_iter)?;

    // 加载配置并验证 Admin
    let config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    verify_admin(admin, &config)?;

    // 加载提案
    let mut proposal = TokenProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证状态
    if proposal.status != ProposalStatus::Pending {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 验证罚没比例
    if slash_percentage > 100 {
        return Err(ListingError::InvalidAmount.into());
    }

    // 更新状态
    proposal.status = ProposalStatus::Rejected;
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    // 计算罚没金额（保留在 treasury 中）
    let slash_amount = proposal.stake_amount * slash_percentage as u64 / 100;
    
    msg!("Token rejected");
    msg!("Reason code: {}", reason_code);
    msg!("Slash percentage: {}%", slash_percentage);
    msg!("Slash amount: {} 1024", slash_amount / 1_000_000);

    Ok(())
}

fn process_cancel_token_proposal(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let proposer = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let treasury_account = next_account_info(account_iter)?;

    // 验证签名
    if !proposer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;

    // 验证 Treasury
    if treasury_account.key != &config.treasury {
        return Err(ListingError::InvalidAccount.into());
    }

    // 加载提案
    let mut proposal = TokenProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证 Proposer
    if &proposal.proposer != proposer.key {
        return Err(ListingError::NotProposer.into());
    }

    // 验证状态
    if proposal.status != ProposalStatus::Pending {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 更新状态
    proposal.status = ProposalStatus::Cancelled;
    
    // 计算退还金额（扣除 5%）
    let refund = proposal.stake_amount * 95 / 100;
    let slash = proposal.stake_amount - refund;
    
    // 从 Treasury PDA 退还原生 N1024 (直接操作 lamports)
    transfer_lamports_from_pda(
        treasury_account,
        proposer,
        refund,
    )?;

    // 更新总质押额（扣除退还部分）
    config.total_staked_lamports = config.total_staked_lamports
        .checked_sub(refund)
        .ok_or(ListingError::Overflow)?;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    msg!("Token proposal cancelled");
    msg!("Refund: {} N1024 (95%)", refund / 1_000_000_000);
    msg!("Slashed: {} N1024 (5%)", slash / 1_000_000_000);

    Ok(())
}

fn process_finalize_token(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let caller = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let registry_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    // 验证签名
    if !caller.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;

    // 加载提案
    let mut proposal = TokenProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证状态
    if proposal.status != ProposalStatus::Pending {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 验证已过审核期
    let current_ts = get_current_timestamp()?;
    if current_ts <= proposal.review_deadline {
        return Err(ListingError::ReviewDeadlineNotReached.into());
    }

    // 检查是否有反对
    if proposal.objection_count > 0 {
        msg!("Cannot auto-approve: {} objections pending", proposal.objection_count);
        return Err(ListingError::ProposalNotPending.into());
    }

    // 分配 token_index
    let token_index = config.total_tokens;
    config.total_tokens = config.total_tokens.checked_add(1)
        .ok_or(ListingError::Overflow)?;

    // 验证 Registry PDA
    let (expected_registry, bump) = derive_token_registry_pda(token_index, program_id);
    if registry_account.key != &expected_registry {
        return Err(ListingError::InvalidPda.into());
    }

    // 创建 Registry 账户
    create_account(
        caller,
        registry_account,
        TOKEN_REGISTRY_SIZE,
        program_id,
        system_program,
        &[TOKEN_REGISTRY_SEED, &token_index.to_le_bytes(), &[bump]],
    )?;

    // 初始化 Registry
    let registry = TokenRegistry {
        discriminator: TokenRegistry::DISCRIMINATOR,
        version: 1,
        token_index,
        symbol: proposal.symbol,
        mint: proposal.mint,
        decimals: proposal.decimals,
        oracle: proposal.oracle,
        is_active: true,
        proposer: proposal.proposer,
        approved_at: current_ts,
        bump,
        reserved: [0u8; 64],
    };

    registry.serialize(&mut &mut registry_account.data.borrow_mut()[..])?;

    // 更新提案状态
    proposal.status = ProposalStatus::Approved;
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    // 保存配置
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("Token auto-approved (finalized)");
    msg!("Token index: {}", token_index);

    Ok(())
}

fn process_claim_token_stake(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let proposer = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let registry_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let treasury_account = next_account_info(account_iter)?;

    // 验证签名
    if !proposer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;

    // 验证 Treasury
    if treasury_account.key != &config.treasury {
        return Err(ListingError::InvalidAccount.into());
    }

    // 加载提案
    let mut proposal = TokenProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证 Proposer
    if &proposal.proposer != proposer.key {
        return Err(ListingError::NotProposer.into());
    }

    // 验证状态已批准
    if proposal.status != ProposalStatus::Approved {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 验证未取回
    if proposal.stake_claimed {
        return Err(ListingError::StakeAlreadyClaimed.into());
    }

    // 验证 Registry 存在
    let registry = TokenRegistry::try_from_slice(&registry_account.data.borrow())?;
    if registry.proposer != *proposer.key {
        return Err(ListingError::NotProposer.into());
    }

    // 验证锁定期已过
    let current_ts = get_current_timestamp()?;
    let unlock_time = registry.approved_at + config.stake_lock_period_seconds as i64;
    if current_ts < unlock_time {
        msg!("Stake locked until: {}", unlock_time);
        return Err(ListingError::StakeLockPeriodNotEnded.into());
    }

    // 从 Treasury PDA 退还原生 N1024
    transfer_lamports_from_pda(
        treasury_account,
        proposer,
        proposal.stake_amount,
    )?;

    // 更新总质押额
    config.total_staked_lamports = config.total_staked_lamports
        .checked_sub(proposal.stake_amount)
        .ok_or(ListingError::Overflow)?;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    // 标记已取回
    proposal.stake_claimed = true;
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    msg!("Token stake claimed");
    msg!("Amount: {} N1024", proposal.stake_amount / 1_000_000_000);

    Ok(())
}

fn process_update_token_status(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    is_active: bool,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let registry_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;

    // 加载配置并验证 Admin
    let config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    verify_admin(admin, &config)?;

    // 加载 Registry
    let mut registry = TokenRegistry::try_from_slice(&registry_account.data.borrow())?;
    
    // 更新状态
    registry.is_active = is_active;
    registry.serialize(&mut &mut registry_account.data.borrow_mut()[..])?;

    msg!("Token status updated: is_active = {}", is_active);

    Ok(())
}

// =============================================================================
// PLP-2: Spot 市场上架指令处理（占位）
// =============================================================================

fn process_propose_spot_market(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    nonce: u64,
    symbol: [u8; 16],
    base_token_index: u16,
    quote_token_index: u16,
    tick_size_e6: u64,
    lot_size_e6: u64,
    taker_fee_bps: u16,
    maker_fee_bps: i16,
    min_order_size_e6: u64,
    max_order_size_e6: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let proposer = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let base_token_registry = next_account_info(account_iter)?;
    let quote_token_registry = next_account_info(account_iter)?;
    let treasury_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    // 验证签名
    if !proposer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    
    // 检查是否暂停
    if config.is_paused {
        return Err(ListingError::ListingPaused.into());
    }

    // 验证 Treasury
    if treasury_account.key != &config.treasury {
        return Err(ListingError::InvalidAccount.into());
    }

    // 验证 Symbol 格式 (必须包含 '/')
    validate_market_symbol(&symbol, true)?;

    // 验证 Base Token 已注册
    let base_registry = TokenRegistry::try_from_slice(&base_token_registry.data.borrow())?;
    if base_registry.token_index != base_token_index || !base_registry.is_active {
        return Err(ListingError::TokenNotRegistered.into());
    }

    // 验证 Quote Token 已注册
    let quote_registry = TokenRegistry::try_from_slice(&quote_token_registry.data.borrow())?;
    if quote_registry.token_index != quote_token_index || !quote_registry.is_active {
        return Err(ListingError::TokenNotRegistered.into());
    }

    // 验证 Base 和 Quote 不同
    if base_token_index == quote_token_index {
        return Err(ListingError::SameTokenPair.into());
    }

    // 验证参数
    validate_sizes(tick_size_e6, lot_size_e6)?;
    validate_fee_rates(taker_fee_bps, maker_fee_bps)?;

    // 验证 Proposal PDA
    let (expected_proposal, bump) = derive_spot_proposal_pda(proposer.key, nonce, program_id);
    if proposal_account.key != &expected_proposal {
        return Err(ListingError::InvalidPda.into());
    }

    // 检查是否已初始化
    if !proposal_account.data_is_empty() {
        return Err(ListingError::AlreadyInitialized.into());
    }

    // 检查余额
    let stake_amount = config.spot_stake_amount;
    if proposer.lamports() < stake_amount {
        msg!("Insufficient balance. Required: {} N1024", stake_amount / 1_000_000_000);
        return Err(ListingError::InsufficientStake.into());
    }

    // 转移原生 N1024 质押
    transfer_native_lamports(
        proposer,
        treasury_account,
        stake_amount,
        system_program,
        None,
    )?;

    // 更新总质押
    config.total_staked_lamports = config.total_staked_lamports
        .checked_add(stake_amount)
        .ok_or(ListingError::Overflow)?;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    // 创建 Proposal 账户
    create_account(
        proposer,
        proposal_account,
        SPOT_PROPOSAL_SIZE,
        program_id,
        system_program,
        &[
            SPOT_PROPOSAL_SEED,
            proposer.key.as_ref(),
            &nonce.to_le_bytes(),
            &[bump],
        ],
    )?;

    // 初始化数据
    let current_ts = get_current_timestamp()?;
    let review_deadline = current_ts + config.spot_review_period_seconds as i64;

    let proposal = SpotMarketProposal {
        discriminator: SpotMarketProposal::DISCRIMINATOR,
        version: 1,
        proposer: *proposer.key,
        nonce,
        symbol,
        base_token_index,
        quote_token_index,
        tick_size_e6,
        lot_size_e6,
        taker_fee_bps,
        maker_fee_bps,
        min_order_size_e6,
        max_order_size_e6,
        stake_amount,
        status: ProposalStatus::Pending,
        created_at: current_ts,
        review_deadline,
        objection_count: 0,
        objection_stake: 0,
        stake_claimed: false,
        bump,
        reserved: [0u8; 64],
    };

    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    msg!("Spot market proposal created");
    msg!("Proposer: {}", proposer.key);
    msg!("Symbol: {:?}", std::str::from_utf8(&symbol).unwrap_or(""));
    msg!("Base Token: {} / Quote Token: {}", base_token_index, quote_token_index);
    msg!("Stake: {} N1024", stake_amount / 1_000_000_000);

    Ok(())
}

fn process_object_spot_market(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    stake_amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let objector = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let treasury_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    // 验证签名
    if !objector.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;

    // 验证 Treasury
    if treasury_account.key != &config.treasury {
        return Err(ListingError::InvalidAccount.into());
    }

    // 加载提案
    let mut proposal = SpotMarketProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证状态
    if proposal.status != ProposalStatus::Pending {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 验证未过期
    let current_ts = get_current_timestamp()?;
    if current_ts > proposal.review_deadline {
        return Err(ListingError::ReviewDeadlinePassed.into());
    }

    // 检查余额
    if objector.lamports() < stake_amount {
        msg!("Insufficient balance for objection stake");
        return Err(ListingError::InsufficientStake.into());
    }

    // 转移原生 N1024 反对质押
    transfer_native_lamports(
        objector,
        treasury_account,
        stake_amount,
        system_program,
        None,
    )?;

    // 更新总质押
    config.total_staked_lamports = config.total_staked_lamports
        .checked_add(stake_amount)
        .ok_or(ListingError::Overflow)?;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    // 更新反对信息
    proposal.objection_count = proposal.objection_count.checked_add(1)
        .ok_or(ListingError::Overflow)?;
    proposal.objection_stake = proposal.objection_stake.checked_add(stake_amount)
        .ok_or(ListingError::Overflow)?;

    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    msg!("Spot market objection recorded");
    msg!("Objector: {}", objector.key);
    msg!("Stake: {} N1024", stake_amount / 1_000_000_000);
    msg!("Total objections: {}", proposal.objection_count);

    Ok(())
}

fn process_approve_spot_market(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let market_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    // 加载配置并验证 Admin
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    verify_admin(admin, &config)?;

    // 加载提案
    let mut proposal = SpotMarketProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证状态
    if proposal.status != ProposalStatus::Pending {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 分配 market_index
    let market_index = config.total_spot_markets;
    config.total_spot_markets = config.total_spot_markets.checked_add(1)
        .ok_or(ListingError::Overflow)?;

    // 验证 Market PDA
    let (expected_market, bump) = derive_spot_market_pda(market_index, program_id);
    if market_account.key != &expected_market {
        return Err(ListingError::InvalidPda.into());
    }

    // 创建 Market 账户
    create_account(
        admin,
        market_account,
        SPOT_MARKET_SIZE,
        program_id,
        system_program,
        &[SPOT_MARKET_SEED, &market_index.to_le_bytes(), &[bump]],
    )?;

    // 初始化 Market
    let current_ts = get_current_timestamp()?;
    let market = SpotMarket {
        discriminator: SpotMarket::DISCRIMINATOR,
        version: 1,
        market_index,
        symbol: proposal.symbol,
        base_token_index: proposal.base_token_index,
        quote_token_index: proposal.quote_token_index,
        tick_size_e6: proposal.tick_size_e6,
        lot_size_e6: proposal.lot_size_e6,
        taker_fee_bps: proposal.taker_fee_bps,
        maker_fee_bps: proposal.maker_fee_bps,
        min_order_size_e6: proposal.min_order_size_e6,
        max_order_size_e6: proposal.max_order_size_e6,
        is_active: true,
        is_paused: false,
        proposer: proposal.proposer,
        approved_at: current_ts,
        bump,
        reserved: [0u8; 64],
    };

    market.serialize(&mut &mut market_account.data.borrow_mut()[..])?;

    // 更新提案状态
    proposal.status = ProposalStatus::Approved;
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    // 保存配置
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("Spot market approved");
    msg!("Market index: {}", market_index);
    msg!("Symbol: {:?}", std::str::from_utf8(&proposal.symbol).unwrap_or(""));

    Ok(())
}

fn process_reject_spot_market(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    reason_code: u8,
    slash_percentage: u8,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let _treasury_account = next_account_info(account_iter)?;

    // 加载配置并验证 Admin
    let config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    verify_admin(admin, &config)?;

    // 加载提案
    let mut proposal = SpotMarketProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证状态
    if proposal.status != ProposalStatus::Pending {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 验证罚没比例
    if slash_percentage > 100 {
        return Err(ListingError::InvalidAmount.into());
    }

    // 更新状态
    proposal.status = ProposalStatus::Rejected;
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    // 计算罚没金额（保留在 treasury 中）
    let slash_amount = proposal.stake_amount * slash_percentage as u64 / 100;
    
    msg!("Spot market rejected");
    msg!("Reason code: {}", reason_code);
    msg!("Slash percentage: {}%", slash_percentage);
    msg!("Slash amount: {} N1024", slash_amount / 1_000_000_000);

    Ok(())
}

fn process_cancel_spot_proposal(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let proposer = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let treasury_account = next_account_info(account_iter)?;

    // 验证签名
    if !proposer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;

    // 验证 Treasury
    if treasury_account.key != &config.treasury {
        return Err(ListingError::InvalidAccount.into());
    }

    // 加载提案
    let mut proposal = SpotMarketProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证 Proposer
    if &proposal.proposer != proposer.key {
        return Err(ListingError::NotProposer.into());
    }

    // 验证状态
    if proposal.status != ProposalStatus::Pending {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 更新状态
    proposal.status = ProposalStatus::Cancelled;
    
    // 计算退还金额（扣除 5%）
    let refund = proposal.stake_amount * 95 / 100;
    let slash = proposal.stake_amount - refund;
    
    // 从 Treasury PDA 退还原生 N1024
    transfer_lamports_from_pda(
        treasury_account,
        proposer,
        refund,
    )?;

    // 更新总质押额（扣除退还部分）
    config.total_staked_lamports = config.total_staked_lamports
        .checked_sub(refund)
        .ok_or(ListingError::Overflow)?;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    msg!("Spot market proposal cancelled");
    msg!("Refund: {} N1024 (95%)", refund / 1_000_000_000);
    msg!("Slashed: {} N1024 (5%)", slash / 1_000_000_000);

    Ok(())
}

fn process_finalize_spot_market(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let caller = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let market_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    // 验证签名
    if !caller.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;

    // 加载提案
    let mut proposal = SpotMarketProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证状态
    if proposal.status != ProposalStatus::Pending {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 验证已过审核期
    let current_ts = get_current_timestamp()?;
    if current_ts <= proposal.review_deadline {
        return Err(ListingError::ReviewDeadlineNotReached.into());
    }

    // 检查是否有反对（如果反对数过多则不能自动批准）
    if proposal.objection_count > 0 && proposal.objection_stake > proposal.stake_amount / 2 {
        msg!("Cannot auto-approve: significant objections (stake: {} vs proposal: {})", 
             proposal.objection_stake, proposal.stake_amount);
        return Err(ListingError::ProposalNotPending.into());
    }

    // 分配 market_index
    let market_index = config.total_spot_markets;
    config.total_spot_markets = config.total_spot_markets.checked_add(1)
        .ok_or(ListingError::Overflow)?;

    // 验证 Market PDA
    let (expected_market, bump) = derive_spot_market_pda(market_index, program_id);
    if market_account.key != &expected_market {
        return Err(ListingError::InvalidPda.into());
    }

    // 创建 Market 账户
    create_account(
        caller,
        market_account,
        SPOT_MARKET_SIZE,
        program_id,
        system_program,
        &[SPOT_MARKET_SEED, &market_index.to_le_bytes(), &[bump]],
    )?;

    // 初始化 Market
    let market = SpotMarket {
        discriminator: SpotMarket::DISCRIMINATOR,
        version: 1,
        market_index,
        symbol: proposal.symbol,
        base_token_index: proposal.base_token_index,
        quote_token_index: proposal.quote_token_index,
        tick_size_e6: proposal.tick_size_e6,
        lot_size_e6: proposal.lot_size_e6,
        taker_fee_bps: proposal.taker_fee_bps,
        maker_fee_bps: proposal.maker_fee_bps,
        min_order_size_e6: proposal.min_order_size_e6,
        max_order_size_e6: proposal.max_order_size_e6,
        is_active: true,
        is_paused: false,
        proposer: proposal.proposer,
        approved_at: current_ts,
        bump,
        reserved: [0u8; 64],
    };

    market.serialize(&mut &mut market_account.data.borrow_mut()[..])?;

    // 更新提案状态
    proposal.status = ProposalStatus::Approved;
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    // 保存配置
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("Spot market auto-approved (finalized)");
    msg!("Market index: {}", market_index);
    msg!("Symbol: {:?}", std::str::from_utf8(&proposal.symbol).unwrap_or(""));

    Ok(())
}

fn process_claim_spot_stake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let proposer = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let market_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let treasury_account = next_account_info(account_iter)?;

    // 验证签名
    if !proposer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;

    // 验证 Treasury
    if treasury_account.key != &config.treasury {
        return Err(ListingError::InvalidAccount.into());
    }

    // 加载提案
    let mut proposal = SpotMarketProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证 Proposer
    if &proposal.proposer != proposer.key {
        return Err(ListingError::NotProposer.into());
    }

    // 验证状态已批准
    if proposal.status != ProposalStatus::Approved {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 验证未取回
    if proposal.stake_claimed {
        return Err(ListingError::StakeAlreadyClaimed.into());
    }

    // 验证 Market 存在（通过检查 proposer 匹配）
    let market = SpotMarket::try_from_slice(&market_account.data.borrow())?;
    if market.proposer != *proposer.key {
        return Err(ListingError::NotProposer.into());
    }

    // 验证锁定期已过
    let current_ts = get_current_timestamp()?;
    let unlock_time = market.approved_at + config.stake_lock_period_seconds as i64;
    if current_ts < unlock_time {
        msg!("Stake locked until: {}", unlock_time);
        return Err(ListingError::StakeLockPeriodNotEnded.into());
    }

    // 从 Treasury PDA 退还原生 N1024
    transfer_lamports_from_pda(
        treasury_account,
        proposer,
        proposal.stake_amount,
    )?;

    // 更新总质押额
    config.total_staked_lamports = config.total_staked_lamports
        .checked_sub(proposal.stake_amount)
        .ok_or(ListingError::Overflow)?;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    // 标记已取回
    proposal.stake_claimed = true;
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    msg!("Spot market stake claimed");
    msg!("Amount: {} N1024", proposal.stake_amount / 1_000_000_000);

    Ok(())
}

fn process_update_spot_status(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    is_active: Option<bool>,
    is_paused: Option<bool>,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let market_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;

    // 加载配置并验证 Admin
    let config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    verify_admin(admin, &config)?;

    // 加载 Market
    let mut market = SpotMarket::try_from_slice(&market_account.data.borrow())?;
    
    // 更新状态
    if let Some(active) = is_active {
        market.is_active = active;
        msg!("Spot market is_active updated to: {}", active);
    }
    if let Some(paused) = is_paused {
        market.is_paused = paused;
        msg!("Spot market is_paused updated to: {}", paused);
    }

    market.serialize(&mut &mut market_account.data.borrow_mut()[..])?;

    msg!("Spot market status updated");
    msg!("Market index: {}", market.market_index);

    Ok(())
}

fn process_update_spot_params(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    taker_fee_bps: Option<u16>,
    maker_fee_bps: Option<i16>,
    min_order_size_e6: Option<u64>,
    max_order_size_e6: Option<u64>,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let market_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;

    // 加载配置并验证 Admin
    let config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    verify_admin(admin, &config)?;

    // 加载 Market
    let mut market = SpotMarket::try_from_slice(&market_account.data.borrow())?;
    
    // 更新参数
    if let Some(fee) = taker_fee_bps {
        if fee > 1000 {
            return Err(ListingError::InvalidFeeRate.into());
        }
        market.taker_fee_bps = fee;
        msg!("Taker fee updated to: {} bps", fee);
    }
    if let Some(fee) = maker_fee_bps {
        if fee < -500 || fee > 500 {
            return Err(ListingError::InvalidFeeRate.into());
        }
        market.maker_fee_bps = fee;
        msg!("Maker fee updated to: {} bps", fee);
    }
    if let Some(size) = min_order_size_e6 {
        market.min_order_size_e6 = size;
        msg!("Min order size updated to: {} (e6)", size);
    }
    if let Some(size) = max_order_size_e6 {
        market.max_order_size_e6 = size;
        msg!("Max order size updated to: {} (e6)", size);
    }

    market.serialize(&mut &mut market_account.data.borrow_mut()[..])?;

    msg!("Spot market params updated");
    msg!("Market index: {}", market.market_index);

    Ok(())
}

// =============================================================================
// PLP-3: Perp 市场上架指令处理（占位）
// =============================================================================

fn process_propose_perp_market(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    nonce: u64,
    symbol: [u8; 16],
    base_token_index: u16,
    quote_token_index: u16,
    oracle: Pubkey,
    tick_size_e6: u64,
    lot_size_e6: u64,
    max_leverage: u8,
    initial_margin_rate_e6: u32,
    maintenance_margin_rate_e6: u32,
    taker_fee_bps: u16,
    maker_fee_bps: i16,
    min_order_size_e6: u64,
    max_order_size_e6: u64,
    max_open_interest_e6: u64,
    insurance_fund_deposit_e6: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let proposer = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let base_token_registry = next_account_info(account_iter)?;
    let quote_token_registry = next_account_info(account_iter)?;
    let oracle_account = next_account_info(account_iter)?;
    let treasury_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    // 验证签名
    if !proposer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    
    // 检查是否暂停
    if config.is_paused {
        return Err(ListingError::ListingPaused.into());
    }

    // 验证 Treasury
    if treasury_account.key != &config.treasury {
        return Err(ListingError::InvalidAccount.into());
    }

    // 验证 Symbol 格式 (必须包含 '-')
    validate_market_symbol(&symbol, false)?;

    // 验证 Base Token 已注册
    let base_registry = TokenRegistry::try_from_slice(&base_token_registry.data.borrow())?;
    if base_registry.token_index != base_token_index || !base_registry.is_active {
        return Err(ListingError::TokenNotRegistered.into());
    }

    // 验证 Quote Token 已注册
    let quote_registry = TokenRegistry::try_from_slice(&quote_token_registry.data.borrow())?;
    if quote_registry.token_index != quote_token_index || !quote_registry.is_active {
        return Err(ListingError::TokenNotRegistered.into());
    }

    // 验证 Base 和 Quote 不同
    if base_token_index == quote_token_index {
        return Err(ListingError::SameTokenPair.into());
    }

    // 验证 Oracle
    if oracle_account.key != &oracle {
        return Err(ListingError::InvalidOracle.into());
    }
    
    // Pyth Oracle 验证
    validate_oracle_exists(oracle_account)?;

    // 验证参数
    validate_sizes(tick_size_e6, lot_size_e6)?;
    validate_fee_rates(taker_fee_bps, maker_fee_bps)?;
    validate_leverage(max_leverage)?;
    validate_margin_rates(initial_margin_rate_e6, maintenance_margin_rate_e6)?;

    // 验证 Proposal PDA
    let (expected_proposal, bump) = derive_perp_proposal_pda(proposer.key, nonce, program_id);
    if proposal_account.key != &expected_proposal {
        return Err(ListingError::InvalidPda.into());
    }

    // 检查是否已初始化
    if !proposal_account.data_is_empty() {
        return Err(ListingError::AlreadyInitialized.into());
    }

    // 检查余额
    let stake_amount = config.perp_stake_amount;
    if proposer.lamports() < stake_amount {
        msg!("Insufficient balance. Required: {} N1024", stake_amount / 1_000_000_000);
        return Err(ListingError::InsufficientStake.into());
    }

    // 转移原生 N1024 质押
    transfer_native_lamports(
        proposer,
        treasury_account,
        stake_amount,
        system_program,
        None,
    )?;

    // 更新总质押
    config.total_staked_lamports = config.total_staked_lamports
        .checked_add(stake_amount)
        .ok_or(ListingError::Overflow)?;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    // 创建 Proposal 账户
    create_account(
        proposer,
        proposal_account,
        PERP_PROPOSAL_SIZE,
        program_id,
        system_program,
        &[
            PERP_PROPOSAL_SEED,
            proposer.key.as_ref(),
            &nonce.to_le_bytes(),
            &[bump],
        ],
    )?;

    // 初始化数据
    let current_ts = get_current_timestamp()?;
    let review_deadline = current_ts + config.perp_review_period_seconds as i64;

    let proposal = PerpMarketProposal {
        discriminator: PerpMarketProposal::DISCRIMINATOR,
        version: 1,
        proposer: *proposer.key,
        nonce,
        symbol,
        base_token_index,
        quote_token_index,
        oracle,
        tick_size_e6,
        lot_size_e6,
        max_leverage,
        initial_margin_rate_e6,
        maintenance_margin_rate_e6,
        taker_fee_bps,
        maker_fee_bps,
        min_order_size_e6,
        max_order_size_e6,
        max_open_interest_e6,
        insurance_fund_deposit_e6,
        stake_amount,
        status: ProposalStatus::Pending,
        created_at: current_ts,
        review_deadline,
        objection_count: 0,
        objection_stake: 0,
        stake_claimed: false,
        bump,
        reserved: [0u8; 64],
    };

    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    msg!("Perp market proposal created");
    msg!("Proposer: {}", proposer.key);
    msg!("Symbol: {:?}", std::str::from_utf8(&symbol).unwrap_or(""));
    msg!("Oracle: {}", oracle);
    msg!("Max Leverage: {}x", max_leverage);
    msg!("Stake: {} N1024", stake_amount / 1_000_000_000);

    Ok(())
}

fn process_object_perp_market(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    stake_amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let objector = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let treasury_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    // 验证签名
    if !objector.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;

    // 验证 Treasury
    if treasury_account.key != &config.treasury {
        return Err(ListingError::InvalidAccount.into());
    }

    // 加载提案
    let mut proposal = PerpMarketProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证状态
    if proposal.status != ProposalStatus::Pending {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 验证未过期
    let current_ts = get_current_timestamp()?;
    if current_ts > proposal.review_deadline {
        return Err(ListingError::ReviewDeadlinePassed.into());
    }

    // 检查余额
    if objector.lamports() < stake_amount {
        msg!("Insufficient balance for objection stake");
        return Err(ListingError::InsufficientStake.into());
    }

    // 转移原生 N1024 反对质押
    transfer_native_lamports(
        objector,
        treasury_account,
        stake_amount,
        system_program,
        None,
    )?;

    // 更新总质押
    config.total_staked_lamports = config.total_staked_lamports
        .checked_add(stake_amount)
        .ok_or(ListingError::Overflow)?;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    // 更新反对信息
    proposal.objection_count = proposal.objection_count.checked_add(1)
        .ok_or(ListingError::Overflow)?;
    proposal.objection_stake = proposal.objection_stake.checked_add(stake_amount)
        .ok_or(ListingError::Overflow)?;

    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    msg!("Perp market objection recorded");
    msg!("Objector: {}", objector.key);
    msg!("Stake: {} N1024", stake_amount / 1_000_000_000);
    msg!("Total objections: {}", proposal.objection_count);

    Ok(())
}

fn process_approve_perp_market(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let market_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    // 验证 Admin
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    verify_admin(admin, &config)?;

    // 加载提案
    let mut proposal = PerpMarketProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证状态
    if proposal.status != ProposalStatus::Pending {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 分配 market_index
    let market_index = config.total_perp_markets;
    config.total_perp_markets = config.total_perp_markets.checked_add(1)
        .ok_or(ListingError::Overflow)?;

    // 验证 PDA
    let (expected_market, bump) = derive_perp_market_pda(market_index, program_id);
    if market_account.key != &expected_market {
        return Err(ListingError::InvalidPda.into());
    }

    // 创建 Market 账户
    create_account(
        admin,
        market_account,
        PERP_MARKET_SIZE,
        program_id,
        system_program,
        &[PERP_MARKET_SEED, &market_index.to_le_bytes(), &[bump]],
    )?;

    // 初始化 Market
    let current_ts = get_current_timestamp()?;
    let market = PerpMarket {
        discriminator: PerpMarket::DISCRIMINATOR,
        version: 1,
        market_index,
        symbol: proposal.symbol,
        base_token_index: proposal.base_token_index,
        quote_token_index: proposal.quote_token_index,
        oracle: proposal.oracle,
        tick_size_e6: proposal.tick_size_e6,
        lot_size_e6: proposal.lot_size_e6,
        max_leverage: proposal.max_leverage,
        initial_margin_rate_e6: proposal.initial_margin_rate_e6,
        maintenance_margin_rate_e6: proposal.maintenance_margin_rate_e6,
        taker_fee_bps: proposal.taker_fee_bps,
        maker_fee_bps: proposal.maker_fee_bps,
        min_order_size_e6: proposal.min_order_size_e6,
        max_order_size_e6: proposal.max_order_size_e6,
        max_open_interest_e6: proposal.max_open_interest_e6,
        current_open_interest_long_e6: 0,
        current_open_interest_short_e6: 0,
        insurance_fund_deposit_e6: proposal.insurance_fund_deposit_e6,
        funding_rate_e9: 0,
        last_funding_ts: current_ts,
        is_active: true,
        is_paused: false,
        proposer: proposal.proposer,
        approved_at: current_ts,
        bump,
        reserved: [0u8; 64],
    };

    market.serialize(&mut &mut market_account.data.borrow_mut()[..])?;

    // 更新提案状态
    proposal.status = ProposalStatus::Approved;
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    // 保存配置
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("Perp market approved");
    msg!("Market index: {}", market_index);
    msg!("Symbol: {:?}", std::str::from_utf8(&proposal.symbol).unwrap_or(""));
    msg!("Max Leverage: {}x", proposal.max_leverage);

    Ok(())
}

fn process_reject_perp_market(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    reason_code: u8,
    slash_percentage: u8,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let _treasury_account = next_account_info(account_iter)?;

    // 加载配置并验证 Admin
    let config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    verify_admin(admin, &config)?;

    // 加载提案
    let mut proposal = PerpMarketProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证状态
    if proposal.status != ProposalStatus::Pending {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 验证罚没比例
    if slash_percentage > 100 {
        return Err(ListingError::InvalidAmount.into());
    }

    // 更新状态
    proposal.status = ProposalStatus::Rejected;
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    // 计算罚没金额（保留在 treasury 中）
    let slash_amount = proposal.stake_amount * slash_percentage as u64 / 100;
    
    msg!("Perp market rejected");
    msg!("Reason code: {}", reason_code);
    msg!("Slash percentage: {}%", slash_percentage);
    msg!("Slash amount: {} N1024", slash_amount / 1_000_000_000);

    Ok(())
}

fn process_cancel_perp_proposal(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let proposer = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let treasury_account = next_account_info(account_iter)?;

    // 验证签名
    if !proposer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;

    // 验证 Treasury
    if treasury_account.key != &config.treasury {
        return Err(ListingError::InvalidAccount.into());
    }

    // 加载提案
    let mut proposal = PerpMarketProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证 Proposer
    if &proposal.proposer != proposer.key {
        return Err(ListingError::NotProposer.into());
    }

    // 验证状态
    if proposal.status != ProposalStatus::Pending {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 更新状态
    proposal.status = ProposalStatus::Cancelled;
    
    // 计算退还金额（扣除 5%）
    let refund = proposal.stake_amount * 95 / 100;
    let slash = proposal.stake_amount - refund;
    
    // 从 Treasury PDA 退还原生 N1024
    transfer_lamports_from_pda(
        treasury_account,
        proposer,
        refund,
    )?;

    // 更新总质押额（扣除退还部分）
    config.total_staked_lamports = config.total_staked_lamports
        .checked_sub(refund)
        .ok_or(ListingError::Overflow)?;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    msg!("Perp market proposal cancelled");
    msg!("Refund: {} N1024 (95%)", refund / 1_000_000_000);
    msg!("Slashed: {} N1024 (5%)", slash / 1_000_000_000);

    Ok(())
}

fn process_finalize_perp_market(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let caller = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let market_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    // 验证签名
    if !caller.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;

    // 加载提案
    let mut proposal = PerpMarketProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证状态
    if proposal.status != ProposalStatus::Pending {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 验证已过审核期
    let current_ts = get_current_timestamp()?;
    if current_ts <= proposal.review_deadline {
        return Err(ListingError::ReviewDeadlineNotReached.into());
    }

    // 检查是否有重大反对（如果反对质押超过提案质押的 50%）
    if proposal.objection_count > 0 && proposal.objection_stake > proposal.stake_amount / 2 {
        msg!("Cannot auto-approve: significant objections (stake: {} vs proposal: {})", 
             proposal.objection_stake, proposal.stake_amount);
        return Err(ListingError::ProposalNotPending.into());
    }

    // 分配 market_index
    let market_index = config.total_perp_markets;
    config.total_perp_markets = config.total_perp_markets.checked_add(1)
        .ok_or(ListingError::Overflow)?;

    // 验证 PDA
    let (expected_market, bump) = derive_perp_market_pda(market_index, program_id);
    if market_account.key != &expected_market {
        return Err(ListingError::InvalidPda.into());
    }

    // 创建 Market 账户
    create_account(
        caller,
        market_account,
        PERP_MARKET_SIZE,
        program_id,
        system_program,
        &[PERP_MARKET_SEED, &market_index.to_le_bytes(), &[bump]],
    )?;

    // 初始化 Market
    let market = PerpMarket {
        discriminator: PerpMarket::DISCRIMINATOR,
        version: 1,
        market_index,
        symbol: proposal.symbol,
        base_token_index: proposal.base_token_index,
        quote_token_index: proposal.quote_token_index,
        oracle: proposal.oracle,
        tick_size_e6: proposal.tick_size_e6,
        lot_size_e6: proposal.lot_size_e6,
        max_leverage: proposal.max_leverage,
        initial_margin_rate_e6: proposal.initial_margin_rate_e6,
        maintenance_margin_rate_e6: proposal.maintenance_margin_rate_e6,
        taker_fee_bps: proposal.taker_fee_bps,
        maker_fee_bps: proposal.maker_fee_bps,
        min_order_size_e6: proposal.min_order_size_e6,
        max_order_size_e6: proposal.max_order_size_e6,
        max_open_interest_e6: proposal.max_open_interest_e6,
        current_open_interest_long_e6: 0,
        current_open_interest_short_e6: 0,
        insurance_fund_deposit_e6: proposal.insurance_fund_deposit_e6,
        funding_rate_e9: 0,
        last_funding_ts: current_ts,
        is_active: true,
        is_paused: false,
        proposer: proposal.proposer,
        approved_at: current_ts,
        bump,
        reserved: [0u8; 64],
    };

    market.serialize(&mut &mut market_account.data.borrow_mut()[..])?;

    // 更新提案状态
    proposal.status = ProposalStatus::Approved;
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    // 保存配置
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("Perp market auto-approved (finalized)");
    msg!("Market index: {}", market_index);
    msg!("Symbol: {:?}", std::str::from_utf8(&proposal.symbol).unwrap_or(""));
    msg!("Max Leverage: {}x", proposal.max_leverage);

    Ok(())
}

fn process_claim_perp_stake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let proposer = next_account_info(account_iter)?;
    let proposal_account = next_account_info(account_iter)?;
    let market_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let treasury_account = next_account_info(account_iter)?;

    // 验证签名
    if !proposer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let mut config = ListingConfig::try_from_slice(&config_account.data.borrow())?;

    // 验证 Treasury
    if treasury_account.key != &config.treasury {
        return Err(ListingError::InvalidAccount.into());
    }

    // 加载提案
    let mut proposal = PerpMarketProposal::try_from_slice(&proposal_account.data.borrow())?;
    
    // 验证 Proposer
    if &proposal.proposer != proposer.key {
        return Err(ListingError::NotProposer.into());
    }

    // 验证状态已批准
    if proposal.status != ProposalStatus::Approved {
        return Err(ListingError::ProposalNotPending.into());
    }

    // 验证未取回
    if proposal.stake_claimed {
        return Err(ListingError::StakeAlreadyClaimed.into());
    }

    // 验证 Market 存在（通过检查 proposer 匹配）
    let market = PerpMarket::try_from_slice(&market_account.data.borrow())?;
    if market.proposer != *proposer.key {
        return Err(ListingError::NotProposer.into());
    }

    // 验证锁定期已过
    let current_ts = get_current_timestamp()?;
    let unlock_time = market.approved_at + config.stake_lock_period_seconds as i64;
    if current_ts < unlock_time {
        msg!("Stake locked until: {}", unlock_time);
        return Err(ListingError::StakeLockPeriodNotEnded.into());
    }

    // 从 Treasury PDA 退还原生 N1024
    transfer_lamports_from_pda(
        treasury_account,
        proposer,
        proposal.stake_amount,
    )?;

    // 更新总质押额
    config.total_staked_lamports = config.total_staked_lamports
        .checked_sub(proposal.stake_amount)
        .ok_or(ListingError::Overflow)?;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    // 标记已取回
    proposal.stake_claimed = true;
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    msg!("Perp market stake claimed");
    msg!("Amount: {} N1024", proposal.stake_amount / 1_000_000_000);

    Ok(())
}

fn process_update_perp_status(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    is_active: Option<bool>,
    is_paused: Option<bool>,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let market_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;

    // 加载配置并验证 Admin
    let config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    verify_admin(admin, &config)?;

    // 加载 Market
    let mut market = PerpMarket::try_from_slice(&market_account.data.borrow())?;
    
    // 更新状态
    if let Some(active) = is_active {
        market.is_active = active;
        msg!("Perp market is_active updated to: {}", active);
    }
    if let Some(paused) = is_paused {
        market.is_paused = paused;
        msg!("Perp market is_paused updated to: {}", paused);
    }

    market.serialize(&mut &mut market_account.data.borrow_mut()[..])?;

    msg!("Perp market status updated");
    msg!("Market index: {}", market.market_index);

    Ok(())
}

fn process_update_perp_params(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    max_leverage: Option<u8>,
    initial_margin_rate_e6: Option<u32>,
    maintenance_margin_rate_e6: Option<u32>,
    taker_fee_bps: Option<u16>,
    maker_fee_bps: Option<i16>,
    max_open_interest_e6: Option<u64>,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let admin = next_account_info(account_iter)?;
    let market_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;

    // 加载配置并验证 Admin
    let config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    verify_admin(admin, &config)?;

    // 加载 Market
    let mut market = PerpMarket::try_from_slice(&market_account.data.borrow())?;
    
    // 更新参数
    if let Some(leverage) = max_leverage {
        if leverage == 0 || leverage > 100 {
            return Err(ListingError::InvalidLeverage.into());
        }
        market.max_leverage = leverage;
        msg!("Max leverage updated to: {}x", leverage);
    }
    if let Some(rate) = initial_margin_rate_e6 {
        if rate == 0 || rate > 1_000_000 {
            return Err(ListingError::InvalidInitialMarginRate.into());
        }
        market.initial_margin_rate_e6 = rate;
        msg!("Initial margin rate updated to: {} (e6)", rate);
    }
    if let Some(rate) = maintenance_margin_rate_e6 {
        // 验证维持保证金率 < 初始保证金率
        let init_rate = initial_margin_rate_e6.unwrap_or(market.initial_margin_rate_e6);
        if rate >= init_rate {
            return Err(ListingError::InvalidMaintenanceMarginRate.into());
        }
        market.maintenance_margin_rate_e6 = rate;
        msg!("Maintenance margin rate updated to: {} (e6)", rate);
    }
    if let Some(fee) = taker_fee_bps {
        if fee > 1000 {
            return Err(ListingError::InvalidFeeRate.into());
        }
        market.taker_fee_bps = fee;
        msg!("Taker fee updated to: {} bps", fee);
    }
    if let Some(fee) = maker_fee_bps {
        if fee < -500 || fee > 500 {
            return Err(ListingError::InvalidFeeRate.into());
        }
        market.maker_fee_bps = fee;
        msg!("Maker fee updated to: {} bps", fee);
    }
    if let Some(max_oi) = max_open_interest_e6 {
        market.max_open_interest_e6 = max_oi;
        msg!("Max open interest updated to: {} (e6)", max_oi);
    }

    market.serialize(&mut &mut market_account.data.borrow_mut()[..])?;

    msg!("Perp market params updated");
    msg!("Market index: {}", market.market_index);

    Ok(())
}

// =============================================================================
// PLP-4: 初始流动性池指令处理（占位）
// =============================================================================

fn process_initialize_liquidity_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    market_type: u8,
    price_lower_e6: u64,
    price_upper_e6: u64,
    order_density: u16,
    spread_bps: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let creator = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;
    let market_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    // 验证签名
    if !creator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载配置
    let config = ListingConfig::try_from_slice(&config_account.data.borrow())?;
    if config.is_paused {
        return Err(ListingError::ListingPaused.into());
    }

    // 解析市场类型
    let market_type_enum = match market_type {
        0 => MarketType::Spot,
        1 => MarketType::Perp,
        _ => return Err(ListingError::InvalidMarketType.into()),
    };

    // 获取市场索引并验证市场
    let market_index: u16;
    match market_type_enum {
        MarketType::Spot => {
            let market = SpotMarket::try_from_slice(&market_account.data.borrow())?;
            if !market.is_active {
                return Err(ListingError::MarketNotFound.into());
            }
            market_index = market.market_index;
        }
        MarketType::Perp => {
            let market = PerpMarket::try_from_slice(&market_account.data.borrow())?;
            if !market.is_active {
                return Err(ListingError::MarketNotFound.into());
            }
            market_index = market.market_index;
        }
    }

    // 验证 PDA - 使用 creator 作为 nonce 的组成部分
    let (expected_pool, bump) = Pubkey::find_program_address(
        &[
            LIQUIDITY_POOL_SEED,
            &[market_type],
            &market_index.to_le_bytes(),
            creator.key.as_ref(),
        ],
        program_id,
    );
    if pool_account.key != &expected_pool {
        return Err(ListingError::InvalidPda.into());
    }

    // 检查是否已存在
    if !pool_account.data_is_empty() {
        return Err(ListingError::AlreadyInitialized.into());
    }

    // 验证价格区间
    if price_lower_e6 >= price_upper_e6 {
        return Err(ListingError::InvalidPriceRange.into());
    }

    // 验证订单密度 (1-100)
    if order_density == 0 || order_density > 100 {
        return Err(ListingError::InvalidOrderDensity.into());
    }

    // 验证 spread (至少 1 bps)
    if spread_bps == 0 || spread_bps > 10000 {
        return Err(ListingError::InvalidSpread.into());
    }

    // 创建账户
    create_account(
        creator,
        pool_account,
        LIQUIDITY_POOL_SIZE,
        program_id,
        system_program,
        &[
            LIQUIDITY_POOL_SEED,
            &[market_type],
            &market_index.to_le_bytes(),
            creator.key.as_ref(),
            &[bump],
        ],
    )?;

    // 初始化 Pool
    let current_ts = get_current_timestamp()?;

    let pool = LiquidityPool {
        discriminator: LiquidityPool::DISCRIMINATOR,
        version: 1,
        market_type: market_type_enum,
        market_index,
        nonce: 0,  // 使用 creator key 作为唯一标识，nonce 设为 0
        creator: *creator.key,
        market: *market_account.key,
        base_amount_e6: 0,  // 初始资金通过 FundLiquidityPool 添加
        quote_amount_e6: 0,
        lp_token_supply_e6: 0,
        price_lower_e6,
        price_upper_e6,
        order_density,
        spread_bps,
        is_active: true,
        created_at: current_ts,
        unlock_time: 0,  // 初始化时未锁定
        retire_at: 0,  // 永不退休
        bump,
        reserved: [0u8; 64],
    };

    pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

    msg!("Liquidity pool initialized");
    msg!("Market type: {}", market_type);
    msg!("Market index: {}", market_index);
    msg!("Price range: {} - {} (e6)", price_lower_e6, price_upper_e6);
    msg!("Order density: {}", order_density);
    msg!("Spread: {} bps", spread_bps);

    Ok(())
}

fn process_fund_liquidity_pool(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    base_amount_e6: u64,
    quote_amount_e6: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let funder = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;
    let _config_account = next_account_info(account_iter)?;
    // 注: 实际实现需要资金转移，通过 Vault Program CPI

    // 验证签名
    if !funder.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载 Pool
    let mut pool = LiquidityPool::try_from_slice(&pool_account.data.borrow())?;
    
    // 验证 Pool 活跃
    if !pool.is_active {
        return Err(ListingError::PoolNotActive.into());
    }

    // 只有创建者可以注入资金（或通过单独的 LP token 机制）
    if pool.creator != *funder.key {
        return Err(ListingError::Unauthorized.into());
    }

    // 验证金额
    if base_amount_e6 == 0 && quote_amount_e6 == 0 {
        return Err(ListingError::InvalidAmount.into());
    }

    // 更新池子余额
    pool.base_amount_e6 = pool.base_amount_e6
        .checked_add(base_amount_e6)
        .ok_or(ListingError::Overflow)?;
    pool.quote_amount_e6 = pool.quote_amount_e6
        .checked_add(quote_amount_e6)
        .ok_or(ListingError::Overflow)?;

    // TODO: 通过 CPI 调用 Vault Program 转移实际 Token

    pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

    msg!("Liquidity pool funded");
    msg!("Base added: {} (e6)", base_amount_e6);
    msg!("Quote added: {} (e6)", quote_amount_e6);
    msg!("New base balance: {} (e6)", pool.base_amount_e6);
    msg!("New quote balance: {} (e6)", pool.quote_amount_e6);

    Ok(())
}

fn process_adjust_liquidity_pool(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    price_lower_e6: Option<u64>,
    price_upper_e6: Option<u64>,
    order_density: Option<u16>,
    spread_bps: Option<u64>,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let operator = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;
    let _config_account = next_account_info(account_iter)?;

    // 验证签名
    if !operator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载 Pool
    let pool = LiquidityPool::try_from_slice(&pool_account.data.borrow())?;
    
    // 验证 Pool 活跃
    if !pool.is_active {
        return Err(ListingError::PoolNotActive.into());
    }

    // 验证权限：只有创建者可以调整
    if pool.creator != *operator.key {
        return Err(ListingError::Unauthorized.into());
    }

    // 验证参数（如果提供）
    if let (Some(lower), Some(upper)) = (price_lower_e6, price_upper_e6) {
        if lower >= upper {
            return Err(ListingError::InvalidPriceRange.into());
        }
    }

    if let Some(density) = order_density {
        if density == 0 || density > 100 {
            return Err(ListingError::InvalidOrderDensity.into());
        }
    }

    if let Some(spread) = spread_bps {
        if spread == 0 || spread > 10000 {
            return Err(ListingError::InvalidSpread.into());
        }
    }

    // TODO: 实际更新参数（需扩展 LiquidityPool 结构或使用单独的 PoolConfig PDA）

    msg!("Liquidity pool params adjusted");
    if let Some(lower) = price_lower_e6 {
        msg!("New price lower: {} (e6)", lower);
    }
    if let Some(upper) = price_upper_e6 {
        msg!("New price upper: {} (e6)", upper);
    }
    if let Some(density) = order_density {
        msg!("New order density: {}", density);
    }
    if let Some(spread) = spread_bps {
        msg!("New spread: {} bps", spread);
    }

    Ok(())
}

fn process_refresh_liquidity_orders(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let operator = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;
    let _market_account = next_account_info(account_iter)?;
    let _oracle_account = next_account_info(account_iter)?;

    // 验证签名
    if !operator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载 Pool
    let pool = LiquidityPool::try_from_slice(&pool_account.data.borrow())?;
    
    // 验证 Pool 活跃
    if !pool.is_active {
        return Err(ListingError::PoolNotActive.into());
    }

    // 验证权限：只有创建者可以刷新订单
    if pool.creator != *operator.key {
        return Err(ListingError::Unauthorized.into());
    }

    // TODO: 实际刷新订单逻辑
    // 1. 从 Oracle 获取当前价格
    // 2. 根据价格区间和订单密度生成订单
    // 3. 通过 CPI 提交订单到 Matcher

    msg!("Liquidity pool orders refreshed");
    msg!("Pool market type: {:?}", pool.market_type);
    msg!("Pool market index: {}", pool.market_index);

    Ok(())
}

fn process_withdraw_liquidity_profit(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    base_amount_e6: u64,
    quote_amount_e6: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let withdrawer = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;
    let _config_account = next_account_info(account_iter)?;

    // 验证签名
    if !withdrawer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载 Pool
    let mut pool = LiquidityPool::try_from_slice(&pool_account.data.borrow())?;
    
    // 验证 Pool 活跃
    if !pool.is_active {
        return Err(ListingError::PoolNotActive.into());
    }

    // 验证权限
    if pool.creator != *withdrawer.key {
        return Err(ListingError::Unauthorized.into());
    }

    // 验证金额
    if base_amount_e6 == 0 && quote_amount_e6 == 0 {
        return Err(ListingError::InvalidAmount.into());
    }

    // 验证余额充足
    if base_amount_e6 > pool.base_amount_e6 || quote_amount_e6 > pool.quote_amount_e6 {
        return Err(ListingError::InsufficientBalance.into());
    }

    // 更新余额
    pool.base_amount_e6 = pool.base_amount_e6
        .checked_sub(base_amount_e6)
        .ok_or(ListingError::Underflow)?;
    pool.quote_amount_e6 = pool.quote_amount_e6
        .checked_sub(quote_amount_e6)
        .ok_or(ListingError::Underflow)?;

    // TODO: 通过 CPI 调用 Vault Program 转移实际 Token 给 withdrawer

    pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

    msg!("Liquidity profit withdrawn");
    msg!("Base withdrawn: {} (e6)", base_amount_e6);
    msg!("Quote withdrawn: {} (e6)", quote_amount_e6);
    msg!("Remaining base: {} (e6)", pool.base_amount_e6);
    msg!("Remaining quote: {} (e6)", pool.quote_amount_e6);

    Ok(())
}

fn process_retire_liquidity_pool(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let operator = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;
    let _config_account = next_account_info(account_iter)?;

    // 验证签名
    if !operator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 加载 Pool
    let mut pool = LiquidityPool::try_from_slice(&pool_account.data.borrow())?;
    
    // 验证 Pool 仍然活跃
    if !pool.is_active {
        return Err(ListingError::PoolNotActive.into());
    }

    // 验证权限
    if pool.creator != *operator.key {
        return Err(ListingError::Unauthorized.into());
    }

    // 验证没有剩余资金（需先提取所有资金）
    if pool.base_amount_e6 > 0 || pool.quote_amount_e6 > 0 {
        msg!("Pool still has funds. Withdraw all funds before retiring.");
        return Err(ListingError::PoolHasRemainingFunds.into());
    }

    // 停用 Pool
    pool.is_active = false;
    pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

    msg!("Liquidity pool retired");
    msg!("Pool market type: {:?}", pool.market_type);
    msg!("Pool market index: {}", pool.market_index);

    Ok(())
}

