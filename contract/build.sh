#!/usr/bin/env bash

set -a

echo "build.sh: Building contracts"
cd ./raw
yarn install
yarn compile:types
npx --yes hardhat export-abi
cd -
rm -rf abi/valio && mkdir abi/valio
cp -rf raw/abi/contracts/* abi/valio
