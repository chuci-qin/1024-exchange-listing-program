//! Listing Program State Definitions
//!
//! PLP (Permissionless Listing Protocol) PDA 账户结构
//!
//! ## 质押机制
//! 使用原生 N1024 (lamports) 进行质押，不使用 SPL Token。
//! 这样用户可以直接使用 1024Chain 原生代币进行质押，无需 wrap。
//!
//! ## PDA 列表
//! - `ListingConfig`: 全局配置
//! - `TokenRegistry`: 已注册 Token (PLP-1)
//! - `TokenProposal`: Token 注册提案 (PLP-1)
//! - `SpotMarket`: Spot 市场配置 (PLP-2)
//! - `SpotMarketProposal`: Spot 市场提案 (PLP-2)
//! - `PerpMarket`: Perp 市场配置 (PLP-3)
//! - `PerpMarketProposal`: Perp 市场提案 (PLP-3)
//! - `LiquidityPool`: 初始流动性池 (PLP-4)

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

// =============================================================================
// Seeds 常量
// =============================================================================

pub const LISTING_CONFIG_SEED: &[u8] = b"listing_config";
pub const LISTING_TREASURY_SEED: &[u8] = b"listing_treasury";
pub const TOKEN_REGISTRY_SEED: &[u8] = b"token";
pub const TOKEN_PROPOSAL_SEED: &[u8] = b"token_proposal";
pub const SPOT_MARKET_SEED: &[u8] = b"spot_market";
pub const SPOT_PROPOSAL_SEED: &[u8] = b"spot_proposal";
pub const PERP_MARKET_SEED: &[u8] = b"perp_market";
pub const PERP_PROPOSAL_SEED: &[u8] = b"perp_proposal";
pub const LIQUIDITY_POOL_SEED: &[u8] = b"plp4_pool";

// =============================================================================
// Discriminators
// =============================================================================

pub const LISTING_CONFIG_DISCRIMINATOR: u64 = 0x4C495354_434F4E46; // "LIST_CONF"
pub const TOKEN_REGISTRY_DISCRIMINATOR: u64 = 0x544F4B45_4E524547; // "TOKENREG"
pub const TOKEN_PROPOSAL_DISCRIMINATOR: u64 = 0x544F4B45_4E50524F; // "TOKENPRO"
pub const SPOT_MARKET_DISCRIMINATOR: u64 = 0x53504F54_4D4B5420; // "SPOTMKT "
pub const SPOT_PROPOSAL_DISCRIMINATOR: u64 = 0x53504F54_50524F50; // "SPOTPROP"
pub const PERP_MARKET_DISCRIMINATOR: u64 = 0x50455250_4D4B5420; // "PERPMKT "
pub const PERP_PROPOSAL_DISCRIMINATOR: u64 = 0x50455250_50524F50; // "PERPPROP"
pub const LIQUIDITY_POOL_DISCRIMINATOR: u64 = 0x504C5034_504F4F4C; // "PLP4POOL"

// =============================================================================
// 账户大小计算
// =============================================================================

pub const LISTING_CONFIG_SIZE: usize = 8 +  // discriminator
    1 +  // version
    32 + // admin
    // stake_token_mint 已移除 - 使用原生 N1024 (lamports)
    32 + // treasury (PDA, 接收原生 N1024 质押)
    32 + // vault_program
    32 + // fund_program
    32 + // ledger_program
    8 +  // token_stake_amount (1,000 N1024 = 1_000_000_000_000 lamports)
    8 +  // spot_stake_amount (2,000 N1024)
    8 +  // perp_stake_amount (5,000 N1024)
    4 +  // token_review_period_seconds (7 days)
    4 +  // spot_review_period_seconds (7 days)
    4 +  // perp_review_period_seconds (14 days)
    4 +  // stake_lock_period_seconds (30 days)
    2 +  // total_tokens
    2 +  // total_spot_markets
    2 +  // total_perp_markets
    8 +  // total_staked_lamports (统计总质押)
    1 +  // is_paused
    1 +  // bump
    64;  // reserved

