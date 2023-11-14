# AAXXII sales

## Introduction

This is a contract that allow the sales of tokens. The logic consider that a `Sale` has three dates:

```
  open                close              release
    |-------------------|-------------------|

Where:
    open_date_timestamp < close_date_timestamp <= release_date_timestamp

IMPORTANT: timestamps are always in milliseconds!
```

The overall steps:

1. Sale creation
2. Users pays for tokens
3. Seller deposits the tokens to sold
4. Tokens to sold are released
5. Seller collects payment tokens

## 1. Sale creation

Only the owner of the contract should be able to create sales.

```rust
#[payable]
pub fn create_sale(
    &mut self,
    slug: String,
    is_in_near: bool,
    sold_token_contract_address: AccountId,
    // "one" references the payment token UNIT
    one_payment_token_purchase_rate: U128,
    max_available_sold_token: U128,
    open_date_timestamp: U64,
    close_date_timestamp: U64,
    release_date_timestamp: U64,
) -> u32
```

The **MOST IMPORTANT** thing to remember is that the `one_payment_token_purchase_rate` variable represent the amount of tokens that a user will receive for a unit of payment token (e.g. If payment token is NEAR then a unit is 1E24, and if payment token is USDC then a unit is 1E6).

## 2. Users pays for tokens

Depending on the sale, the buyers can pay with NEAR or any other NEP-141 token (e.g. USDC or USDT). **Only one payment token is available.** Remember that timestamps are in milliseconds.

### 2.1 Paying with `$NEAR`

Send the NEAR tokens as attached.

```rust
#[payable]
pub fn purchase_token_with_near(&mut self, sale_id: u32)
```

### 2.2 Paying with `$USDT`

Use the callback method of the NEP-141 `ft_transfer_call`.

## 3. Seller deposits the tokens to sold

Use the callback method of the NEP-141 `ft_transfer_call`.

## 4. Tokens to sold are released

User have 2 possibilities, and for both they should call the same method.

1. The Seller never deposited the full amount of sold tokens before the release timestamp, then all the buyers keep their deposits in the original token.
2. The Seller deposited correctly the amount of sold tokens, then all the buyers receive the amount of tokens they purchased.

```rust
pub fn withdraw_tokens(&mut self, sale_id: u32) -> Promise
```

3. If the Seller deposited more and have an excess of sold tokens in the contract, use:

```rust
pub fn withdraw_excess_sold_tokens(&mut self, sale_id: u32) -> Promise
```

4. And the last case is that the Seller deposited *some* but not all the sold tokens, then the sale fails and the seller can recover the tokens using the same `withdraw_excess_sold_tokens` method.

## 5. Seller collects payment tokens

The last step is for the seller to collect the payment tokens.

```rust
pub fn collect_tokens(&mut self, sale_id: u32) -> Promise
```