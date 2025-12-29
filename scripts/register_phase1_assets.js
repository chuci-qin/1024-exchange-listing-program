/**
 * 1024 Exchange Listing Program - Register Phase 1 Assets
 * 
 * This script registers the 12 Phase 1 assets using the full PLP flow:
 * 1. ProposeToken (instruction 10) - Submit token proposal
 * 2. ApproveToken (instruction 12) - Admin approves proposal
 * 3. ProposePerpMarket (instruction 30) - Submit perp market proposal
 * 4. ApprovePerpMarket (instruction 32) - Admin approves perp market
 * 
 * Phase 1 Assets:
 * - MAG7 Stocks: AAPL, MSFT, GOOGL, AMZN, META, NVDA, TSLA
 * - Index ETFs: SPY (S&P 500), QQQ (Nasdaq 100)
 * - Metals: XAU (Gold), XAG (Silver)
 * - Energy: WTI (Crude Oil)
 * 
 * Usage:
 *   node register_phase1_assets.js [--keypair <path>]
 */

const {
    Connection,
    Keypair,
    PublicKey,
    Transaction,
    TransactionInstruction,
    SystemProgram,
} = require('@solana/web3.js');
const fs = require('fs');
const borsh = require('borsh');

// ============================================================================
// Configuration
// ============================================================================

const RPC_URL = 'https://testnet-rpc.1024chain.com/rpc/';
const LISTING_PROGRAM_ID = new PublicKey('41QWGy3LpKcjrVVgXnCpFRa45wthZCk91sjmp8DccZzq');

// PDA Seeds (from processor.rs)
// PDA Seeds (must match state.rs exactly)
const LISTING_CONFIG_SEED = Buffer.from('listing_config');
const LISTING_TREASURY_SEED = Buffer.from('listing_treasury');
const TOKEN_PROPOSAL_SEED = Buffer.from('token_proposal');
const TOKEN_REGISTRY_SEED = Buffer.from('token');  // b"token" in Rust
const PERP_PROPOSAL_SEED = Buffer.from('perp_proposal');
const PERP_MARKET_SEED = Buffer.from('perp_market');

// 1024Chain timing
const MAX_RETRIES = 5;
const CONFIRMATION_POLL_INTERVAL_MS = 300;
const CONFIRMATION_TIMEOUT_MS = 45000;

// ============================================================================
// Phase 1 Assets Configuration
// ============================================================================

/**
 * Phase 1 Token Registry
 * 
 * Note: These are "virtual" tokens for price tracking only.
 * They don't have real SPL Token mints.
 */
const PHASE1_TOKENS = [
    // === MAG7 Stocks ===
    { symbol: 'AAPL', decimals: 6, description: 'Apple Inc.' },
    { symbol: 'MSFT', decimals: 6, description: 'Microsoft Corp.' },
    { symbol: 'GOOGL', decimals: 6, description: 'Alphabet Inc.' },
    { symbol: 'AMZN', decimals: 6, description: 'Amazon.com Inc.' },
    { symbol: 'META', decimals: 6, description: 'Meta Platforms Inc.' },
    { symbol: 'NVDA', decimals: 6, description: 'NVIDIA Corp.' },
    { symbol: 'TSLA', decimals: 6, description: 'Tesla Inc.' },
    
    // === Index ETFs ===
    { symbol: 'SPY', decimals: 6, description: 'S&P 500 ETF' },
    { symbol: 'QQQ', decimals: 6, description: 'Nasdaq 100 ETF' },
    
    // === Metals ===
    { symbol: 'XAU', decimals: 6, description: 'Gold (Troy Ounce)' },
    { symbol: 'XAG', decimals: 6, description: 'Silver (Troy Ounce)' },
    
    // === Energy ===
    { symbol: 'WTI', decimals: 6, description: 'WTI Crude Oil (Barrel)' },
];

/**
 * Pyth Oracle Feed IDs
 */
