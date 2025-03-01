use ethers::prelude::abigen;

abigen!(LotteryProvider, "abi/contracts/LotteryProvider.sol/LotteryProvider.json");
abigen!(House, "abi/contracts/House.sol/House.json");
abigen!(ERC20, "abi/ERC20.json");