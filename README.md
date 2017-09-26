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
curl http://127.0.0.1:8000/api/services/cryptocurrency/wallet/68bb8b1b1451be91e53dc304b42e516ae04d3c2b8cb7c046fca8240beb7f598b
```

Transaction info: 
```
curl http://127.0.0.1:8000/api/system/v1/transactions/84fceb5d517648b1169483bbfb75ea09ffdfd9fafc4ff710804cf51a24821572
```