const PYTH_FEED_IDS = {
    'AAPL': '0x49f6b65cb1de6b10eaf75e7c03ca029c306d0357e91b5311b175084a5ad55688',
    'MSFT': '0xd0ca23c1cc005e004ccf1db5bf76aeb6a49218f43dac3d4b275e92de12ded4d1',
    'GOOGL': '0x5a48c03e9b9cb337801073ed9d166817473697efff0d138874e0f6a33d6d5aa6',
    'AMZN': '0xb5d0e0fa58a1f8b81498ae670ce93c872d14434b72c364885d4fa1b257cbb07a',
    'META': '0x78a3e3b8e676a8f73c439f5d749737034b139bbbe899ba5775216fba596607fe',
    'NVDA': '0xb1073854ed24cbc755dc527418f52b7d271f6cc967bbf8d8129112b18860a593',
    'TSLA': '0x16dad506d7db8da01c87581c87ca897a012a153557d4d578c3b9c9e1bc0632f1',
    'SPY': '0x19e09bb805456ada3979a7d1cbb4b6d63babc3a0f8e8a9509f68afa5c4c11cd5',
    'QQQ': '0x9695e2b96ea7b3859da9ed25b7a46a920a776e2fdae19a7bcfdf2b219230452d',
    'XAU': '0x765d2ba906dbc32ca17cc11f5310a89e9ee1f6420508c63861f2f8ba4ee34bb2',
    'XAG': '0xf2fb02c32b055c805e7238d628e5e9dadef274376114eb1f012337cabe93871e',
    'WTI': '0x925ca92ff005ae943c158e3563f59698ce7e75c5a8c8dd43303a0a154887b3e6',
};

/**
 * Market configurations for Perp
 */
const PERP_CONFIGS = {
    // MAG7 Stocks: 10x leverage
    'AAPL': { maxLeverage: 10, tickSizeE6: 10_000, lotSizeE6: 1_000 },
    'MSFT': { maxLeverage: 10, tickSizeE6: 10_000, lotSizeE6: 1_000 },
    'GOOGL': { maxLeverage: 10, tickSizeE6: 10_000, lotSizeE6: 1_000 },
    'AMZN': { maxLeverage: 10, tickSizeE6: 10_000, lotSizeE6: 1_000 },
    'META': { maxLeverage: 10, tickSizeE6: 10_000, lotSizeE6: 1_000 },
    'NVDA': { maxLeverage: 10, tickSizeE6: 10_000, lotSizeE6: 1_000 },
    'TSLA': { maxLeverage: 10, tickSizeE6: 10_000, lotSizeE6: 1_000 },
    // Index ETFs: 20x leverage
    'SPY': { maxLeverage: 20, tickSizeE6: 10_000, lotSizeE6: 1_000 },
    'QQQ': { maxLeverage: 20, tickSizeE6: 10_000, lotSizeE6: 1_000 },
    // Metals: 20x leverage
    'XAU': { maxLeverage: 20, tickSizeE6: 100_000, lotSizeE6: 1_000 },
    'XAG': { maxLeverage: 20, tickSizeE6: 1_000, lotSizeE6: 1_000 },
    // Energy: 20x leverage
    'WTI': { maxLeverage: 20, tickSizeE6: 10_000, lotSizeE6: 1_000 },
};

// ============================================================================
// Main
// ============================================================================

