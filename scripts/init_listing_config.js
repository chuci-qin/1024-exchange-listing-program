/**
 * 1024 Exchange Listing Program - Initialize ListingConfig
 * 
 * This script initializes the ListingConfig PDA account using native N1024 staking.
 * 
 * Note: 1024Chain is an Agave fork with 4x faster ticks (16 ticks/slot vs 64),
 * so blockhash expires ~4x faster. This script uses polling instead of WebSocket
 * for transaction confirmation.
 * 
 * Usage:
 *   node init_listing_config.js [--keypair <path>]
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

// Configuration
const RPC_URL = 'https://testnet-rpc.1024chain.com/rpc/';
const LISTING_PROGRAM_ID = new PublicKey('41QWGy3LpKcjrVVgXnCpFRa45wthZCk91sjmp8DccZzq');

// 1024Chain specific: faster block times mean shorter blockhash validity
// Default Solana: ~150 slots * 400ms = 60s
// 1024Chain: ~150 slots * 100ms = 15s (4x faster)
const MAX_RETRIES = 3;
const CONFIRMATION_POLL_INTERVAL_MS = 500;
const CONFIRMATION_TIMEOUT_MS = 30000; // 30 seconds

// Related Program IDs (from 当前配置信息.md)
const VAULT_PROGRAM_ID = new PublicKey('vR3BifKCa2TGKP2uhToxZAMYAYydqpesvKGX54gzFny');
const FUND_PROGRAM_ID = new PublicKey('FPhDzu7yCDC1BBvzGwpM6dHHNQBPpKEv6Y3Ptdc7o3fJ');
const LEDGER_PROGRAM_ID = new PublicKey('Hf5vLwWoFK6e22wwYqT33YUCsxoTz3Jv2FEjrSa3GJPw');

// PDA Seeds
const LISTING_CONFIG_SEED = Buffer.from('listing_config');
const LISTING_TREASURY_SEED = Buffer.from('listing_treasury');

// Instruction schema
class InitializeInstruction {
    constructor(fields) {
        this.vault_program = fields.vault_program;
        this.fund_program = fields.fund_program;
        this.ledger_program = fields.ledger_program;
    }
}

const initializeSchema = new Map([
    [InitializeInstruction, {
        kind: 'struct',
        fields: [
            ['vault_program', [32]],
            ['fund_program', [32]],
            ['ledger_program', [32]],
        ]
    }]
]);

async function main() {
    console.log('=== Initialize ListingConfig (Native N1024 Staking) ===\n');

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
    console.log('Balance:', balance / 1e9, 'N1024');

    // Derive ListingConfig PDA
    const [listingConfigPda, configBump] = await PublicKey.findProgramAddress(
        [LISTING_CONFIG_SEED],
        LISTING_PROGRAM_ID
    );
    console.log('\nListingConfig PDA:', listingConfigPda.toBase58());
    console.log('Config Bump:', configBump);

    // Derive Treasury PDA (for native N1024 staking)
    const [treasuryPda, treasuryBump] = await PublicKey.findProgramAddress(
        [LISTING_TREASURY_SEED],
        LISTING_PROGRAM_ID
    );
    console.log('Treasury PDA:', treasuryPda.toBase58());
    console.log('Treasury Bump:', treasuryBump);

    // Check if already initialized
    const accountInfo = await connection.getAccountInfo(listingConfigPda);
    if (accountInfo) {
        console.log('\n⚠️ ListingConfig already initialized!');
        console.log('Account size:', accountInfo.data.length, 'bytes');
        
        // Show treasury balance
        const treasuryBalance = await connection.getBalance(treasuryPda);
        console.log('Treasury Balance:', treasuryBalance / 1e9, 'N1024');
        return;
    }

    // Build instruction data
    // Instruction index 0 = Initialize
    const instructionIndex = Buffer.alloc(1);
    instructionIndex.writeUInt8(0, 0);

    const instruction = new InitializeInstruction({
        vault_program: VAULT_PROGRAM_ID.toBytes(),
        fund_program: FUND_PROGRAM_ID.toBytes(),
        ledger_program: LEDGER_PROGRAM_ID.toBytes(),
    });
    const instructionData = borsh.serialize(initializeSchema, instruction);

    // Combine: index + data
    const data = Buffer.concat([instructionIndex, Buffer.from(instructionData)]);

    // Build transaction
    // Accounts:
    // 0. [signer, writable] Admin (payer)
    // 1. [writable] ListingConfig PDA
    // 2. [writable] Treasury PDA (native N1024 staking)
    // 3. [] System Program
    const tx = new Transaction().add(
        new TransactionInstruction({
            programId: LISTING_PROGRAM_ID,
            keys: [
                { pubkey: admin.publicKey, isSigner: true, isWritable: true },
                { pubkey: listingConfigPda, isSigner: false, isWritable: true },
                { pubkey: treasuryPda, isSigner: false, isWritable: true },
                { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
            ],
            data,
        })
    );

    // Send transaction with retry logic for 1024Chain's faster blockhash expiry
    console.log('\nSending transaction...');
    
    let signature;
    let lastError;
    
    for (let attempt = 1; attempt <= MAX_RETRIES; attempt++) {
        try {
            console.log(`\nAttempt ${attempt}/${MAX_RETRIES}...`);
            
            // Get fresh blockhash right before sending (critical for 1024Chain)
            const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash('confirmed');
            tx.recentBlockhash = blockhash;
            tx.lastValidBlockHeight = lastValidBlockHeight;
            tx.feePayer = admin.publicKey;
            
            console.log('Blockhash:', blockhash);
            console.log('Valid until block:', lastValidBlockHeight);
            
            // Sign with fresh blockhash
            tx.sign(admin);
            
            // Send raw transaction for faster submission
            const rawTx = tx.serialize();
            signature = await connection.sendRawTransaction(rawTx, {
                skipPreflight: true,  // Skip preflight for speed
                maxRetries: 0,        // We handle retries ourselves
            });
            console.log('Signature:', signature);
            
            // Poll for confirmation (no WebSocket needed)
            const confirmed = await pollForConfirmation(connection, signature, lastValidBlockHeight);
            
            if (confirmed) {
                console.log('\n✅ ListingConfig initialized successfully!');
                
                // Display staking info
                console.log('\n📊 Staking Configuration:');
                console.log('  - Token Proposal: 1,000 N1024');
                console.log('  - Spot Proposal:  2,000 N1024');
                console.log('  - Perp Proposal:  5,000 N1024');
                console.log('\n💡 Note: Uses native N1024 (lamports) for staking, not SPL Token.');
                return;
            }
            
            lastError = new Error('Transaction not confirmed within timeout');
        } catch (error) {
            lastError = error;
            console.log(`Attempt ${attempt} failed:`, error.message);
            
            // If blockhash expired, retry with new blockhash
            if (error.message.includes('block height exceeded') || 
                error.message.includes('expired')) {
                console.log('Blockhash expired, will retry with fresh blockhash...');
                continue;
            }
            
            // For other errors, throw immediately
            throw error;
        }
    }
    
    throw new Error(`Failed after ${MAX_RETRIES} attempts: ${lastError?.message}`);
}

/**
 * Poll for transaction confirmation without WebSocket
 * 
 * @param {Connection} connection 
 * @param {string} signature 
 * @param {number} lastValidBlockHeight 
 * @returns {Promise<boolean>}
 */
