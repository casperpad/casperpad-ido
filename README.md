# Casper IDO

IDO platform smart contract for the Casper Network.
## Development

Make sure the `wasm32-unknown-unknown` Rust target is installed.

```
make prepare
```

## Build Smart Contracts
To build the example ERC-20 contract and supporting test contracts:

```
make build-contracts
```

## Test

```
make test
```

## Install IDO contract to Testnet

```
casper-client put-deploy -n http://95.217.34.115:7777/rpc \
--chain-name casper-test \
--secret-key /home/master/pitzerbert_secret_key.pem \
--session-path /home/master/workspace/swappery-ido/target/wasm32-unknown-unknown/release/casper_ido.wasm \
--payment-amount 150000000000
```

## Set purse to handle CSPR