pub const TOKEN_REGISTRY_SIZE: usize = 8 +  // discriminator
    1 +  // version
    2 +  // token_index
    8 +  // symbol [u8; 8]
    32 + // mint
    1 +  // decimals
    33 + // oracle (Option<Pubkey>)
    1 +  // is_active
    32 + // proposer
    8 +  // approved_at
    1 +  // bump
    64;  // reserved

pub const TOKEN_PROPOSAL_SIZE: usize = 8 +  // discriminator
    1 +  // version
    32 + // proposer
    8 +  // nonce
    8 +  // symbol [u8; 8]
    32 + // mint
    1 +  // decimals
    33 + // oracle (Option<Pubkey>)
    8 +  // stake_amount
    1 +  // status
    8 +  // created_at
    8 +  // review_deadline
    2 +  // objection_count
    8 +  // objection_stake
    1 +  // stake_claimed
    1 +  // bump
    64;  // reserved

pub const SPOT_MARKET_SIZE: usize = 8 +  // discriminator
    1 +  // version
    2 +  // market_index
    16 + // symbol [u8; 16] (e.g., "BTC/USDC")
    2 +  // base_token_index
    2 +  // quote_token_index
    8 +  // tick_size_e6
    8 +  // lot_size_e6
    2 +  // taker_fee_bps
    2 +  // maker_fee_bps (i16)
    8 +  // min_order_size_e6
    8 +  // max_order_size_e6
    1 +  // is_active
    1 +  // is_paused
    32 + // proposer
    8 +  // approved_at
    1 +  // bump
    64;  // reserved

pub const SPOT_PROPOSAL_SIZE: usize = 8 +  // discriminator
    1 +  // version
    32 + // proposer
    8 +  // nonce
    16 + // symbol [u8; 16]
    2 +  // base_token_index
    2 +  // quote_token_index
    8 +  // tick_size_e6
    8 +  // lot_size_e6
    2 +  // taker_fee_bps
    2 +  // maker_fee_bps (i16)
    8 +  // min_order_size_e6
    8 +  // max_order_size_e6
    8 +  // stake_amount
    1 +  // status
    8 +  // created_at
    8 +  // review_deadline
    2 +  // objection_count
    8 +  // objection_stake
    1 +  // stake_claimed
    1 +  // bump
    64;  // reserved

pub const PERP_MARKET_SIZE: usize = 8 +  // discriminator
    1 +  // version
    2 +  // market_index
    16 + // symbol [u8; 16] (e.g., "BTC-USDC")
    2 +  // base_token_index
    2 +  // quote_token_index
    32 + // oracle
    8 +  // tick_size_e6
    8 +  // lot_size_e6
    1 +  // max_leverage
    4 +  // initial_margin_rate_e6
    4 +  // maintenance_margin_rate_e6
    2 +  // taker_fee_bps
    2 +  // maker_fee_bps (i16)
    8 +  // min_order_size_e6
    8 +  // max_order_size_e6
    8 +  // max_open_interest_e6
    8 +  // current_open_interest_long_e6
    8 +  // current_open_interest_short_e6
    8 +  // insurance_fund_deposit_e6
    8 +  // funding_rate_e9
    8 +  // last_funding_ts
    1 +  // is_active
    1 +  // is_paused
    32 + // proposer
    8 +  // approved_at
    1 +  // bump
    64;  // reserved

pub const PERP_PROPOSAL_SIZE: usize = 8 +  // discriminator
    1 +  // version
    32 + // proposer
    8 +  // nonce
    16 + // symbol [u8; 16]
    2 +  // base_token_index
    2 +  // quote_token_index
    32 + // oracle
    8 +  // tick_size_e6
    8 +  // lot_size_e6
    1 +  // max_leverage
    4 +  // initial_margin_rate_e6
    4 +  // maintenance_margin_rate_e6
    2 +  // taker_fee_bps
    2 +  // maker_fee_bps (i16)
    8 +  // min_order_size_e6
    8 +  // max_order_size_e6
    8 +  // max_open_interest_e6
    8 +  // insurance_fund_deposit_e6
    8 +  // stake_amount
    1 +  // status
    8 +  // created_at
    8 +  // review_deadline
    2 +  // objection_count
    8 +  // objection_stake
    1 +  // stake_claimed
    1 +  // bump
    64;  // reserved

