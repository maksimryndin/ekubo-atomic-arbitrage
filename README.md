# Atomic arbitrage bot for Ekubo

## Articles:
* [How to create an atomic arbitrage bot in Starknet: part 1 (basics)](https://medium.com/@maksim.ryndin/how-to-create-an-atomic-arbitrage-bot-in-starknet-part-1-basics-418333ed9cd3)
* [How to create an atomic arbitrage bot in Starknet: part 2 (the foggy desert)](https://medium.com/@maksim.ryndin/how-to-create-an-atomic-arbitrage-bot-in-starknet-part-2-the-foggy-desert-d3f28fad69c7)
* [A low-risk arbitrage without an upfront capital: flash loans on Starknet](https://medium.com/@maksim.ryndin/a-low-risk-arbitrage-without-an-upfront-capital-flash-loans-on-starknet-c606fd077059)

## Prerequisites
* A Starknet account (see the article)
* Rust

## Run

1. `cp .env.sepolia.example .env`
2. provide account details - a private key and an account address
3. `cargo run -- simple`

Modes (arbitrage strategies):
* `simple` - this is a Rust port of https://github.com/EkuboProtocol/atomic-arbitrage-bot with more comments and some little improvements
* `ekubo-flash` - with Ekubo flash loan, see also https://github.com/maksimryndin/ekubo_flash_loan

## Development

If `EKUBO_API_REBUILD` env variable is set to any value, then the openapi Ekubo stubs are built. For that Docker is required.

## Troubleshooting

General advice: an error usually contains a backtrace with contract addresses - try to check the related address versus your account address, token contract, [Ekubo contracts](https://docs.ekubo.org/integration-guides/reference/contract-addresses) and verify an abi.

If you see the error `ContractNotFound` that it is highly probable that either you put the wrong address of token/router contract/account contract or your [account is not deployed](https://medium.com/@maksim.ryndin/how-to-create-an-atomic-arbitrage-bot-in-starknet-part-1-basics-418333ed9cd3).

`UPPERCASE_REASON` - check Ekubo smart contracts [error codes](https://docs.ekubo.org/integration-guides/reference/error-codes).

`CLEAR_AT_LEAST_MINIMUM` - every swap on Ekubo encompasses three phases: transfer some input amount, swap and withdraw the output. The order doesn’t matter because of the flash accounting. The error means that there is nothing to withdraw. Important: we cannot rely on the error instead of the profit estimation. Otherwise unprofitable strategies would just drain our balance at Ekubo Core contract.