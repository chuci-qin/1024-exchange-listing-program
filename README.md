# 1024 Exchange Listing Program

> **Permissionless Listing Protocol (PLP)** - 去中心化市场上架程序

---

## 📋 概述

1024 Exchange Listing Program 实现了 PLP (Permissionless Listing Protocol) 协议体系，允许任何人无需许可地：

- **PLP-1**: 注册新的 Token
- **PLP-2**: 上架 Spot 交易市场
- **PLP-3**: 上架 Perp 永续合约市场
- **PLP-4**: 为新市场提供初始流动性

---

## 🏗️ 架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    1024-exchange-listing-program                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐              │
│   │   PLP-1   │ │   PLP-2   │ │   PLP-3   │ │   PLP-4   │              │
│   │  Token    │ │   Spot    │ │   Perp    │ │ Liquidity │              │
│   │ Registry  │ │  Listing  │ │  Listing  │ │   Pool    │              │
│   └───────────┘ └───────────┘ └───────────┘ └───────────┘              │
│                                                                         │
│   共享模块: ListingConfig, ProposalStatus, 质押/罚没逻辑                │
└─────────────────────────────────────────────────────────────────────────┘
                                  │
                                  │ CPI
                                  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│ 1024-vault-program     │ 1024-fund-program   │ 1024-ledger-program      │
│ (Spot 余额管理)        │ (保险金/手续费)     │ (Perp 仓位)              │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 📦 PDA 账户

| PDA | Seeds | 说明 |
|-----|-------|------|
| `ListingConfig` | `["listing_config"]` | 全局配置 |
| `TokenRegistry` | `["token", token_index]` | 已注册 Token |
| `TokenProposal` | `["token_proposal", proposer, nonce]` | Token 注册提案 |
| `SpotMarket` | `["spot_market", market_index]` | Spot 市场配置 |
| `SpotMarketProposal` | `["spot_proposal", proposer, nonce]` | Spot 市场提案 |
| `PerpMarket` | `["perp_market", market_index]` | Perp 市场配置 |
| `PerpMarketProposal` | `["perp_proposal", proposer, nonce]` | Perp 市场提案 |
| `LiquidityPool` | `["plp4_pool", market]` | 初始流动性池 |

---

## 📜 指令列表

### Admin 指令 (0-9)

| Index | 指令 | 说明 |
|-------|------|------|
| 0 | `Initialize` | 初始化全局配置 |
| 1 | `UpdateAdmin` | 更新管理员 |
| 2 | `UpdateStakeConfig` | 更新质押金额配置 |
| 3 | `UpdateReviewPeriods` | 更新审核期配置 |
| 4 | `SetPaused` | 暂停/恢复上架 |

### PLP-1: Token 注册 (10-19)

| Index | 指令 | 权限 | 说明 |
|-------|------|------|------|
| 10 | `ProposeToken` | Anyone | 提交 Token 注册申请 |
| 11 | `ObjectToken` | Anyone | 反对申请 |
| 12 | `ApproveToken` | Admin | 批准 Token |
| 13 | `RejectToken` | Admin | 拒绝 Token |
| 14 | `CancelTokenProposal` | Proposer | 撤回申请 |
| 15 | `FinalizeToken` | Anyone | 超时自动批准 |
| 16 | `ClaimTokenStake` | Proposer | 取回质押 |
| 17 | `UpdateTokenStatus` | Admin | 更新 Token 状态 |

### PLP-2: Spot 市场上架 (20-29)

| Index | 指令 | 权限 | 说明 |
|-------|------|------|------|
| 20 | `ProposeSpotMarket` | Anyone | 提交 Spot 市场申请 |
| 21 | `ObjectSpotMarket` | Anyone | 反对申请 |
| 22 | `ApproveSpotMarket` | Admin | 批准市场 |
| 23 | `RejectSpotMarket` | Admin | 拒绝市场 |
| 24 | `CancelSpotMarketProposal` | Proposer | 撤回申请 |
| 25 | `FinalizeSpotMarket` | Anyone | 超时自动批准 |
| 26 | `ClaimSpotMarketStake` | Proposer | 取回质押 |
| 27 | `UpdateSpotMarketStatus` | Admin | 更新市场状态 |
| 28 | `UpdateSpotMarketParams` | Admin | 更新市场参数 |