pub const LIQUIDITY_POOL_SIZE: usize = 8 +  // discriminator
    1 +  // version
    1 +  // market_type (0=Spot, 1=Perp)
    2 +  // market_index
    8 +  // nonce
    32 + // creator
    32 + // market
    8 +  // base_amount_e6
    8 +  // quote_amount_e6
    8 +  // lp_token_supply_e6
    8 +  // price_lower_e6
    8 +  // price_upper_e6
    2 +  // order_density
    8 +  // spread_bps
    1 +  // is_active
    8 +  // created_at
    8 +  // unlock_time
    8 +  // retire_at (0 = never)
    1 +  // bump
    64;  // reserved

// =============================================================================
// 枚举类型
// =============================================================================

/// 提案状态
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ProposalStatus {
    /// 待审核
    Pending = 0,
    /// 已批准
    Approved = 1,
    /// 已拒绝
    Rejected = 2,
    /// 已取消（申请者主动撤回）
    Cancelled = 3,
}

impl Default for ProposalStatus {
    fn default() -> Self {
        ProposalStatus::Pending
    }
}

/// 市场类型
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum MarketType {
    Spot = 0,
    Perp = 1,
}

// =============================================================================
// PDA 账户结构
// =============================================================================

/// 全局配置 (PDA)
/// Seeds: ["listing_config"]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ListingConfig {
    /// 账户类型标识符
    pub discriminator: u64,
    /// 版本号
    pub version: u8,
    /// 管理员
    pub admin: Pubkey,
    // stake_token_mint 已移除 - 使用原生 N1024 (lamports) 质押
    /// 质押国库 PDA（存放原生 N1024 质押和罚没）
    /// Seeds: ["listing_treasury"]
    pub treasury: Pubkey,
    /// Vault Program ID
    pub vault_program: Pubkey,
    /// Fund Program ID
    pub fund_program: Pubkey,
    /// Ledger Program ID
    pub ledger_program: Pubkey,
    /// PLP-1 Token 注册质押金额 (lamports)
    /// 1,000 N1024 = 1_000_000_000_000 lamports (9 decimals)
    pub token_stake_amount: u64,
    /// PLP-2 Spot 上架质押金额 (lamports)
    /// 2,000 N1024
    pub spot_stake_amount: u64,
    /// PLP-3 Perp 上架质押金额 (lamports)
    /// 5,000 N1024
    pub perp_stake_amount: u64,
    /// Token 审核期（秒）（7 天 = 604,800）
    pub token_review_period_seconds: u32,
    /// Spot 审核期（秒）（7 天）
    pub spot_review_period_seconds: u32,
    /// Perp 审核期（秒）（14 天 = 1,209,600）
    pub perp_review_period_seconds: u32,
    /// 质押锁定期（秒）（批准后 30 天 = 2,592,000）
    pub stake_lock_period_seconds: u32,
    /// 已注册 Token 总数
    pub total_tokens: u16,
    /// Spot 市场总数
    pub total_spot_markets: u16,
    /// Perp 市场总数
    pub total_perp_markets: u16,
    /// 累计质押总额 (lamports)
    pub total_staked_lamports: u64,
    /// 是否暂停新上架
    pub is_paused: bool,
    /// PDA bump
    pub bump: u8,
    /// 预留空间
    pub reserved: [u8; 64],
}

impl ListingConfig {
    pub const DISCRIMINATOR: u64 = LISTING_CONFIG_DISCRIMINATOR;
    