async function main() {
    console.log('=== Register Phase 1 Assets & Perp Markets (PLP Flow) ===\n');
    console.log('Assets to register:');
    console.log('  - MAG7 Stocks: AAPL, MSFT, GOOGL, AMZN, META, NVDA, TSLA');
    console.log('  - Index ETFs:  SPY, QQQ');
    console.log('  - Metals:      XAU (Gold), XAG (Silver)');
    console.log('  - Energy:      WTI (Crude Oil)');
    console.log('');

    // Load keypair
    const keypairPath = process.argv.includes('--keypair')
        ? process.argv[process.argv.indexOf('--keypair') + 1]
        : '/Users/chuciqin/Desktop/project1024/1024codebase/1024-chain/keys/faucet.json';
    
    console.log('Loading keypair from:', keypairPath);
    const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'));
    const admin = Keypair.fromSecretKey(Uint8Array.from(keypairData));
    console.log('Admin:', admin.publicKey.toBase58());

    // Connect
    const connection = new Connection(RPC_URL, 'confirmed');
    const balance = await connection.getBalance(admin.publicKey);
    console.log('Balance:', balance / 1e9, 'N1024\n');

    // Derive ListingConfig PDA
    const [listingConfigPda] = PublicKey.findProgramAddressSync(
        [LISTING_CONFIG_SEED],
        LISTING_PROGRAM_ID
    );
    console.log('ListingConfig PDA:', listingConfigPda.toBase58());

    // Derive Treasury PDA
    const [treasuryPda] = PublicKey.findProgramAddressSync(
        [LISTING_TREASURY_SEED],
        LISTING_PROGRAM_ID
    );
    console.log('Treasury PDA:', treasuryPda.toBase58());

    // Check if ListingConfig exists
    const configInfo = await connection.getAccountInfo(listingConfigPda);
    if (!configInfo) {
        console.log('\n❌ ListingConfig not initialized!');
        console.log('   Run: node init_listing_config.js first');
        return;
    }
    console.log('✅ ListingConfig exists (', configInfo.data.length, 'bytes)\n');

    // Parse ListingConfig to get current token count
    const configData = configInfo.data;
    // Skip discriminator (8) + version (1) + admin (32) + paused (1)
    // total_tokens is at offset 8+1+32+1+4+4+4+4+4+4+4 = 66
    // Actually let's check structure...
    // According to state.rs, total_tokens is u16 at a specific offset
    // Let's just track token indices ourselves

    let tokenIndices = {};  // symbol -> token_index

    // ========================================================================
    // Step 1: Register Tokens via ProposeToken + ApproveToken
    // ========================================================================
    console.log('='.repeat(60));
    console.log('Step 1: Register Tokens (ProposeToken + ApproveToken)');
    console.log('='.repeat(60) + '\n');

    // First, check how many tokens are already registered
    // We'll probe token_registry PDAs starting from 0
    let existingTokenCount = 0;
    for (let i = 0; i < 100; i++) {
        const indexBuffer = Buffer.alloc(2);
        indexBuffer.writeUInt16LE(i);
        const [registryPda] = PublicKey.findProgramAddressSync(
            [TOKEN_REGISTRY_SEED, indexBuffer],
            LISTING_PROGRAM_ID
        );
        const info = await connection.getAccountInfo(registryPda);
        if (info) {
            existingTokenCount = i + 1;
            // Parse symbol from registry (offset: 8 discriminator + 1 version + 2 index = 11)
            const symbolBytes = info.data.slice(11, 19);
            const symbol = Buffer.from(symbolBytes).toString('utf8').replace(/\0/g, '');
            tokenIndices[symbol] = i;
            console.log(`  Token ${i}: ${symbol} (exists at ${registryPda.toBase58().substring(0, 12)}...)`);
        } else {
            break;
        }
    }
    console.log(`\nExisting tokens: ${existingTokenCount}\n`);

    for (let i = 0; i < PHASE1_TOKENS.length; i++) {
        const token = PHASE1_TOKENS[i];
        
        if (tokenIndices[token.symbol] !== undefined) {
            console.log(`⏭️ ${token.symbol} already registered (index=${tokenIndices[token.symbol]})`);
            continue;
        }
        
        const nonce = BigInt(Date.now()) + BigInt(i);
        const tokenIndex = await registerTokenPLP(
            connection,
            admin,
            listingConfigPda,
            treasuryPda,
            token,
            nonce
        );
        
        if (tokenIndex !== null) {
            tokenIndices[token.symbol] = tokenIndex;
        }
        
        // Small delay between transactions
        await sleep(500);
    }

    console.log('\nToken registration summary:');
    for (const [symbol, index] of Object.entries(tokenIndices)) {
        console.log(`  ${symbol}: index=${index}`);
    }

    // ========================================================================
    // Step 2: List Perp Markets
    // ========================================================================
    console.log('\n' + '='.repeat(60));
    console.log('Step 2: List Perp Markets (ProposePerpMarket + ApprovePerpMarket)');
    console.log('='.repeat(60) + '\n');

    // USDC token index is 0 (assumed to be pre-registered)
    const USDC_TOKEN_INDEX = 0;

    for (let i = 0; i < PHASE1_TOKENS.length; i++) {
        const token = PHASE1_TOKENS[i];
        const baseTokenIndex = tokenIndices[token.symbol];
        const marketSymbol = `${token.symbol}-USDC`;
        
        if (baseTokenIndex === undefined) {
            console.log(`⚠️ Skipping ${marketSymbol}: token not registered`);
            continue;
        }
        
        // Check if market already exists by searching existing markets
        const existingMarketIndex = await findExistingPerpMarket(connection, marketSymbol, listingConfigPda);
        if (existingMarketIndex !== null) {
            console.log(`⏭️ ${marketSymbol} already exists (market_index=${existingMarketIndex})`);
            continue;
        }
        
        // Use deterministic nonce based on symbol to allow resumption
        const symbolHash = token.symbol.split('').reduce((acc, c) => acc + c.charCodeAt(0), 0);
        const nonce = BigInt(symbolHash * 1000000);
        
        await listPerpMarketPLP(
            connection,
            admin,
            listingConfigPda,
            treasuryPda,
            token.symbol,
            baseTokenIndex,
            USDC_TOKEN_INDEX,
            nonce
        );
        
        // Small delay
        await sleep(500);
    }

    console.log('\n' + '='.repeat(60));
    console.log('✅ Phase 1 Registration Complete!');
    console.log('='.repeat(60));
    console.log('\nRegistered:');
    console.log(`  - ${Object.keys(tokenIndices).length} tokens`);
    console.log(`  - ${PHASE1_TOKENS.length} perp markets (pending approval)`);
}

// ============================================================================
// Token Registration via PLP
// ============================================================================