### PLP-3: Perp 市场上架 (30-39)

| Index | 指令 | 权限 | 说明 |
|-------|------|------|------|
| 30 | `ProposePerpMarket` | Anyone | 提交 Perp 市场申请 |
| 31 | `ObjectPerpMarket` | Anyone | 反对申请 |
| 32 | `ApprovePerpMarket` | Admin | 批准市场 |
| 33 | `RejectPerpMarket` | Admin | 拒绝市场 |
| 34 | `CancelPerpMarketProposal` | Proposer | 撤回申请 |
| 35 | `FinalizePerpMarket` | Anyone | 超时自动批准 |
| 36 | `ClaimPerpMarketStake` | Proposer | 取回质押 |
| 37 | `UpdatePerpMarketStatus` | Admin | 更新市场状态 |
| 38 | `UpdatePerpMarketParams` | Admin | 更新市场参数 |

### PLP-4: 初始流动性池 (40-49)

| Index | 指令 | 权限 | 说明 |
|-------|------|------|------|
| 40 | `InitializeLiquidityPool` | Admin/Proposer | 初始化流动性池 |
| 41 | `FundLiquidityPool` | Anyone | 注入流动性 |
| 42 | `AdjustLiquidityPoolParams` | Admin | 调整参数 |
| 43 | `RefreshLiquidityPoolOrders` | Relayer | 刷新订单 |
| 44 | `WithdrawLiquidityPoolProfit` | Admin | 提取收益 |
| 45 | `RetireLiquidityPool` | Admin | 退休池 |

---

## ⚙️ 配置参数

### 质押要求

| PLP | 质押金额 | 审核期 |
|-----|----------|--------|
| PLP-1 (Token) | 1,000 1024 | 7 天 |
| PLP-2 (Spot) | 2,000 1024 | 7 天 |
| PLP-3 (Perp) | 5,000 1024 + 保险金 | 14 天 |

### 质押锁定期

批准后 **30 天** 方可取回质押。

### 罚没规则

| 情况 | 罚没比例 |
|------|----------|
| 自行取消 | 5% |
| Admin 拒绝 (轻微违规) | 10% |
| Admin 拒绝 (恶意行为) | 50% |
| Admin 拒绝 (欺诈) | 100% |

---

## 🔧 开发

### 编译

```bash
cd onchain-program/1024-exchange-listing-program
cargo build-sbf
```

### 测试

```bash
cargo test
```

### 部署

```bash
# 生成 keypair
solana-keygen new -o target/deploy/listing_program-keypair.json

# 部署到 1024Chain Testnet
solana program deploy \
  --url https://testnet-rpc.1024chain.com/rpc/ \
  --keypair ~/1024chain-testnet/keys/faucet.json \
  target/deploy/listing_program.so
```

---

## 📝 Symbol 命名规范

| 市场类型 | 分隔符 | 格式 | 示例 |
|----------|--------|------|------|
| **Spot** | `/` 斜杠 | `{BASE}/{QUOTE}` | `BTC/USDC` |
| **Perp** | `-` 横线 | `{BASE}-{QUOTE}` | `BTC-USDC` |

---

## 📚 相关文档

- [PLP-PROTOCOL.md](../../1024-docs/real-spot-n-perp-market/PLP-PROTOCOL.md) - PLP 协议完整规范
- [ONCHAIN-PROGRAMS.md](../../1024-docs/real-spot-n-perp-market/ONCHAIN-PROGRAMS.md) - 链上程序架构
- [IMPLEMENTATION-TRACKER.md](../../1024-docs/real-spot-n-perp-market/IMPLEMENTATION-TRACKER.md) - 开发进度追踪

---

**Version**: 1.0.0  
**Last Updated**: 2025-12-22