    /// 默认 Token 质押金额 (1,000 N1024)
    /// N1024 有 9 decimals，所以 1,000 N1024 = 1_000_000_000_000 lamports
    pub const DEFAULT_TOKEN_STAKE: u64 = 1_000_000_000_000; // 1,000 N1024
    /// 默认 Spot 质押金额 (2,000 N1024)
    pub const DEFAULT_SPOT_STAKE: u64 = 2_000_000_000_000; // 2,000 N1024
    /// 默认 Perp 质押金额 (5,000 N1024)
    pub const DEFAULT_PERP_STAKE: u64 = 5_000_000_000_000; // 5,000 N1024
    /// 默认 Token 审核期 (7 天)
    pub const DEFAULT_TOKEN_REVIEW_PERIOD: u32 = 7 * 24 * 60 * 60;
    /// 默认 Spot 审核期 (7 天)
    pub const DEFAULT_SPOT_REVIEW_PERIOD: u32 = 7 * 24 * 60 * 60;
    /// 默认 Perp 审核期 (14 天)
    pub const DEFAULT_PERP_REVIEW_PERIOD: u32 = 14 * 24 * 60 * 60;
    /// 默认质押锁定期 (30 天)
    pub const DEFAULT_STAKE_LOCK_PERIOD: u32 = 30 * 24 * 60 * 60;
}

/// 已注册 Token (PLP-1)
/// Seeds: ["token", token_index.to_le_bytes()]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TokenRegistry {
    /// 账户类型标识符
    pub discriminator: u64,
    /// 版本号
    pub version: u8,
    /// Token 索引
    pub token_index: u16,
    /// Symbol (最多 8 字符)
    pub symbol: [u8; 8],
    /// SPL Token Mint 地址
    pub mint: Pubkey,
    /// 精度
    pub decimals: u8,
    /// Oracle 地址 (Pyth/Switchboard)，Perp 必须，Spot 可选
    pub oracle: Option<Pubkey>,
    /// 是否激活
    pub is_active: bool,
    /// 申请者
    pub proposer: Pubkey,
    /// 批准时间戳
    pub approved_at: i64,
    /// PDA bump
    pub bump: u8,
    /// 预留空间
    pub reserved: [u8; 64],
}

impl TokenRegistry {
    pub const DISCRIMINATOR: u64 = TOKEN_REGISTRY_DISCRIMINATOR;
    
    /// 获取 symbol 字符串
    pub fn symbol_str(&self) -> &str {
        let len = self.symbol.iter().position(|&c| c == 0).unwrap_or(8);
        std::str::from_utf8(&self.symbol[..len]).unwrap_or("")
    }
}

/// Token 注册提案 (PLP-1)
/// Seeds: ["token_proposal", proposer, nonce.to_le_bytes()]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TokenProposal {
    /// 账户类型标识符
    pub discriminator: u64,
    /// 版本号
    pub version: u8,
    /// 申请者
    pub proposer: Pubkey,
    /// 唯一序号
    pub nonce: u64,
    /// Symbol
    pub symbol: [u8; 8],
    /// SPL Token Mint 地址
    pub mint: Pubkey,
    /// 精度
    pub decimals: u8,
    /// Oracle 地址
    pub oracle: Option<Pubkey>,
    /// 质押金额
    pub stake_amount: u64,
    /// 提案状态
    pub status: ProposalStatus,
    /// 创建时间戳
    pub created_at: i64,
    /// 审核截止时间
    pub review_deadline: i64,
    /// 反对数量
    pub objection_count: u16,
    /// 反对质押总额
    pub objection_stake: u64,
    /// 质押是否已取回
    pub stake_claimed: bool,
    /// PDA bump
    pub bump: u8,
    /// 预留空间
    pub reserved: [u8; 64],
}

impl TokenProposal {
    pub const DISCRIMINATOR: u64 = TOKEN_PROPOSAL_DISCRIMINATOR;
}

