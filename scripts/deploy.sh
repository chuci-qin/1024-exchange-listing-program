#!/bin/bash

# 1024 Exchange Listing Program - Deployment Script
# Deploy to 1024Chain Testnet

set -e

# Configuration
RPC_URL="https://testnet-rpc.1024chain.com/rpc/"
KEYPAIR_PATH="$HOME/1024chain-testnet/keys/faucet.json"
PROGRAM_SO="../target/deploy/listing_program.so"
PROGRAM_KEYPAIR="../target/deploy/listing_program-keypair.json"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}=== 1024 Exchange Listing Program Deployment ===${NC}"
echo ""

# Check if program .so exists
if [ ! -f "$PROGRAM_SO" ]; then
    echo -e "${YELLOW}Building program...${NC}"
    cd ..
    cargo build-sbf
    cd scripts
fi

# Check program size
PROGRAM_SIZE=$(ls -l "$PROGRAM_SO" | awk '{print $5}')
echo -e "Program size: ${GREEN}$((PROGRAM_SIZE / 1024)) KB${NC}"

# Check deployer balance
echo ""
echo -e "${YELLOW}Checking deployer balance...${NC}"
BALANCE=$(solana balance --url "$RPC_URL" --keypair "$KEYPAIR_PATH" | awk '{print $1}')
echo -e "Deployer balance: ${GREEN}$BALANCE N1024${NC}"

# Deploy program
echo ""
echo -e "${YELLOW}Deploying program...${NC}"

solana program deploy \
    --url "$RPC_URL" \
    --keypair "$KEYPAIR_PATH" \
    --program-id "$PROGRAM_KEYPAIR" \
    "$PROGRAM_SO"

# Get program ID
PROGRAM_ID=$(solana-keygen pubkey "$PROGRAM_KEYPAIR")
echo ""
echo -e "${GREEN}=== Deployment Successful ===${NC}"
echo -e "Program ID: ${GREEN}$PROGRAM_ID${NC}"

# Verify deployment
echo ""
echo -e "${YELLOW}Verifying deployment...${NC}"
solana program show "$PROGRAM_ID" --url "$RPC_URL"

echo ""
echo -e "${GREEN}Done!${NC}"