async function registerTokenPLP(connection, admin, listingConfigPda, treasuryPda, tokenConfig, nonce) {
    console.log(`\nRegistering token: ${tokenConfig.symbol}...`);

    // Step 1: ProposeToken (instruction 10)
    console.log('  1. Submitting ProposeToken...');
    
    // Derive TokenProposal PDA
    const nonceBuffer = Buffer.alloc(8);
    nonceBuffer.writeBigUInt64LE(nonce);
    
    const [proposalPda, proposalBump] = PublicKey.findProgramAddressSync(
        [TOKEN_PROPOSAL_SEED, admin.publicKey.toBuffer(), nonceBuffer],
        LISTING_PROGRAM_ID
    );

    // Check if proposal exists
    const proposalInfo = await connection.getAccountInfo(proposalPda);
    let proposalExists = proposalInfo !== null;

    if (!proposalExists) {
        // Build ProposeToken instruction
        // Index 10 in enum = ProposeToken
        const instructionData = buildProposeTokenData(nonce, tokenConfig);

        // Use a dummy mint for virtual tokens (system program address)
        const dummyMint = new PublicKey('11111111111111111111111111111111');
        
        // ProposeToken accounts (from processor.rs):
        // 0. proposer (signer, writable)
        // 1. proposal_account (writable)
        // 2. config_account
        // 3. treasury_account (writable)
        // 4. token_mint
        // 5. oracle_account (optional - we still need to pass something)
        // 6. system_program

        const proposeTx = new Transaction().add(
            new TransactionInstruction({
                programId: LISTING_PROGRAM_ID,
                keys: [
                    { pubkey: admin.publicKey, isSigner: true, isWritable: true },
                    { pubkey: proposalPda, isSigner: false, isWritable: true },
                    { pubkey: listingConfigPda, isSigner: false, isWritable: true },  // Needs write for total_proposals counter
                    { pubkey: treasuryPda, isSigner: false, isWritable: true },
                    { pubkey: dummyMint, isSigner: false, isWritable: false },  // token_mint
                    { pubkey: dummyMint, isSigner: false, isWritable: false },  // oracle (optional, but need to pass)
                    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
                ],
                data: instructionData,
            })
        );

        const proposeSig = await sendAndConfirmTransaction(connection, proposeTx, admin);
        if (!proposeSig) {
            console.log('  ❌ ProposeToken failed');
            return null;
        }
        console.log(`  ✅ ProposeToken: ${proposeSig.substring(0, 16)}...`);
        proposalExists = true;
    } else {
        console.log('  ⏭️ Proposal already exists');
    }

    // Step 2: ApproveToken (instruction 12)
    console.log('  2. Approving token...');

    // Get config to read total_tokens
    const configInfo = await connection.getAccountInfo(listingConfigPda);
    if (!configInfo) {
        console.log('  ❌ Config not found');
        return null;
    }
    
    // Parse total_tokens from ListingConfig (from state.rs)
    // Structure:
    // - discriminator: u64 (8)
    // - version: u8 (1)
    // - admin: Pubkey (32)
    // - treasury: Pubkey (32)
    // - vault_program: Pubkey (32)
    // - fund_program: Pubkey (32)
    // - ledger_program: Pubkey (32)
    // - token_stake_amount: u64 (8)
    // - spot_stake_amount: u64 (8)
    // - perp_stake_amount: u64 (8)
    // - token_review_period_seconds: u32 (4)
    // - spot_review_period_seconds: u32 (4)
    // - perp_review_period_seconds: u32 (4)
    // - stake_lock_period_seconds: u32 (4)
    // - total_tokens: u16 (2)
    // Offset = 8+1+32+32+32+32+32+8+8+8+4+4+4+4 = 209
    const totalTokensOffset = 209;
    const tokenIndex = configInfo.data.readUInt16LE(totalTokensOffset);
    console.log(`  Config total_tokens: ${tokenIndex}`);

    // Derive TokenRegistry PDA for this index
    const indexBuffer = Buffer.alloc(2);
    indexBuffer.writeUInt16LE(tokenIndex);
    const [registryPda] = PublicKey.findProgramAddressSync(
        [TOKEN_REGISTRY_SEED, indexBuffer],
        LISTING_PROGRAM_ID
    );

    // Check if already approved
    const registryInfo = await connection.getAccountInfo(registryPda);
    if (registryInfo) {
        console.log(`  ⏭️ Already approved at index ${tokenIndex}`);
        return tokenIndex;
    }

    // Build ApproveToken instruction
    // Enum variant index: 7 (Initialize=0, UpdateAdmin=1, UpdateStakeConfig=2, 
    //                        UpdateReviewPeriods=3, SetPaused=4, ProposeToken=5,
    //                        ObjectToken=6, ApproveToken=7)
    const APPROVE_TOKEN_INDEX = 7;
    const approveData = Buffer.alloc(1);
    approveData.writeUInt8(APPROVE_TOKEN_INDEX, 0);

    const approveTx = new Transaction().add(
        new TransactionInstruction({
            programId: LISTING_PROGRAM_ID,
            keys: [
                { pubkey: admin.publicKey, isSigner: true, isWritable: true },  // Admin needs to pay for registry creation
                { pubkey: proposalPda, isSigner: false, isWritable: true },
                { pubkey: registryPda, isSigner: false, isWritable: true },
                { pubkey: listingConfigPda, isSigner: false, isWritable: true },
                { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
            ],
            data: approveData,
        })
    );

    const approveSig = await sendAndConfirmTransaction(connection, approveTx, admin);
    if (!approveSig) {
        console.log('  ❌ ApproveToken failed');
        return null;
    }
    console.log(`  ✅ ApproveToken: ${approveSig.substring(0, 16)}... (index=${tokenIndex})`);
    return tokenIndex;
}

function buildProposeTokenData(nonce, tokenConfig) {
    // ProposeToken (instruction 10) - Borsh enum variant index
    // ProposeToken { nonce: u64, symbol: [u8; 8], mint: Pubkey, decimals: u8, oracle: Option<Pubkey> }
    //
    // Borsh enum serialization: 1 byte variant index + variant fields
    // Looking at instruction.rs, the enum variants are:
    //   0: Initialize
    //   1: UpdateAdmin
    //   2: UpdateStakeConfig
    //   3: UpdateReviewPeriods
    //   4: SetPaused
    //   5: ProposeToken  <-- NOT 10! Borsh uses sequential index
    //   ...
    // Wait, let me re-check: the comments say [10], [11], etc. but those are documentation IDs.
    // The actual Borsh variant index is the position in the enum definition.
    //
    // Actually: Initialize=0, UpdateAdmin=1, UpdateStakeConfig=2, UpdateReviewPeriods=3, SetPaused=4
    //           ProposeToken=5, ObjectToken=6, ApproveToken=7, RejectToken=8, ...
    // But the comments say [10] ProposeToken. Let me count the variants:
    // 0-4: Admin (5 variants)
    // 5-12: PLP-1 Token (8 variants: indices 5-12 in enum)
    // Wait, let me just count properly...
    
    // From instruction.rs enum order:
    // 0: Initialize
    // 1: UpdateAdmin
    // 2: UpdateStakeConfig
    // 3: UpdateReviewPeriods
    // 4: SetPaused
    // 5: ProposeToken
    // 6: ObjectToken
    // 7: ApproveToken
    // 8: RejectToken
    // 9: CancelTokenProposal
    // 10: FinalizeToken
    // 11: ClaimTokenStake
    // 12: UpdateTokenStatus
    // 13: ProposeSpotMarket
    // ... and so on
    
    const PROPOSE_TOKEN_INDEX = 5;  // Actual enum variant index
    
    const buffer = Buffer.alloc(1 + 8 + 8 + 32 + 1 + 1); // index + nonce + symbol + mint + decimals + option_tag
    let offset = 0;

    // Instruction variant index
    buffer.writeUInt8(PROPOSE_TOKEN_INDEX, offset); offset += 1;

    // nonce: u64
    buffer.writeBigUInt64LE(nonce, offset); offset += 8;

    // symbol: [u8; 8]
    const symbolBuffer = Buffer.alloc(8);
    symbolBuffer.write(tokenConfig.symbol);
    symbolBuffer.copy(buffer, offset); offset += 8;

    // mint: Pubkey (use system program as dummy for virtual tokens)
    const dummyMint = new PublicKey('11111111111111111111111111111111');
    dummyMint.toBuffer().copy(buffer, offset); offset += 32;

    // decimals: u8
    buffer.writeUInt8(tokenConfig.decimals, offset); offset += 1;

    // oracle: Option<Pubkey> = None
    buffer.writeUInt8(0, offset); offset += 1;

    return buffer.slice(0, offset);
}

// ============================================================================
// Perp Market Listing via PLP
// ============================================================================

/**
 * Find if a perp market already exists by checking all existing markets
 */
async function findExistingPerpMarket(connection, marketSymbol, listingConfigPda) {
    // Get total_perp_markets from config
    const configInfo = await connection.getAccountInfo(listingConfigPda);
    if (!configInfo) return null;
    
    const totalPerpMarkets = configInfo.data.readUInt16LE(213);
    
    // Check each existing market
    for (let i = 0; i < totalPerpMarkets; i++) {
        const indexBuf = Buffer.alloc(2);
        indexBuf.writeUInt16LE(i);
        const [marketPda] = PublicKey.findProgramAddressSync(
            [Buffer.from('perp_market'), indexBuf],
            LISTING_PROGRAM_ID
        );
        
        const marketInfo = await connection.getAccountInfo(marketPda);
        if (!marketInfo) continue;
        
        // Read symbol from market data
        // PerpMarket layout: discriminator(8) + version(1) + market_index(2) + symbol([u8;16])
        const symbolBytes = marketInfo.data.slice(11, 27);
        const existingSymbol = symbolBytes.toString('utf8').replace(/\0/g, '');
        
        if (existingSymbol === marketSymbol) {
            return i;
        }
    }
    
    return null;
}

async function listPerpMarketPLP(connection, admin, listingConfigPda, treasuryPda, symbol, baseTokenIndex, quoteTokenIndex, nonce) {
    const marketSymbol = `${symbol}-USDC`;
    console.log(`\nListing perp market: ${marketSymbol}...`);

    const config = PERP_CONFIGS[symbol];
    if (!config) {
        console.log(`  ⚠️ No config for ${symbol}`);
        return;
    }

    // Check if market already exists
    // Probe perp_market PDAs
    for (let i = 0; i < 100; i++) {
        const indexBuffer = Buffer.alloc(2);
        indexBuffer.writeUInt16LE(i);
        const [marketPda] = PublicKey.findProgramAddressSync(
            [PERP_MARKET_SEED, indexBuffer],
            LISTING_PROGRAM_ID
        );
        const info = await connection.getAccountInfo(marketPda);
        if (info) {
            // Parse symbol from market
            const symbolBytes = info.data.slice(11, 27);
            const existingSymbol = Buffer.from(symbolBytes).toString('utf8').replace(/\0/g, '');
            if (existingSymbol === marketSymbol) {
                console.log(`  ⏭️ Already listed at index ${i}`);
                return;
            }
        } else {
            break;
        }
    }

    // Step 1: ProposePerpMarket (instruction 30)
    console.log('  1. Submitting ProposePerpMarket...');
    
    // Derive PerpProposal PDA
    const nonceBuffer = Buffer.alloc(8);
    nonceBuffer.writeBigUInt64LE(nonce);
    
    const [proposalPda] = PublicKey.findProgramAddressSync(
        [PERP_PROPOSAL_SEED, admin.publicKey.toBuffer(), nonceBuffer],
        LISTING_PROGRAM_ID
    );

    // Check if proposal exists
    const proposalInfo = await connection.getAccountInfo(proposalPda);
    let proposalExists = proposalInfo !== null;

    if (!proposalExists) {
        // Derive TokenRegistry PDAs
        const baseIndexBuffer = Buffer.alloc(2);
        baseIndexBuffer.writeUInt16LE(baseTokenIndex);
        const [baseRegistryPda] = PublicKey.findProgramAddressSync(
            [TOKEN_REGISTRY_SEED, baseIndexBuffer],
            LISTING_PROGRAM_ID
        );

        const quoteIndexBuffer = Buffer.alloc(2);
        quoteIndexBuffer.writeUInt16LE(quoteTokenIndex);
        const [quoteRegistryPda] = PublicKey.findProgramAddressSync(
            [TOKEN_REGISTRY_SEED, quoteIndexBuffer],
            LISTING_PROGRAM_ID
        );

        // For testing on 1024Chain, use a deterministic placeholder oracle address
        // In production, this should be a real Pyth Oracle account
        // We'll derive a PDA from the market symbol to ensure consistency
        const [oracleAccount] = PublicKey.findProgramAddressSync(
            [Buffer.from('test_oracle'), Buffer.from(marketSymbol)],
            LISTING_PROGRAM_ID
        );
        
        // Build ProposePerpMarket instruction
        // Pass the oracle placeholder address as both the account and in instruction data
        const instructionData = buildProposePerpMarketData(nonce, marketSymbol, baseTokenIndex, quoteTokenIndex, config, symbol, oracleAccount);

        const proposeTx = new Transaction().add(
            new TransactionInstruction({
                programId: LISTING_PROGRAM_ID,
                keys: [
                    { pubkey: admin.publicKey, isSigner: true, isWritable: true },
                    { pubkey: proposalPda, isSigner: false, isWritable: true },
                    { pubkey: listingConfigPda, isSigner: false, isWritable: true },  // Writable for updating total_perp_proposals
                    { pubkey: baseRegistryPda, isSigner: false, isWritable: false },
                    { pubkey: quoteRegistryPda, isSigner: false, isWritable: false },
                    { pubkey: oracleAccount, isSigner: false, isWritable: false },
                    { pubkey: treasuryPda, isSigner: false, isWritable: true },
                    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
                ],
                data: instructionData,
            })
        );

        const proposeSig = await sendAndConfirmTransaction(connection, proposeTx, admin);
        if (!proposeSig) {
            console.log('  ❌ ProposePerpMarket failed');
            return;
        }
        console.log(`  ✅ ProposePerpMarket: ${proposeSig.substring(0, 16)}...`);
        proposalExists = true;
    } else {
        console.log('  ⏭️ Proposal already exists');
    }

    // Step 2: ApprovePerpMarket (Borsh index = 24)
    // 0-4: Admin (5 variants)
    // 5-12: Token (8 variants)
    // 13-21: Spot (9 variants)
    // 22: ProposePerpMarket
    // 23: ObjectPerpMarket
    // 24: ApprovePerpMarket
    console.log('  2. Approving PerpMarket...');
    
    // Get current total_perp_markets to derive the market PDA
    const configInfo = await connection.getAccountInfo(listingConfigPda);
    const configData = configInfo.data;
    // ListingConfig layout (from state.rs): 
    //   discriminator: u64 (8)
    //   version: u8 (1)
    //   admin: Pubkey (32)
    //   treasury: Pubkey (32)
    //   vault_program: Pubkey (32)
    //   fund_program: Pubkey (32)
    //   ledger_program: Pubkey (32)
    //   token_stake_amount: u64 (8)
    //   spot_stake_amount: u64 (8)
    //   perp_stake_amount: u64 (8)
    //   token_review_period_seconds: u32 (4)
    //   spot_review_period_seconds: u32 (4)
    //   perp_review_period_seconds: u32 (4)
    //   stake_lock_period_seconds: u32 (4)
    //   total_tokens: u16 (2)
    //   total_spot_markets: u16 (2)
    //   total_perp_markets: u16 (2)
    // Offset for total_perp_markets = 8+1+32+32+32+32+32+8+8+8+4+4+4+4+2+2 = 213
    const totalPerpMarkets = configData.readUInt16LE(213);
    
    // Derive PerpMarket PDA using current total_perp_markets (which will be the index)
    const [marketPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('perp_market'), Buffer.from(new Uint16Array([totalPerpMarkets]).buffer)],
        LISTING_PROGRAM_ID
    );
    
    // Build ApprovePerpMarket instruction (enum index 24, no data)
    const APPROVE_PERP_MARKET_INDEX = 24;
    const approveData = Buffer.from([APPROVE_PERP_MARKET_INDEX]);
    
    const approveTx = new Transaction().add(
        new TransactionInstruction({
            programId: LISTING_PROGRAM_ID,
            keys: [
                { pubkey: admin.publicKey, isSigner: true, isWritable: true },
                { pubkey: proposalPda, isSigner: false, isWritable: true },
                { pubkey: marketPda, isSigner: false, isWritable: true },
                { pubkey: listingConfigPda, isSigner: false, isWritable: true },
                { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
            ],
            data: approveData,
        })
    );
    
    const approveSig = await sendAndConfirmTransaction(connection, approveTx, admin);
    if (!approveSig) {
        console.log('  ❌ ApprovePerpMarket failed');
        return;
    }
    console.log(`  ✅ PerpMarket approved: ${approveSig.substring(0, 16)}... (market_index=${totalPerpMarkets})`);
}

function buildProposePerpMarketData(nonce, marketSymbol, baseTokenIndex, quoteTokenIndex, config, symbol, oraclePubkey) {
    // ProposePerpMarket - Enum variant index
    // After PLP-1 Token (8 variants: 5-12), PLP-2 Spot (9 variants: 13-21)
    // PLP-3 Perp starts at 22: ProposePerpMarket=22
    // Wait, let me count again from instruction.rs:
    // 0-4: Admin (5)
    // 5-12: Token (8)
    // 13-21: Spot (9)
    // 22-30: Perp (9)
    // So ProposePerpMarket = 22
    const PROPOSE_PERP_MARKET_INDEX = 22;
    
    const buffer = Buffer.alloc(200);
    let offset = 0;

    // Instruction index
    buffer.writeUInt8(PROPOSE_PERP_MARKET_INDEX, offset); offset += 1;

    // nonce: u64
    buffer.writeBigUInt64LE(nonce, offset); offset += 8;

    // symbol: [u8; 16]
    const symbolBuffer = Buffer.alloc(16);
    symbolBuffer.write(marketSymbol);
    symbolBuffer.copy(buffer, offset); offset += 16;

    // base_token_index: u16
    buffer.writeUInt16LE(baseTokenIndex, offset); offset += 2;

    // quote_token_index: u16
    buffer.writeUInt16LE(quoteTokenIndex, offset); offset += 2;

    // oracle: Pubkey - must match the oracle account passed to the instruction
    oraclePubkey.toBuffer().copy(buffer, offset); offset += 32;

    // tick_size_e6: u64
    buffer.writeBigUInt64LE(BigInt(config.tickSizeE6), offset); offset += 8;

    // lot_size_e6: u64
    buffer.writeBigUInt64LE(BigInt(config.lotSizeE6), offset); offset += 8;

    // max_leverage: u8
    buffer.writeUInt8(config.maxLeverage, offset); offset += 1;

    // initial_margin_rate_e6: u32 = 1_000_000 / max_leverage
    buffer.writeUInt32LE(Math.floor(1_000_000 / config.maxLeverage), offset); offset += 4;

    // maintenance_margin_rate_e6: u32 = 500_000 / max_leverage
    buffer.writeUInt32LE(Math.floor(500_000 / config.maxLeverage), offset); offset += 4;

    // taker_fee_bps: u16
    buffer.writeUInt16LE(5, offset); offset += 2;

    // maker_fee_bps: i16
    buffer.writeInt16LE(2, offset); offset += 2;

    // min_order_size_e6: u64
    buffer.writeBigUInt64LE(BigInt(1_000), offset); offset += 8;

    // max_order_size_e6: u64
    buffer.writeBigUInt64LE(BigInt(100_000_000), offset); offset += 8;

    // max_open_interest_e6: u64
    buffer.writeBigUInt64LE(BigInt(1_000_000_000_000), offset); offset += 8;

    // insurance_fund_deposit_e6: u64
    buffer.writeBigUInt64LE(BigInt(0), offset); offset += 8;

    return buffer.slice(0, offset);
}

function hexToBytes(hex) {
    const bytes = [];
    for (let i = 0; i < hex.length; i += 2) {
        bytes.push(parseInt(hex.substr(i, 2), 16));
    }
    return bytes;
}

// ============================================================================
// Transaction Helpers
// ============================================================================

async function sendAndConfirmTransaction(connection, tx, signer) {
    let lastError;

    for (let attempt = 1; attempt <= MAX_RETRIES; attempt++) {
        try {
            // Get fresh blockhash
            const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash('confirmed');
            tx.recentBlockhash = blockhash;
            tx.lastValidBlockHeight = lastValidBlockHeight;
            tx.feePayer = signer.publicKey;

            // Sign
            tx.sign(signer);

            // Send
            const rawTx = tx.serialize();
            const signature = await connection.sendRawTransaction(rawTx, {
                skipPreflight: false,  // Enable preflight to catch errors early
                maxRetries: 0,
            });

            console.log(`    Sent: ${signature.substring(0, 16)}... (attempt ${attempt})`);

            // Poll for confirmation
            const confirmed = await pollForConfirmation(connection, signature, lastValidBlockHeight);
            if (confirmed) {
                return signature;
            }

            lastError = new Error('Transaction not confirmed');
        } catch (error) {
            lastError = error;
            const errorMsg = error.message || error.toString();
            
            if (errorMsg.includes('block height exceeded') ||
                errorMsg.includes('expired') ||
                errorMsg.includes('Blockhash not found')) {
                console.log(`    ⚠️ Attempt ${attempt}/${MAX_RETRIES} - blockhash expired, retrying...`);
                continue;
            }
            
            // Check for specific program errors
            if (errorMsg.includes('already') || errorMsg.includes('initialized')) {
                console.log(`    ⏭️ Already exists, skipping...`);
                return null;
            }
            
            console.log(`    ❌ Error: ${errorMsg.substring(0, 100)}`);
            
            if (attempt < MAX_RETRIES) {
                await sleep(1000);
                continue;
            }
            throw error;
        }
    }

    console.log(`    ❌ Failed after ${MAX_RETRIES} attempts: ${lastError?.message}`);
    return null;
}

async function pollForConfirmation(connection, signature, lastValidBlockHeight) {
    const startTime = Date.now();

    while (Date.now() - startTime < CONFIRMATION_TIMEOUT_MS) {
        try {
            const currentBlockHeight = await connection.getBlockHeight('confirmed');
            if (currentBlockHeight > lastValidBlockHeight) {
                return false;
            }

            const status = await connection.getSignatureStatus(signature);
            if (status && status.value) {
                if (status.value.err) {
                    const errStr = JSON.stringify(status.value.err);
                    console.log(`    ❌ Tx failed: ${errStr}`);
                    throw new Error(`Transaction failed: ${errStr}`);
                }
                if (status.value.confirmationStatus === 'confirmed' ||
                    status.value.confirmationStatus === 'finalized') {
                    return true;
                }
            }

            await sleep(CONFIRMATION_POLL_INTERVAL_MS);
        } catch (error) {
            if (error.message.includes('Transaction failed')) {
                throw error;
            }
            await sleep(CONFIRMATION_POLL_INTERVAL_MS);
        }
    }

    return false;
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

main().catch(console.error);