/// Spot 市场配置 (PLP-2)
/// Seeds: ["spot_market", market_index.to_le_bytes()]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SpotMarket {
    /// 账户类型标识符
    pub discriminator: u64,
    /// 版本号
    pub version: u8,
    /// 市场索引
    pub market_index: u16,
    /// Symbol (e.g., "BTC/USDC")
    pub symbol: [u8; 16],
    /// Base Token 索引
    pub base_token_index: u16,
    /// Quote Token 索引 (通常是 USDC)
    pub quote_token_index: u16,
    /// 最小价格变动 (e6)
    pub tick_size_e6: u64,
    /// 最小数量变动 (e6)
    pub lot_size_e6: u64,
    /// Taker 费率 (bps, 1 = 0.01%)
    pub taker_fee_bps: u16,
    /// Maker 费率 (bps, 可为负表示返佣)
    pub maker_fee_bps: i16,
    /// 最小订单大小 (e6)
    pub min_order_size_e6: u64,
    /// 最大订单大小 (e6)
    pub max_order_size_e6: u64,
    /// 是否激活
    pub is_active: bool,
    /// 是否暂停交易
    pub is_paused: bool,
    /// 申请者
    pub proposer: Pubkey,
    /// 批准时间戳
    pub approved_at: i64,
    /// PDA bump
    pub bump: u8,
    /// 预留空间
    pub reserved: [u8; 64],
}

impl SpotMarket {
    pub const DISCRIMINATOR: u64 = SPOT_MARKET_DISCRIMINATOR;
    
    /// 获取 symbol 字符串
    pub fn symbol_str(&self) -> &str {
        let len = self.symbol.iter().position(|&c| c == 0).unwrap_or(16);
        std::str::from_utf8(&self.symbol[..len]).unwrap_or("")
    }
}

/// Spot 市场上架提案 (PLP-2)
/// Seeds: ["spot_proposal", proposer, nonce.to_le_bytes()]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SpotMarketProposal {
    /// 账户类型标识符
    pub discriminator: u64,
    /// 版本号
    pub version: u8,
    /// 申请者
    pub proposer: Pubkey,
    /// 唯一序号
    pub nonce: u64,
    /// Symbol
    pub symbol: [u8; 16],
    /// Base Token 索引
    pub base_token_index: u16,
    /// Quote Token 索引
    pub quote_token_index: u16,
    /// 最小价格变动 (e6)
    pub tick_size_e6: u64,
    /// 最小数量变动 (e6)
    pub lot_size_e6: u64,
    /// Taker 费率 (bps)
    pub taker_fee_bps: u16,
    /// Maker 费率 (bps)
    pub maker_fee_bps: i16,
    /// 最小订单大小 (e6)
    pub min_order_size_e6: u64,
    /// 最大订单大小 (e6)
    pub max_order_size_e6: u64,
    /// 质押金额
    pub stake_amount: u64,
    /// 提案状态
    pub status: ProposalStatus,
    /// 创建时间戳
    pub created_at: i64,
    /// 审核截止时间
    pub review_deadline: i64,
    /// 反对数量
    pub objection_count: u16,
    /// 反对质押总额
    pub objection_stake: u64,
    /// 质押是否已取回
    pub stake_claimed: bool,
    /// PDA bump
    pub bump: u8,
    /// 预留空间
    pub reserved: [u8; 64],
}

impl SpotMarketProposal {
    pub const DISCRIMINATOR: u64 = SPOT_PROPOSAL_DISCRIMINATOR;
}

