/**
 * 1024 Exchange Listing Program E2E Test Script
 * 
 * Tests the deployed Listing Program on 1024Chain Testnet
 * 
 * Usage: node test_listing_program.js
 */

const {
    Connection,
    Keypair,
    PublicKey,
    Transaction,
    SystemProgram,
    sendAndConfirmTransaction,
} = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");
const bs58 = require("bs58");

// Configuration
const RPC_URL = "https://testnet-rpc.1024chain.com/rpc/";
const WS_URL = "wss://testnet-rpc.1024chain.com/ws/";
const LISTING_PROGRAM_ID = new PublicKey("41QWGy3LpKcjrVVgXnCpFRa45wthZCk91sjmp8DccZzq");

// PDA Seeds
const LISTING_CONFIG_SEED = Buffer.from("listing_config");
const LISTING_TREASURY_SEED = Buffer.from("listing_treasury");
const TOKEN_REGISTRY_SEED = Buffer.from("token_registry");
const TOKEN_PROPOSAL_SEED = Buffer.from("token_proposal");
const SPOT_MARKET_SEED = Buffer.from("spot_market");
const PERP_MARKET_SEED = Buffer.from("perp_market");

// Test accounts
const TEST_ACCOUNTS = [
    {
        name: "Test Account #1",
        pubkey: "9ocm9zv5F2QghKaFSLGSjkVg6f8XZf54nVTjfC2M3dG4",
        privateKey: "65d7pAydmKwgo5mVBwnKQUS7BUP1ZBhisEbeRyfzFnGLez85AGSqcqbZCUbsccogzSyLBqYcoZVgU7x7AARtKMHz"
    },
    {
        name: "Test Account #2",
        pubkey: "G23icA8QJiAM2UwENf1112rGFxoqHP6JJa3TuwVseVxu",
        privateKey: "2Rc3q4XFhUeZE5LUQCCzuMDVy4iom7mevWgCFCeMobNWAymrNAGe8UEXKkfVJQHb4af4F81JJL86qQz16a1wnv4y"
    },
    {
        name: "Test Account #3",
        pubkey: "9S55H6Bbh2JCqdmQGcw2MWCdWeBNNQYb9GWiCHL62CUH",
        privateKey: "5isgvaK7oNcxNEctu6hRyYf7z1xEavfMRKmNGb6h9Ect2iFXtA9qKCFhWFhvxSzPJQBBMePuQ5Sd4VUYEKtd3oaq"
    }
];

class ListingProgramTester {
    constructor() {
        this.connection = new Connection(RPC_URL, {
            commitment: "confirmed",
            wsEndpoint: WS_URL
        });
        this.testResults = [];
    }

    async deriveListingConfigPda() {
        return PublicKey.findProgramAddressSync(
            [LISTING_CONFIG_SEED],
            LISTING_PROGRAM_ID
        );
    }

    async deriveTreasuryPda() {
        return PublicKey.findProgramAddressSync(
            [LISTING_TREASURY_SEED],
            LISTING_PROGRAM_ID
        );
    }

    async deriveTokenRegistryPda(tokenIndex) {
        const indexBuffer = Buffer.alloc(2);
        indexBuffer.writeUInt16LE(tokenIndex);
        return PublicKey.findProgramAddressSync(
            [TOKEN_REGISTRY_SEED, indexBuffer],
            LISTING_PROGRAM_ID
        );
    }

    async deriveSpotMarketPda(marketIndex) {
        const indexBuffer = Buffer.alloc(2);
        indexBuffer.writeUInt16LE(marketIndex);
        return PublicKey.findProgramAddressSync(
            [SPOT_MARKET_SEED, indexBuffer],
            LISTING_PROGRAM_ID
        );
    }

    async derivePerpMarketPda(marketIndex) {
        const indexBuffer = Buffer.alloc(2);
        indexBuffer.writeUInt16LE(marketIndex);
        return PublicKey.findProgramAddressSync(
            [PERP_MARKET_SEED, indexBuffer],
            LISTING_PROGRAM_ID
        );
    }

    async deriveTokenProposalPda(proposer, nonce) {
        const nonceBuffer = Buffer.alloc(8);
        nonceBuffer.writeBigUInt64LE(BigInt(nonce));
        return PublicKey.findProgramAddressSync(
            [TOKEN_PROPOSAL_SEED, proposer.toBuffer(), nonceBuffer],
            LISTING_PROGRAM_ID
        );
    }

    logTest(name, status, details = "") {
        const statusIcon = status ? "✅" : "❌";
        console.log(`${statusIcon} ${name}${details ? `: ${details}` : ""}`);
        this.testResults.push({ name, status, details });
    }

    async testConnection() {
        console.log("\n🔗 Testing Connection...\n");
        
        try {
            const version = await this.connection.getVersion();
            this.logTest("RPC Connection", true, `Version: ${version["solana-core"]}`);
            
            const slot = await this.connection.getSlot();
            this.logTest("Current Slot", true, slot.toString());
            
            return true;
        } catch (error) {
            this.logTest("RPC Connection", false, error.message);
            return false;
        }
    }

    async testProgramDeployment() {
        console.log("\n📦 Testing Program Deployment...\n");
        
        try {
            const accountInfo = await this.connection.getAccountInfo(LISTING_PROGRAM_ID);
            if (accountInfo) {
                this.logTest("Program Exists", true, `Size: ${accountInfo.data.length} bytes`);
                this.logTest("Program Executable", accountInfo.executable, accountInfo.executable ? "Yes" : "No");
                return true;
            } else {
                this.logTest("Program Exists", false, "Account not found");
                return false;
            }
        } catch (error) {
            this.logTest("Program Deployment", false, error.message);
            return false;
        }
    }

