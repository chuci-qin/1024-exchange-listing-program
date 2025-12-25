//! Listing Program Instructions
//!
//! PLP (Permissionless Listing Protocol) 指令定义
//!
//! ## 指令分类
//! - **Admin**: 初始化和管理指令
//! - **PLP-1**: Token 注册指令
//! - **PLP-2**: Spot 市场上架指令
//! - **PLP-3**: Perp 市场上架指令
//! - **PLP-4**: 初始流动性池指令

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// Listing Program 指令
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum ListingInstruction {
    // =========================================================================
    // Admin 指令 (0-9)
    // =========================================================================
    
    /// [0] 初始化 Listing 全局配置
    ///
    /// 使用原生 N1024 (lamports) 进行质押，不需要 SPL Token Mint。
    ///
    /// Accounts:
    /// 0. `[signer, writable]` Admin (payer)
    /// 1. `[writable]` ListingConfig PDA
    /// 2. `[writable]` Treasury PDA (接收原生 N1024 质押)
    /// 3. `[]` System Program
    Initialize {
        /// Vault Program ID
        vault_program: Pubkey,
        /// Fund Program ID
        fund_program: Pubkey,
        /// Ledger Program ID
        ledger_program: Pubkey,
    },

    /// [1] 更新 Admin
    ///
    /// Accounts:
    /// 0. `[signer]` Current Admin
    /// 1. `[writable]` ListingConfig PDA
    UpdateAdmin {
        /// 新管理员
        new_admin: Pubkey,
    },

    /// [2] 更新质押金额配置
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` ListingConfig PDA
    UpdateStakeConfig {
        /// PLP-1 Token 质押金额
        token_stake_amount: Option<u64>,
        /// PLP-2 Spot 质押金额
        spot_stake_amount: Option<u64>,
        /// PLP-3 Perp 质押金额
        perp_stake_amount: Option<u64>,
    },

    /// [3] 更新审核期配置
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` ListingConfig PDA
    UpdateReviewPeriods {
        /// Token 审核期（秒）
        token_review_period: Option<u32>,
        /// Spot 审核期（秒）
        spot_review_period: Option<u32>,
        /// Perp 审核期（秒）
        perp_review_period: Option<u32>,
        /// 质押锁定期（秒）
        stake_lock_period: Option<u32>,
    },

    /// [4] 暂停/恢复上架
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` ListingConfig PDA
    SetPaused {
        /// 是否暂停
        paused: bool,
    },

    // =========================================================================
    // PLP-1: Token 注册指令 (10-19)
    // =========================================================================

    /// [10] 提交 Token 注册申请
    ///
    /// 使用原生 N1024 (lamports) 进行质押。
    ///
    /// Accounts:
    /// 0. `[signer, writable]` Proposer (payer, 扣除 N1024 质押)
    /// 1. `[writable]` TokenProposal PDA
    /// 2. `[]` ListingConfig PDA
    /// 3. `[writable]` Treasury PDA (接收 N1024 质押)
    /// 4. `[]` Token Mint (验证存在)
    /// 5. `[]` Oracle (可选)
    /// 6. `[]` System Program
    ProposeToken {
        /// 唯一序号
        nonce: u64,
        /// Symbol (2-8 字符)
        symbol: [u8; 8],
        /// Token Mint 地址
        mint: Pubkey,
        /// 精度
        decimals: u8,
        /// Oracle 地址（可选）
        oracle: Option<Pubkey>,
    },

    /// [11] 反对 Token 注册
    ///
    /// Accounts:
    /// 0. `[signer, writable]` Objector (payer)
    /// 1. `[writable]` TokenProposal PDA
    /// 2. `[]` ListingConfig PDA
    /// 3. `[writable]` Objector Stake Token Account
    /// 4. `[writable]` Treasury Stake Token Account
    /// 5. `[]` Token Program
    ObjectToken {
        /// 反对质押金额
        stake_amount: u64,
    },

    /// [12] Admin 批准 Token 注册
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` TokenProposal PDA
    /// 2. `[writable]` TokenRegistry PDA
    /// 3. `[writable]` ListingConfig PDA
    /// 4. `[]` System Program
    ApproveToken,

    /// [13] Admin 拒绝 Token 注册
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` TokenProposal PDA
    /// 2. `[]` ListingConfig PDA
    /// 3. `[writable]` Treasury Stake Token Account (罚没)
    RejectToken {
        /// 拒绝原因代码
        reason_code: u8,
        /// 罚没比例 (0-100)
        slash_percentage: u8,
    },

    /// [14] Proposer 取消 Token 提案
    ///
    /// Accounts:
    /// 0. `[signer]` Proposer
    /// 1. `[writable]` TokenProposal PDA
    /// 2. `[]` ListingConfig PDA
    /// 3. `[writable]` Proposer Stake Token Account
    /// 4. `[writable]` Treasury Stake Token Account
    /// 5. `[]` Token Program
    CancelTokenProposal,

    /// [15] 超时自动批准 Token
    ///
    /// Accounts:
    /// 0. `[signer, writable]` Caller (payer)
    /// 1. `[writable]` TokenProposal PDA
    /// 2. `[writable]` TokenRegistry PDA
    /// 3. `[writable]` ListingConfig PDA
    /// 4. `[]` System Program
    FinalizeToken,

    /// [16] Proposer 取回 Token 质押
    ///
    /// Accounts:
    /// 0. `[signer]` Proposer
    /// 1. `[writable]` TokenProposal PDA
    /// 2. `[]` TokenRegistry PDA (验证已批准)
    /// 3. `[]` ListingConfig PDA
    /// 4. `[writable]` Proposer Stake Token Account
    /// 5. `[writable]` Treasury Stake Token Account
    /// 6. `[]` Token Program
    ClaimTokenStake,

    /// [17] Admin 更新 Token 状态
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` TokenRegistry PDA
    /// 2. `[]` ListingConfig PDA
    UpdateTokenStatus {
        /// 是否激活
        is_active: bool,
    },

    // =========================================================================
    // PLP-2: Spot 市场上架指令 (20-29)
    // =========================================================================

    /// [20] 提交 Spot 市场上架申请
    ///
    /// Accounts:
    /// 0. `[signer, writable]` Proposer (payer)
    /// 1. `[writable]` SpotMarketProposal PDA
    /// 2. `[]` ListingConfig PDA
    /// 3. `[]` Base TokenRegistry PDA
    /// 4. `[]` Quote TokenRegistry PDA
    /// 5. `[writable]` Proposer Stake Token Account
    /// 6. `[writable]` Treasury Stake Token Account
    /// 7. `[]` Token Program
    /// 8. `[]` System Program
    ProposeSpotMarket {
        /// 唯一序号
        nonce: u64,
        /// Symbol (e.g., "BTC/USDC")
        symbol: [u8; 16],
        /// Base Token 索引
        base_token_index: u16,
        /// Quote Token 索引
        quote_token_index: u16,
        /// 最小价格变动 (e6)
        tick_size_e6: u64,
        /// 最小数量变动 (e6)
        lot_size_e6: u64,
        /// Taker 费率 (bps)
        taker_fee_bps: u16,
        /// Maker 费率 (bps)
        maker_fee_bps: i16,
        /// 最小订单大小 (e6)
        min_order_size_e6: u64,
        /// 最大订单大小 (e6)
        max_order_size_e6: u64,
    },

    /// [21] 反对 Spot 市场上架
    ///
    /// Accounts:
    /// 0. `[signer, writable]` Objector
    /// 1. `[writable]` SpotMarketProposal PDA
    /// 2. `[]` ListingConfig PDA
    /// 3. `[writable]` Objector Stake Token Account
    /// 4. `[writable]` Treasury Stake Token Account
    /// 5. `[]` Token Program
    ObjectSpotMarket {
        /// 反对质押金额
        stake_amount: u64,
    },

    /// [22] Admin 批准 Spot 市场
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` SpotMarketProposal PDA
    /// 2. `[writable]` SpotMarket PDA
    /// 3. `[writable]` ListingConfig PDA
    /// 4. `[]` System Program
    ApproveSpotMarket,

    /// [23] Admin 拒绝 Spot 市场
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` SpotMarketProposal PDA
    /// 2. `[]` ListingConfig PDA
    RejectSpotMarket {
        /// 拒绝原因代码
        reason_code: u8,
        /// 罚没比例 (0-100)
        slash_percentage: u8,
    },

    /// [24] Proposer 取消 Spot 提案
    ///
    /// Accounts:
    /// 0. `[signer]` Proposer
    /// 1. `[writable]` SpotMarketProposal PDA
    /// 2. `[]` ListingConfig PDA
    /// 3. `[writable]` Proposer Stake Token Account
    /// 4. `[writable]` Treasury Stake Token Account
    /// 5. `[]` Token Program
    CancelSpotMarketProposal,

    /// [25] 超时自动批准 Spot 市场
    ///
    /// Accounts:
    /// 0. `[signer, writable]` Caller
    /// 1. `[writable]` SpotMarketProposal PDA
    /// 2. `[writable]` SpotMarket PDA
    /// 3. `[writable]` ListingConfig PDA
    /// 4. `[]` System Program
    FinalizeSpotMarket,

    /// [26] Proposer 取回 Spot 质押
    ///
    /// Accounts:
    /// 0. `[signer]` Proposer
    /// 1. `[writable]` SpotMarketProposal PDA
    /// 2. `[]` SpotMarket PDA
    /// 3. `[]` ListingConfig PDA
    /// 4. `[writable]` Proposer Stake Token Account
    /// 5. `[writable]` Treasury Stake Token Account
    /// 6. `[]` Token Program
    ClaimSpotMarketStake,

    /// [27] Admin 更新 Spot 市场状态
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` SpotMarket PDA
    /// 2. `[]` ListingConfig PDA
    UpdateSpotMarketStatus {
        /// 是否激活
        is_active: Option<bool>,
        /// 是否暂停
        is_paused: Option<bool>,
    },

    /// [28] Admin 更新 Spot 市场参数
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` SpotMarket PDA
    /// 2. `[]` ListingConfig PDA
    UpdateSpotMarketParams {
        /// Taker 费率 (bps)
        taker_fee_bps: Option<u16>,
        /// Maker 费率 (bps)
        maker_fee_bps: Option<i16>,
        /// 最小订单大小 (e6)
        min_order_size_e6: Option<u64>,
        /// 最大订单大小 (e6)
        max_order_size_e6: Option<u64>,
    },

    // =========================================================================
    // PLP-3: Perp 市场上架指令 (30-39)
    // =========================================================================

    /// [30] 提交 Perp 市场上架申请
    ///
    /// Accounts:
    /// 0. `[signer, writable]` Proposer (payer)
    /// 1. `[writable]` PerpMarketProposal PDA
    /// 2. `[]` ListingConfig PDA
    /// 3. `[]` Base TokenRegistry PDA
    /// 4. `[]` Quote TokenRegistry PDA
    /// 5. `[]` Oracle Account (验证)
    /// 6. `[writable]` Proposer Stake Token Account
    /// 7. `[writable]` Treasury Stake Token Account
    /// 8. `[]` Token Program
    /// 9. `[]` System Program
    ProposePerpMarket {
        /// 唯一序号
        nonce: u64,
        /// Symbol (e.g., "BTC-USDC")
        symbol: [u8; 16],
        /// Base Token 索引
        base_token_index: u16,
        /// Quote Token 索引
        quote_token_index: u16,
        /// Oracle 地址
        oracle: Pubkey,
        /// 最小价格变动 (e6)
        tick_size_e6: u64,
        /// 最小数量变动 (e6)
        lot_size_e6: u64,
        /// 最大杠杆 (1-100)
        max_leverage: u8,
        /// 初始保证金率 (e6)
        initial_margin_rate_e6: u32,
        /// 维持保证金率 (e6)
        maintenance_margin_rate_e6: u32,
        /// Taker 费率 (bps)
        taker_fee_bps: u16,
        /// Maker 费率 (bps)
        maker_fee_bps: i16,
        /// 最小订单大小 (e6)
        min_order_size_e6: u64,
        /// 最大订单大小 (e6)
        max_order_size_e6: u64,
        /// 最大持仓量 (e6)
        max_open_interest_e6: u64,
        /// 保险金存款 (e6)
        insurance_fund_deposit_e6: u64,
    },

    /// [31] 反对 Perp 市场上架
    ///
    /// Accounts:
    /// 0. `[signer, writable]` Objector
    /// 1. `[writable]` PerpMarketProposal PDA
    /// 2. `[]` ListingConfig PDA
    /// 3. `[writable]` Objector Stake Token Account
    /// 4. `[writable]` Treasury Stake Token Account
    /// 5. `[]` Token Program
    ObjectPerpMarket {
        /// 反对质押金额
        stake_amount: u64,
    },

    /// [32] Admin 批准 Perp 市场
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` PerpMarketProposal PDA
    /// 2. `[writable]` PerpMarket PDA
    /// 3. `[writable]` ListingConfig PDA
    /// 4. `[]` Fund Program (CPI 存入保险金)
    /// 5. `[writable]` Proposer USDC Token Account
    /// 6. `[writable]` Insurance Fund Token Account
    /// 7. `[]` Token Program
    /// 8. `[]` System Program
    ApprovePerpMarket,

    /// [33] Admin 拒绝 Perp 市场
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` PerpMarketProposal PDA
    /// 2. `[]` ListingConfig PDA
    RejectPerpMarket {
        /// 拒绝原因代码
        reason_code: u8,
        /// 罚没比例 (0-100)
        slash_percentage: u8,
    },

    /// [34] Proposer 取消 Perp 提案
    ///
    /// Accounts:
    /// 0. `[signer]` Proposer
    /// 1. `[writable]` PerpMarketProposal PDA
    /// 2. `[]` ListingConfig PDA
    /// 3. `[writable]` Proposer Stake Token Account
    /// 4. `[writable]` Treasury Stake Token Account
    /// 5. `[]` Token Program
    CancelPerpMarketProposal,

    /// [35] 超时自动批准 Perp 市场
    ///
    /// Accounts:
    /// 0. `[signer, writable]` Caller
    /// 1. `[writable]` PerpMarketProposal PDA
    /// 2. `[writable]` PerpMarket PDA
    /// 3. `[writable]` ListingConfig PDA
    /// 4. `[]` Fund Program (CPI)
    /// 5. `[writable]` Proposer USDC Token Account
    /// 6. `[writable]` Insurance Fund Token Account
    /// 7. `[]` Token Program
    /// 8. `[]` System Program
    FinalizePerpMarket,

    /// [36] Proposer 取回 Perp 质押
    ///
    /// Accounts:
    /// 0. `[signer]` Proposer
    /// 1. `[writable]` PerpMarketProposal PDA
    /// 2. `[]` PerpMarket PDA
    /// 3. `[]` ListingConfig PDA
    /// 4. `[writable]` Proposer Stake Token Account
    /// 5. `[writable]` Treasury Stake Token Account
    /// 6. `[]` Token Program
    ClaimPerpMarketStake,

    /// [37] Admin 更新 Perp 市场状态
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` PerpMarket PDA
    /// 2. `[]` ListingConfig PDA
    UpdatePerpMarketStatus {
        /// 是否激活
        is_active: Option<bool>,
        /// 是否暂停
        is_paused: Option<bool>,
    },

    /// [38] Admin 更新 Perp 市场参数
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` PerpMarket PDA
    /// 2. `[]` ListingConfig PDA
    UpdatePerpMarketParams {
        /// 最大杠杆
        max_leverage: Option<u8>,
        /// 初始保证金率 (e6)
        initial_margin_rate_e6: Option<u32>,
        /// 维持保证金率 (e6)
        maintenance_margin_rate_e6: Option<u32>,
        /// Taker 费率 (bps)
        taker_fee_bps: Option<u16>,
        /// Maker 费率 (bps)
        maker_fee_bps: Option<i16>,
        /// 最大持仓量 (e6)
        max_open_interest_e6: Option<u64>,
    },

    // =========================================================================
    // PLP-4: 初始流动性池指令 (40-49)
    // =========================================================================

    /// [40] 初始化 PLP-4 流动性池
    ///
    /// 市场批准后由 Admin 或 Proposer 调用
    ///
    /// Accounts:
    /// 0. `[signer, writable]` Initializer (payer)
    /// 1. `[writable]` LiquidityPool PDA
    /// 2. `[]` Market PDA (SpotMarket or PerpMarket)
    /// 3. `[]` ListingConfig PDA
    /// 4. `[]` System Program
    InitializeLiquidityPool {
        /// 市场类型
        market_type: u8, // 0=Spot, 1=Perp
        /// 做市价格下限 (e6)
        price_lower_e6: u64,
        /// 做市价格上限 (e6)
        price_upper_e6: u64,
        /// 订单密度
        order_density: u16,
        /// 价差 (bps)
        spread_bps: u64,
    },

    /// [41] 向流动性池注入资金
    ///
    /// Accounts:
    /// 0. `[signer, writable]` Funder (payer)
    /// 1. `[writable]` LiquidityPool PDA
    /// 2. `[]` ListingConfig PDA
    /// 3. `[writable]` Funder Base Token Account
    /// 4. `[writable]` Funder Quote Token Account
    /// 5. `[writable]` Pool Base Token Account
    /// 6. `[writable]` Pool Quote Token Account
    /// 7. `[]` Token Program
    FundLiquidityPool {
        /// Base Token 数量 (e6)
        base_amount_e6: u64,
        /// Quote Token 数量 (e6)
        quote_amount_e6: u64,
    },

    /// [42] 调整流动性池参数
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` LiquidityPool PDA
    /// 2. `[]` ListingConfig PDA
    AdjustLiquidityPoolParams {
        /// 做市价格下限 (e6)
        price_lower_e6: Option<u64>,
        /// 做市价格上限 (e6)
        price_upper_e6: Option<u64>,
        /// 订单密度
        order_density: Option<u16>,
        /// 价差 (bps)
        spread_bps: Option<u64>,
    },

    /// [43] 刷新流动性池订单 (由 Relayer 调用)
    ///
    /// Accounts:
    /// 0. `[signer]` Relayer
    /// 1. `[]` LiquidityPool PDA
    /// 2. `[]` Market PDA
    /// 3. `[]` ListingConfig PDA
    RefreshLiquidityPoolOrders,

    /// [44] 提取流动性池收益
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` LiquidityPool PDA
    /// 2. `[]` ListingConfig PDA
    /// 3. `[writable]` Pool Base Token Account
    /// 4. `[writable]` Pool Quote Token Account
    /// 5. `[writable]` Treasury Base Token Account
    /// 6. `[writable]` Treasury Quote Token Account
    /// 7. `[]` Token Program
    WithdrawLiquidityPoolProfit {
        /// Base Token 数量 (e6)
        base_amount_e6: u64,
        /// Quote Token 数量 (e6)
        quote_amount_e6: u64,
    },

    /// [45] 退休流动性池
    ///
    /// Accounts:
    /// 0. `[signer]` Admin
    /// 1. `[writable]` LiquidityPool PDA
    /// 2. `[]` ListingConfig PDA
    RetireLiquidityPool,

    // =========================================================================
    // Query 指令 (50-59) - 链上查询接口
    // =========================================================================

    /// [50] 获取 Token 信息 (No-op, 用于 simulate)
    QueryToken {
        /// Token 索引
        token_index: u16,
    },

    /// [51] 获取 Spot 市场信息
    QuerySpotMarket {
        /// 市场索引
        market_index: u16,
    },

    /// [52] 获取 Perp 市场信息
    QueryPerpMarket {
        /// 市场索引
        market_index: u16,
    },
}