/// Perp 市场配置 (PLP-3)
/// Seeds: ["perp_market", market_index.to_le_bytes()]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PerpMarket {
    /// 账户类型标识符
    pub discriminator: u64,
    /// 版本号
    pub version: u8,
    /// 市场索引
    pub market_index: u16,
    /// Symbol (e.g., "BTC-USDC")
    pub symbol: [u8; 16],
    /// Base Token 索引
    pub base_token_index: u16,
    /// Quote Token 索引 (USDC)
    pub quote_token_index: u16,
    /// Oracle 地址 (必须)
    pub oracle: Pubkey,
    /// 最小价格变动 (e6)
    pub tick_size_e6: u64,
    /// 最小数量变动 (e6)
    pub lot_size_e6: u64,
    /// 最大杠杆 (1-100)
    pub max_leverage: u8,
    /// 初始保证金率 (e6, 如 100000 = 10%)
    pub initial_margin_rate_e6: u32,
    /// 维持保证金率 (e6, 如 50000 = 5%)
    pub maintenance_margin_rate_e6: u32,
    /// Taker 费率 (bps)
    pub taker_fee_bps: u16,
    /// Maker 费率 (bps)
    pub maker_fee_bps: i16,
    /// 最小订单大小 (e6)
    pub min_order_size_e6: u64,
    /// 最大订单大小 (e6)
    pub max_order_size_e6: u64,
    /// 最大持仓量 (e6)
    pub max_open_interest_e6: u64,
    /// 当前多头持仓量 (e6)
    pub current_open_interest_long_e6: u64,
    /// 当前空头持仓量 (e6)
    pub current_open_interest_short_e6: u64,
    /// 保险金存款 (e6)
    pub insurance_fund_deposit_e6: u64,
    /// 当前资金费率 (e9)
    pub funding_rate_e9: i64,
    /// 上次资金费率更新时间戳
    pub last_funding_ts: i64,
    /// 是否激活
    pub is_active: bool,
    /// 是否暂停交易
    pub is_paused: bool,
    /// 申请者
    pub proposer: Pubkey,
    /// 批准时间戳
    pub approved_at: i64,
    /// PDA bump
    pub bump: u8,
    /// 预留空间
    pub reserved: [u8; 64],
}

impl PerpMarket {
    pub const DISCRIMINATOR: u64 = PERP_MARKET_DISCRIMINATOR;
    
    /// 获取 symbol 字符串
    pub fn symbol_str(&self) -> &str {
        let len = self.symbol.iter().position(|&c| c == 0).unwrap_or(16);
        std::str::from_utf8(&self.symbol[..len]).unwrap_or("")
    }
}

/// Perp 市场上架提案 (PLP-3)
/// Seeds: ["perp_proposal", proposer, nonce.to_le_bytes()]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PerpMarketProposal {
    /// 账户类型标识符
    pub discriminator: u64,
    /// 版本号
    pub version: u8,
    /// 申请者
    pub proposer: Pubkey,
    /// 唯一序号
    pub nonce: u64,
    /// Symbol
    pub symbol: [u8; 16],
    /// Base Token 索引
    pub base_token_index: u16,
    /// Quote Token 索引
    pub quote_token_index: u16,
    /// Oracle 地址
    pub oracle: Pubkey,
    /// 最小价格变动 (e6)
    pub tick_size_e6: u64,
    /// 最小数量变动 (e6)
    pub lot_size_e6: u64,
    /// 最大杠杆
    pub max_leverage: u8,
    /// 初始保证金率 (e6)
    pub initial_margin_rate_e6: u32,
    /// 维持保证金率 (e6)
    pub maintenance_margin_rate_e6: u32,
    /// Taker 费率 (bps)
    pub taker_fee_bps: u16,
    /// Maker 费率 (bps)
    pub maker_fee_bps: i16,
    /// 最小订单大小 (e6)
    pub min_order_size_e6: u64,
    /// 最大订单大小 (e6)
    pub max_order_size_e6: u64,
    /// 最大持仓量 (e6)
    pub max_open_interest_e6: u64,
    /// 保险金存款 (e6)
    pub insurance_fund_deposit_e6: u64,
    /// 质押金额
    pub stake_amount: u64,
    /// 提案状态
    pub status: ProposalStatus,
    /// 创建时间戳
    pub created_at: i64,
    /// 审核截止时间
    pub review_deadline: i64,
    /// 反对数量
    pub objection_count: u16,
    /// 反对质押总额
    pub objection_stake: u64,
    /// 质押是否已取回
    pub stake_claimed: bool,
    /// PDA bump
    pub bump: u8,
    /// 预留空间
    pub reserved: [u8; 64],
}

impl PerpMarketProposal {
    pub const DISCRIMINATOR: u64 = PERP_PROPOSAL_DISCRIMINATOR;
}

