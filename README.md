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
curl http://127.0.0.1:8000/api/services/cryptocurrency/wallets
curl http://127.0.0.1:8000/api/services/cryptocurrency/wallet/22c59b41d317c458732273fc8a6383f5acdda66e1ea00fcfc2cf36efe3ef9ad7
```

Transaction info: 
```
curl http://127.0.0.1:8000/api/system/v1/transactions/50929fa35ad167cad4a1b839405ab9d39f8f23a9513fad9fd27886c81a9c1bf4
```