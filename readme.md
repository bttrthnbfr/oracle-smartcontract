ORACLE-SMARTCONTRACT
===================
This contract is used to collect NFT from several source and map based on `contract_id`, `owner`, and get `previous_owner` of specific token.


Building this contract
======================
Run the following, and we'll build our rust project up via cargo. This will generate our WASM binaries into our `res/` directory. This is the smart contract we'll be deploying onto the NEAR blockchain later.
```bash
./build.sh
```

Using this contract
===================
### Quickest deploy

You can build and deploy this smart contract to a development account. [Dev Accounts](https://docs.near.org/docs/concepts/account#dev-accounts)
```bash
chmod +x dev-deploy.sh
./dev-deploy
```