/// 初始流动性池 (PLP-4)
/// Seeds: ["plp4_pool", market]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct LiquidityPool {
    /// 账户类型标识符
    pub discriminator: u64,
    /// 版本号
    pub version: u8,
    /// 市场类型
    pub market_type: MarketType,
    /// 市场索引
    pub market_index: u16,
    /// Nonce (用于唯一标识)
    pub nonce: u64,
    /// 创建者
    pub creator: Pubkey,
    /// 关联的市场 PDA
    pub market: Pubkey,
    /// Base Token 余额 (e6)
    pub base_amount_e6: u64,
    /// Quote Token 余额 (e6)
    pub quote_amount_e6: u64,
    /// LP Token 总供应量 (e6)
    pub lp_token_supply_e6: u64,
    /// 做市价格下限 (e6)
    pub price_lower_e6: u64,
    /// 做市价格上限 (e6)
    pub price_upper_e6: u64,
    /// 订单密度（每个价格档位的订单数）
    pub order_density: u16,
    /// 价差 (bps)
    pub spread_bps: u64,
    /// 是否激活
    pub is_active: bool,
    /// 创建时间戳
    pub created_at: i64,
    /// 解锁时间戳
    pub unlock_time: i64,
    /// 退休时间戳（0 = 永不退休）
    pub retire_at: i64,
    /// PDA bump
    pub bump: u8,
    /// 预留空间
    pub reserved: [u8; 64],
}

impl LiquidityPool {
    pub const DISCRIMINATOR: u64 = LIQUIDITY_POOL_DISCRIMINATOR;
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_status_default() {
        let status = ProposalStatus::default();
        assert_eq!(status, ProposalStatus::Pending);
    }

    #[test]
    fn test_token_registry_symbol() {
        let mut token = TokenRegistry {
            discriminator: TokenRegistry::DISCRIMINATOR,
            version: 1,
            token_index: 0,
            symbol: [0u8; 8],
            mint: Pubkey::default(),
            decimals: 6,
            oracle: None,
            is_active: true,
            proposer: Pubkey::default(),
            approved_at: 0,
            bump: 255,
            reserved: [0u8; 64],
        };
        
        // Set symbol to "BTC"
        token.symbol[..3].copy_from_slice(b"BTC");
        assert_eq!(token.symbol_str(), "BTC");
    }

    #[test]
    fn test_spot_market_symbol() {
        let mut market = SpotMarket {
            discriminator: SpotMarket::DISCRIMINATOR,
            version: 1,
            market_index: 0,
            symbol: [0u8; 16],
            base_token_index: 0,
            quote_token_index: 1,
            tick_size_e6: 1000,
            lot_size_e6: 1000,
            taker_fee_bps: 10,
            maker_fee_bps: 0,
            min_order_size_e6: 1_000_000,
            max_order_size_e6: 1_000_000_000_000,
            is_active: true,
            is_paused: false,
            proposer: Pubkey::default(),
            approved_at: 0,
            bump: 255,
            reserved: [0u8; 64],
        };
        
        // Set symbol to "BTC/USDC"
        market.symbol[..8].copy_from_slice(b"BTC/USDC");
        assert_eq!(market.symbol_str(), "BTC/USDC");
    }

    #[test]
    fn test_listing_config_defaults() {
        // N1024 has 9 decimals, so 1,000 N1024 = 1e12 lamports
        assert_eq!(ListingConfig::DEFAULT_TOKEN_STAKE, 1_000_000_000_000);
        assert_eq!(ListingConfig::DEFAULT_SPOT_STAKE, 2_000_000_000_000);
        assert_eq!(ListingConfig::DEFAULT_PERP_STAKE, 5_000_000_000_000);
        assert_eq!(ListingConfig::DEFAULT_TOKEN_REVIEW_PERIOD, 604_800);
        assert_eq!(ListingConfig::DEFAULT_PERP_REVIEW_PERIOD, 1_209_600);
        assert_eq!(ListingConfig::DEFAULT_STAKE_LOCK_PERIOD, 2_592_000);
    }
}