async function pollForConfirmation(connection, signature, lastValidBlockHeight) {
    const startTime = Date.now();
    
    while (Date.now() - startTime < CONFIRMATION_TIMEOUT_MS) {
        try {
            // Check current block height
            const currentBlockHeight = await connection.getBlockHeight('confirmed');
            
            if (currentBlockHeight > lastValidBlockHeight) {
                console.log('Block height exceeded, transaction likely expired');
                return false;
            }
            
            // Check transaction status
            const status = await connection.getSignatureStatus(signature);
            
            if (status && status.value) {
                if (status.value.err) {
                    throw new Error(`Transaction failed: ${JSON.stringify(status.value.err)}`);
                }
                
                if (status.value.confirmationStatus === 'confirmed' || 
                    status.value.confirmationStatus === 'finalized') {
                    console.log(`Transaction confirmed at slot ${status.value.slot}`);
                    return true;
                }
            }
            
            // Wait before next poll
            await sleep(CONFIRMATION_POLL_INTERVAL_MS);
            
        } catch (error) {
            // Ignore RPC errors during polling, just retry
            console.log('Polling error (retrying):', error.message);
            await sleep(CONFIRMATION_POLL_INTERVAL_MS);
        }
    }
    
    console.log('Confirmation timeout reached');
    return false;
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

main().catch(console.error);
