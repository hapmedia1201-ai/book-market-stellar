# Textbook Exchange Hub

## Project Title
Textbook Exchange Hub

## Project Description
Textbook Exchange Hub is a decentralized smart contract platform built on Soroban (Stellar blockchain) that enables students to list, buy, and swap secondhand textbooks. It supports two listing modes — sale with token payment and peer-to-peer exchange — with on-chain records of every transaction. A 2% platform fee is collected automatically on each sale and held in the contract until the admin withdraws it.

## Project Vision
The vision of Textbook Exchange Hub is to reduce the financial burden on students by creating a trustless, transparent marketplace for secondhand textbooks. By removing intermediaries and recording all listings and transactions on-chain, it guarantees fair pricing, verifiable book conditions, and instant peer-to-peer token settlement — making quality education resources more accessible and affordable.

## Key Features
- **Dual Listing Mode:** Sellers choose between listing a book for sale (with a token price) or for swap.
- **Token Payments:** Buyers pay in SEP-41 compatible tokens; funds transfer directly to the seller on purchase.
- **Automatic Fee Collection:** A configurable platform fee (up to 10%) is deducted from each sale and held in the contract.
- **Price Updates:** Sellers can update the price of any active listing before it is sold.
- **Search & Filter:** Buyers can filter available books by grade, subject, listing type (sale or exchange), or browse all listings.
- **User Stats:** On-chain tracking of each user's listings, sales, purchases, swaps, and token totals.
- **Access Control:** Admin-only initialization and fee withdrawal; owner-only listing management.
- **Immutable Records:** All books and transactions recorded on-chain for full auditability.

## Usage Instructions
1. **Deploy & Initialize:** Deploy the contract, then call `initialize` with an admin address, a SEP-41 token contract address, and a fee percentage.
2. **List for Sale:** Sellers call `list_for_sale` with book details and a token price.
3. **List for Exchange:** Sellers call `list_for_exchange` to offer a book for swap at no token cost.
4. **Buy a Book:** Buyers approve the token amount, then call `buy_book`. Tokens transfer instantly to the seller minus the platform fee.
5. **Request a Swap:** Buyers call `request_exchange` on an exchange listing to initiate a swap.
6. **Manage Listings:** Sellers can call `update_price` or `unlist_book` at any time before a sale.
7. **Withdraw Fees:** Admin calls `withdraw_fees` to transfer accumulated fees to the admin wallet.
8. **Query:** Anyone can call search functions or `get_book` / `get_user` for transparent on-chain data.

## Future Scope
- **Escrow for Swaps:** Hold both parties' collateral in escrow until a swap is confirmed complete.
- **Condition Verification:** Allow buyers to dispute book condition with a time-locked resolution mechanism.
- **Rating System:** On-chain seller and buyer ratings after each completed transaction.
- **Multi-token Support:** Accept multiple SEP-41 tokens as payment currency.
- **Trial Listings:** Time-limited free listings for new users to encourage adoption.
- **Frontend Dashboard:** Full web interface for students to browse, list, and manage books with Freighter wallet integration.
- **Notification Alerts:** Off-chain indexer to notify users when a matching book is listed.

## Technology Stack
- Rust and Soroban SDK v22 for smart contract development.
- Stellar blockchain for decentralized, immutable state and token settlement.
- SEP-41 token standard for on-chain payments.
- @stellar/stellar-sdk and Freighter wallet for frontend integration.

## Contract Functions

| Function | Description |
|---|---|
| `initialize` | Set up contract with admin, token address, and platform fee |
| `list_for_exchange` | List a book for peer-to-peer swap |
| `list_for_sale` | List a book for sale with a token price |
| `buy_book` | Purchase a book; token transfer executed automatically |
| `request_exchange` | Request a swap on an exchange listing |
| `update_price` | Update the sale price of an active listing |
| `unlist_book` | Remove an active listing |
| `search_by_grade` | Filter available books by school grade (1–12) |
| `search_by_subject` | Filter available books by subject |
| `search_for_sale` | Browse all books currently listed for sale |
| `search_for_exchange` | Browse all books currently listed for swap |
| `get_book` | Get full details of a specific book by ID |
| `get_user` | Get stats and history for a specific user |
| `withdraw_fees` | Admin withdraws accumulated platform fees |

## Token Flow

```
Buyer approves token → buy_book()
    ├── (100% - fee%) → Seller (instant on-chain transfer)
    └── fee%          → Contract (held until admin withdraws)

Admin calls withdraw_fees() → full fee balance transfers to admin wallet
```

## How to Run

1. Clone:
   ```bash
   git clone https://github.com/yourname/textbook-exchange.git
   cd textbook-exchange
   ```

2. Build:
   ```bash
   cd contracts/textbook-exchange
   stellar contract build
   ```

3. Test:
   ```bash
   cargo test
   ```

4. Deploy to Testnet:
   ```bash
   stellar contract deploy \
     --wasm target/wasm32-unknown-unknown/release/textbook_exchange.wasm \
     --source-account student \
     --network testnet
   ```

5. Initialize:
   ```bash
   stellar contract invoke \
     --id <CONTRACT_ID> \
     -- initialize \
     --admin <ADMIN_ADDRESS> \
     --token_contract <TOKEN_ADDRESS> \
     --fee_percent 2
   ```

6. Frontend:
   ```bash
   cd frontend && npx serve .
   ```

## Contract Detail
- Network: Stellar Testnet
- **Contract ID**: `CBSCNRNTMAGQGYC6DXS7I6VCSIEEEMXRXFSNAGLOJEJE7HO4I4FRT2NF`
- **Transaction**: https://stellar.expert/explorer/testnet/tx/9e99462a6f167ed13b7b9b8f28da590d63b497fdc805059b00cf6408ead69a36

## Contribution
Contributions are welcome from blockchain developers and educators. Fork the repository and submit a pull request to help improve the platform.

## License
This project is licensed under the MIT License.

## Team
- [Your Name] | [@telegram] | [email] | [University + Year]
