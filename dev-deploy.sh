#!/bin/bash
./build.sh
rm -rf neardev
near dev-deploy --wasmFile res/oracle_smartcontract.wasm --helperUrl https://near-contract-helper.onrender.com