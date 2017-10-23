# Dmarket Blockchain

Install exomun framework:
```
cargo build
```

Start node service:
```
cargo run
```


Create Wallets:
```
curl -H "Content-Type: application/json" -X POST -d @create-wallet-1.json http://127.0.0.1:8000/api/services/cryptocurrency/v1/wallets/transaction
curl -H "Content-Type: application/json" -X POST -d @create-wallet-2.json http://127.0.0.1:8000/api/services/cryptocurrency/v1/wallets/transaction
```

Send 10 coin:
```
curl -H "Content-Type: application/json" -X POST -d @transfer-funds.json http://127.0.0.1:8000/api/services/cryptocurrency/v1/wallets/transaction
```

Block info:
```
curl http://127.0.0.1:8000/api/explorer/v1/blocks/6026
curl http://94.130.33.18:8000/api/explorer/v1/blocks?count=50&skip_empty_blocks=true&latest=44000  
```

Wallet info:
```
curl http://127.0.0.1:8000/api/services/cryptocurrency/v1/wallets
curl http://127.0.0.1:8000/api/services/cryptocurrency/v1/wallet/faa7c54f1b7450d7f42c2dd8ace24655430d7ea587712bc8e0ba7102d034464c
```

Transaction info: 
```
curl http://127.0.0.1:8000/api/system/v1/transactions/1a7ddbb85d07c9a95aaac65df5d90aa4f9a422bad0265a0ae691164ae7820b5a
```

Creator for Asset:
```
curl http://127.0.0.1:8000/api/services/cryptocurrency/v1/asset/
```