    async testPdaDerivation() {
        console.log("\n🔑 Testing PDA Derivation...\n");
        
        try {
            const [configPda, configBump] = await this.deriveListingConfigPda();
            this.logTest("ListingConfig PDA", true, `${configPda.toBase58()} (bump: ${configBump})`);
            
            const [treasuryPda, treasuryBump] = await this.deriveTreasuryPda();
            this.logTest("Treasury PDA", true, `${treasuryPda.toBase58()} (bump: ${treasuryBump})`);
            
            const [tokenPda, tokenBump] = await this.deriveTokenRegistryPda(0);
            this.logTest("TokenRegistry[0] PDA", true, `${tokenPda.toBase58()} (bump: ${tokenBump})`);
            
            const [spotPda, spotBump] = await this.deriveSpotMarketPda(0);
            this.logTest("SpotMarket[0] PDA", true, `${spotPda.toBase58()} (bump: ${spotBump})`);
            
            const [perpPda, perpBump] = await this.derivePerpMarketPda(0);
            this.logTest("PerpMarket[0] PDA", true, `${perpPda.toBase58()} (bump: ${perpBump})`);
            
            return true;
        } catch (error) {
            this.logTest("PDA Derivation", false, error.message);
            return false;
        }
    }

    async testListingConfig() {
        console.log("\n⚙️ Testing ListingConfig...\n");
        
        try {
            const [configPda] = await this.deriveListingConfigPda();
            const accountInfo = await this.connection.getAccountInfo(configPda);
            
            if (accountInfo) {
                this.logTest("ListingConfig Account Exists", true, `Size: ${accountInfo.data.length} bytes`);
                
                // Parse basic fields
                const data = accountInfo.data;
                const discriminator = data.readBigUInt64LE(0);
                const version = data.readUInt8(8);
                
                this.logTest("Discriminator", true, `0x${discriminator.toString(16)}`);
                this.logTest("Version", true, version.toString());
                
                // Read admin (pubkey at offset 9)
                const admin = new PublicKey(data.slice(9, 41));
                this.logTest("Admin", true, admin.toBase58());
                
                // Read treasury (pubkey at offset 41)
                const treasury = new PublicKey(data.slice(41, 73));
                this.logTest("Treasury", true, treasury.toBase58());
                
                // Read stake amounts (starting at offset after program IDs)
                // Offset: 9 (version) + 32*4 (4 pubkeys) = 137
                const offset = 9 + 32 * 4;
                const tokenStake = data.readBigUInt64LE(offset);
                const spotStake = data.readBigUInt64LE(offset + 8);
                const perpStake = data.readBigUInt64LE(offset + 16);
                
                this.logTest("Token Stake", true, `${Number(tokenStake) / 1e9} N1024`);
                this.logTest("Spot Stake", true, `${Number(spotStake) / 1e9} N1024`);
                this.logTest("Perp Stake", true, `${Number(perpStake) / 1e9} N1024`);
                
                return true;
            } else {
                this.logTest("ListingConfig Account", false, "Not initialized");
                return false;
            }
        } catch (error) {
            this.logTest("ListingConfig", false, error.message);
            return false;
        }
    }

    async testTreasuryBalance() {
        console.log("\n💰 Testing Treasury Balance...\n");
        
        try {
            const [treasuryPda] = await this.deriveTreasuryPda();
            const balance = await this.connection.getBalance(treasuryPda);
            
            this.logTest("Treasury Balance", true, `${balance / 1e9} N1024 (${balance} lamports)`);
            return true;
        } catch (error) {
            this.logTest("Treasury Balance", false, error.message);
            return false;
        }
    }

    async testAccountBalances() {
        console.log("\n👤 Testing Account Balances...\n");
        
        for (const account of TEST_ACCOUNTS) {
            try {
                const pubkey = new PublicKey(account.pubkey);
                const balance = await this.connection.getBalance(pubkey);
                this.logTest(account.name, true, `${balance / 1e9} N1024`);
            } catch (error) {
                this.logTest(account.name, false, error.message);
            }
        }
        return true;
    }

    printSummary() {
        console.log("\n" + "=".repeat(60));
        console.log("📊 Test Summary");
        console.log("=".repeat(60) + "\n");
        
        const passed = this.testResults.filter(t => t.status).length;
        const failed = this.testResults.filter(t => !t.status).length;
        
        console.log(`Total Tests: ${this.testResults.length}`);
        console.log(`✅ Passed: ${passed}`);
        console.log(`❌ Failed: ${failed}`);
        console.log(`Pass Rate: ${((passed / this.testResults.length) * 100).toFixed(1)}%`);
        
        if (failed > 0) {
            console.log("\n⚠️ Failed Tests:");
            this.testResults.filter(t => !t.status).forEach(t => {
                console.log(`  - ${t.name}: ${t.details}`);
            });
        }
        
        console.log("\n" + "=".repeat(60));
    }

    async run() {
        console.log("=".repeat(60));
        console.log("🧪 1024 Exchange Listing Program E2E Tests");
        console.log("=".repeat(60));
        console.log(`\nProgram ID: ${LISTING_PROGRAM_ID.toBase58()}`);
        console.log(`RPC URL: ${RPC_URL}`);
        console.log(`Timestamp: ${new Date().toISOString()}\n`);
        
        // Run tests
        await this.testConnection();
        await this.testProgramDeployment();
        await this.testPdaDerivation();
        await this.testListingConfig();
        await this.testTreasuryBalance();
        await this.testAccountBalances();
        
        // Print summary
        this.printSummary();
        
        return this.testResults.every(t => t.status);
    }
}

// Run tests
const tester = new ListingProgramTester();
tester.run().then(success => {
    process.exit(success ? 0 : 1);
}).catch(error => {
    console.error("Fatal error:", error);
    process.exit(1);
});

