//! 1024 DEX Listing Program
//!
//! Permissionless Listing Protocol (PLP) 实现
//!
//! ## 协议体系
//! - **PLP-1**: Token Registry - Token 注册
//! - **PLP-2**: Spot Listing - Spot 市场上架
//! - **PLP-3**: Perp Listing - Perp 市场上架
//! - **PLP-4**: Initial Liquidity - 初始流动性池
//!
//! ## 核心功能
//! - 任何人可质押申请注册新 Token
//! - 任何人可质押申请上架 Spot/Perp 市场
//! - 审核期 + 反对机制
//! - 罚没和质押返还逻辑
//! - 初始流动性池自动做市

use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod utils;

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

/// Program entrypoint
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    processor::process_instruction(program_id, accounts, instruction_data)
}

