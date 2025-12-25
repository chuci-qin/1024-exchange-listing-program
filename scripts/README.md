# Listing Program Scripts

## 部署流程

### 1. 编译程序

```bash
cd ..
cargo build-sbf
```

### 2. 部署到 1024Chain Testnet

```bash
chmod +x deploy.sh
./deploy.sh
```

### 3. 初始化 ListingConfig

更新 `init_listing_config.js` 中的 `LISTING_PROGRAM_ID`，然后：

```bash
npm install
node init_listing_config.js
```

## 配置

- **RPC**: https://testnet-rpc.1024chain.com/rpc/
- **部署者 Keypair**: ~/1024chain-testnet/keys/faucet.json

## 关联程序

| Program | ID |
|---------|-----|
| Vault | vR3BifKCa2TGKP2uhToxZAMYAYydqpesvKGX54gzFny |
| Ledger | Hf5vLwWoFK6e22wwYqT33YUCsxoTz3Jv2FEjrSa3GJPw |
| Fund | FPhDzu7yCDC1BBvzGwpM6dHHNQBPpKEv6Y3Ptdc7o3fJ |

