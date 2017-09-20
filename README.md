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
curl http://127.0.0.1:8000/api/explorer/v1/blocks/1
curl http://127.0.0.1:8000/api/explorer/v1/blocks?count=50&skip_empty_blocks=true&latest=44000  
```

Wallet info:
```
curl http://127.0.0.1:8000/api/services/cryptocurrency/wallet/cdfe0378c3b7614410c468b7179cd5ba2b4ff3b9e5e24965b1aa23c5f623d28c
```

Transaction info: 
```
curl http://127.0.0.1:8000/api/system/v1/transactions/cc07a264f4635c144167183a8eadcb5b8deeccb87e5387aa0a744ff3ed484865
```