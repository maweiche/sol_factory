# Sol Factory - Solana Program
A Solana program built for use in correlation with the [Sol Ai SDK](https://npmjs.com) and Generative AI Art platforms.

Below you can find instructions to running the program locally followed by a deeper dive into the program's structure and details.

> [!WARNING]  
> Sol Factory is still in development and is unstable, there can be minor and major changes at any time.

## Program ID
| Cluster      | Program Id |
| :---        |    :----:   |
| **Localnet**     | `6rHuJFF9XCxi9eDHtgJPcBKNpMWyBHhQhrFSkUD5XMYo` |
| **Devnet**  | `6rHuJFF9XCxi9eDHtgJPcBKNpMWyBHhQhrFSkUD5XMYo` |
| **Mainnet**  | ``  |

## Description

The Sol Factory program is structured for Artists to create on-chain `Collections` that a user can mint a Token-2022 `NFT` from on the Solana blockchain.

The Solana Factory program was built for use with the [Sol Ai SDK](https://npmjs.com).

You can view a baremetal combination of the Program and SDK live on Devnet [here]().

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/en/)
- [Anchor](https://www.anchor-lang.com/)
- [Solana CLI](https://docs.solanalabs.com/cli/install)
- [Solana Wallet](https://docs.solanalabs.com/cli/wallets/paper)

***For testing purposes it is advised to use at least three developer wallets (Admin, Artist, User)***

> If you don't have Anchor installed, you can use [Solana Playground](https://beta.solpg.io/) to build and deploy.


### Program Dependencies

This program is built with Anchor and requires a minimum version for certain Rust crates:

* **anchor-lang** = { version = "^0.29.0", features = ["init-if-needed"]}
* **aanchor-spl** = { version = "^0.29.0", features = ["token", "metadata"]}
* **solana-program** = "^1.17"

### Installing

To begin, clone the repo and install the necessary dependencies.

```
git clone https://github.com/maweiche/sol_factory.git
cd sol_factory
npm install
```

Inside of the `Anchor.toml` at the root level edit the cluster you want to test on and the following path to your local Wallet (**Admin Wallet**).

```rust
[provider]
cluster = "localnet"
wallet = "/Users/matt/.config/solana/id.json"
```

Next, set your local Wallet address to the `multisig_wallet` in `programs/sol_factory/src/constant.rs`. This will be the Wallet to perform **Admin** actions within the program.

```rust
pub mod multisig_wallet {
    use super::*;
    declare_id!("6KuX26FZqzqpsHDLfkXoBXbQRPEDEbstqNiPBKHNJQ9e");
}
```

### Building and Deploying program

To build and deploy on Solana's Localnet execute the following commands:

1. Make sure your Solana CLI cluster localnet by running: `solana config get`

```
matt@Matts-MBP sol_factory % solana config get                
Config File: /Users/matt/.config/solana/cli/config.yml
RPC URL: http://localhost:8899 
WebSocket URL: ws://localhost:8900/ (computed)
Keypair Path: /Users/matt/.config/solana/id.json 
Commitment: confirmed 
```

If you you need to set your Solana CLI cluster to localhost run: `solana config set --url localhost`

2. Start up `solana-test-validator` on a separate tab in your terminal, this will need to be operating while deploying and testing the program.

3. Next, build and deploy the program then grab the **Program Id** returned: 

```
anchor build
anchor deploy

Deploying cluster: http://localhost:8899
Upgrade authority: /Users/matt/.config/solana/id.json
Deploying program "sol_factory"...
Program path: /Users/matt/Desktop/AiSol/sol_factory/target/deploy/sol_factory.so...
Program Id: 4Fj9kuGYLye3pwCBYaXbuzocEy22gPWT5TcJVJ6JauUt

Deploy success
```
4. Change the **Program Id** in the following places:

```rust
/Anchor.toml

[programs.localnet]
sol_factory = "4Fj9kuGYLye3pwCBYaXbuzocEy22gPWT5TcJVJ6JauUt"
```

```rust
/programs/sol_factory/src/lib

declare_id!("4Fj9kuGYLye3pwCBYaXbuzocEy22gPWT5TcJVJ6JauUt");
```

5. Build and deploy the program one more time to register the deployed **Program Id** within the program itself.

```
anchor build
anchor deploy
```

### Testing program

The program comes with tests already designed to demo Admin, Artist, and User actions. To begin testing make sure you change the **Program Id** inside of the test file:

```rust
/tests/sol_factory.ts

const programId = new PublicKey("4Fj9kuGYLye3pwCBYaXbuzocEy22gPWT5TcJVJ6JauUt");
```

It is also recomended to create a directory at the root level and name it `test-wallet` and place two Keypairs to use in addition to your local Solana CLI wallet for testing.

🚨 **`test-wallet` is already included in the `.gitignore` if you name the folder something else then make sure to add the folder name to the `.gitignore`**

The current `Keypair` function utilizes a `Uint8array`, if using a Private Key exported from a Wallet like Phantom then change it to:

```ts
const buyer = Keypair.fromSecretKey(base58.decode(buyer_keypair))
```

To execute the tests, with `solana-test-validator` running, use the command:

```
anchor test
```

**🚨 Important Notes**
- The test file should be ran in complete totality just once. After the first successful execution the following tests can be skipped:
```ts
- "Initialize lock on Protocol"
- "Change the lock on the Protocol"
- "Create Collection"
```

### Deploy

This project is compatible with the generators from [create-solana-dapp](https://npm.im/create-solana-dapp).

You can use it to generate an Anchor application:

```shell
pnpm add -D @solana-developers/preset-anchor
pnpm nx generate @solana-developers/preset-anchor:application anchor --dry-run
```

With this base set up, you can now add Anchor programs and tests to your project.

```shell
pnpm nx generate @solana-developers/preset-anchor:template --projectName anchor --directory anchor --template counter counter --dry-run
```

## Program Structure and Details

The program takes on the following Structure:

```ssh 
Sol_Factory (owner: Admin)
|
|-> Collection (owner: Artist)
    |
    |-> Placeholder (owner: User, authority: Admin)
    |-> NFT (owner: User, authority: User)
```

The `Mint` flow looks like this under the hood:

```
1. User initiates Mint & pays Mint Price
2. User receives Placeholder NFT & AI Image Generation begins
3. AI Image Generation completes & new NFT minted then transferred to User while also burning the Placeholder
```

**🚨 Important Notes**
- The Placeholder Burn and the new NFT Transfer take place in the same txn, if the User does not have a Placeholder NFT for the respective Collection then the txn will not work.

We utilize the Placeholder NFT for 2 reasons:
- Provide the User something immediate after payment
- Provide an extra layer of security while minting a new NFT since it's burn is required

### Admin

The Admin, listed as the `multisig_wallet` in the `lib.rs`, has the ability to execute the `lock_protocol` command located in the `/src/context/protocol_setting.rs` setting. If the protocol is locked then all functions within the Sol Factory program will not work.

Upon first running `intialize_protocol_account` the program will lock and `change_locked_setting` will need to be called to unlock it. From there on, only `change_locked_setting` should be executed to toggle the locked setting.

The Admin will also be set as the Permanent Delegate on the Placeholder (Token 2022 NFT), thus allowing the Admin to Burn the Placeholder once the new NFT is transferred.

### Collection

A Collection is a Token 2022 NFT Collection where the details are set by the Artist and displayed to the User. The structure of a Sol Factory Collection is:

```rust
pub struct Collection {
    pub reference: Pubkey, // generic publickey all collections used to filter program's accounts
    pub name: String, // name of collection
    pub symbol: String, // collection symbol
    pub owner: Pubkey, // signer of createCollection transaction
    pub url: String, // url for AI Image Generation api call
    pub sale_start_time: i64, // start time for nft minting
    pub max_supply: u64, // max supply of collection
    pub total_supply: u64, // total number of nft's minted
    pub price: u64, // mint price
    pub stable_id: String, // stable diffusion id
}
```

To derive the `Publickey` of the `Collection` PDA you can use the following:

```rust
const collection = PublicKey.findProgramAddressSync([Buffer.from('collection'), owner.toBuffer()], program.programId)[0];
```

**🚨 Important Notes**
- The `owner` of the `Collection` is the `Signer` of the transaction when `createCollection` is called.
- The URL must be set to the AI Image Generation URL which must have the same Authorization as the other Collecitons in the Program


### Placeholder

The Placeholder is a temporary NFT that Sol Factory delivers to the User while the AI Image for their new NFT is being generated. The Placeholder has the same image and URI for each issuance, but the metadata will be reflective of the Artist's Collection they are minting from.

```
ex. placeholder mint: 
placeholder image:
placeholder uri:
```

The structure of a Sol Factory Placeholder is:

```rust
pub struct Placeholder {
    pub id: u64, // id used to generate new Placeholder
    pub collection: Pubkey, // NFT Collection 
    pub reference: String, //NFT Collection reference
    pub name: String, // Placeholder name
    pub price: u64, // Price paid on Mint
    pub time_stamp: i64, // time Placeholder was generated
}
```

To derive the `Publickey` of the `Placeholder` PDA along with it's `Mint` address you can use the following:

```ts
const placeholder = PublicKey.findProgramAddressSync([Buffer.from('placeholder'), collection.toBuffer(), new anchor.BN(id).toBuffer("le", 8)], program.programId)[0];
const placeholder_mint = PublicKey.findProgramAddressSync([Buffer.from('mint'), placeholder.toBuffer()], program.programId)[0];
```

**🚨 Important Notes**
- The same PDA seed structure is used to locate/burn the Placeholder, therefore a Placeholder for Collection A can not be used to mint from Collection B


### NFT

The Token 2022 NFT is referred to as a `AiNft` in the Sol Factory progarm. The structure for a AiNft is:

```rust
pub struct AiNft {
    pub id: u64, // id used to generate new NFT
    pub collection: Pubkey, // NFT Collection
    pub reference: String, // NFT Collection reference
    pub price: u64, // Price paid on Mint
    pub time_stamp: i64, // time NFT was generated
}
```

To derive the `Publickey` of the `AiNft` PDA along with it's `Mint` address you can use the following:

```ts
const nft = PublicKey.findProgramAddressSync([Buffer.from('ainft'), collection.toBuffer(), new anchor.BN(id).toBuffer("le", 8)], program.programId)[0];
const nft_mint = PublicKey.findProgramAddressSync([Buffer.from('mint'), nft.toBuffer()], program.programId)[0];
```


## Program Calls and Functions

The following examples for Program Calls and Functions are snippets taken from the `sol_factory.ts` file. Each program call will return instructions that can be included in a Solana transaction such as:

```ts
transaction.add(programIx)
```

For these examples `program`, instructions, and accounts were defined in the following manner:

```ts
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync } from "@solana/spl-token";
import * as anchor from "@coral-xyz/anchor";
import { IDL, SolFactory } from "../target/types/sol_factory"; // imports from files generated during anchor build/deply
const provider = anchor.getProvider();
const programId = new PublicKey("4Fj9kuGYLye3pwCBYaXbuzocEy22gPWT5TcJVJ6JauUt"); // your deployed Program Id

// Program
const program = new anchor.Program<SolFactory>(IDL, programId, provider);

// Instructions
const username = "WAGMI" // 5 characters MAX

const collectionReference: PublicKey = new PublicKey("mwUt7aCktvBeSm8bry6TvqEcNSUGtxByKCbBKfkxAzA")
const collectionName: string = "McNoot Dreams"
const symbol: string = "MND" // 4 characters MAX
const url: string = "https://stabledilution.xyz/123"
const saleStartTime: BigNumber = new anchor.BN(Date.now() * 1000); // 1 second from now
const maxSupply: BigNumber = new anchor.BN(100);
const price: BigNumber = new anchor.BN(10);
const stableId: string = "TST2333232131";

const id = new anchor.BN(Math.floor(Math.random() * 100000);)
const placeholderUri: string = "https://gateway.irys.xyz/-mpn67FnEePrsoKez4f6Dvjb1aMcH1CqCdZX0NCyHK8"

// Accounts
const admin: PublicKey = wallet.publicKey
const adminState = PublicKey.findProgramAddressSync([Buffer.from('admin_state'), wallet.publicKey.toBuffer()], program.programId)[0];

const collectionOwner: PublicKey = collection_wallet.publicKey
const collection: PublicKey = PublicKey.findProgramAddressSync([Buffer.from('collection'), collection_wallet.publicKey.toBuffer()], program.programId)[0];

const placeholder: PublicKey = PublicKey.findProgramAddressSync([Buffer.from('placeholder'), collection.toBuffer(), new anchor.BN(id).toBuffer("le", 8)], program.programId)[0];
const placeholderMint: PublicKey = PublicKey.findProgramAddressSync([Buffer.from('mint'), placeholder.toBuffer()], program.programId)[0];
const buyerPlaceholderMintAta: PublicKey = getAssociatedTokenAddressSync(placeholder_mint, buyer.publicKey, false, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);

const nft: PublicKey = PublicKey.findProgramAddressSync([Buffer.from('ainft'), collection.toBuffer(), new anchor.BN(id).toBuffer("le", 8)], program.programId)[0];
const nftMint: PublicKey = PublicKey.findProgramAddressSync([Buffer.from('mint'), nft.toBuffer()], program.programId)[0];
const buyerNftAta: PublicKey = getAssociatedTokenAddressSync(nft_mint, buyer.publicKey, false, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID)


const protocol: PublicKey = PublicKey.findProgramAddressSync([Buffer.from('protocol')], program.programId)[0];
const systemProgram = SystemProgram.programId
const rent = anchor.web3.SYSVAR_RENT_PUBKEY
const token2022Program = TOKEN_2022_PROGRAM_ID
```

### Admin

**Functions :** `initializeProtocolAccount()`, `lockProtocol()`

| Function     | Instructions | Accounts | Action |
| :---        |    :----:   |    :----:   |    :----:   |
| **`initializeProtocolAccount()`**     | n/a | admin, protocol, systemProgram | initializes the protocol account that can freeze all program actions |
| **`lockProtocol()`**  | n/a | admin, protocol, systemProgram | toggles the current locked state of the protocol account |
| **`initializeAdminAccount()`**  | username | admin, adminState, newAdmin, newAdminState, protocol, systemProgram | creates a new admin that has authority to sign collection and nft instructions |

### Collection

**Functions :** `createCollection()`

| Function     | Instructions | Accounts | Action |
| :---        |    :----:   |    :----:   |    :----:   |
| **`createCollection()`**     | reference, name, symbol, url, saleStartTime, maxSupply, price, stableId | admin, owner, collection, adminState, protocol, systemProgram | creates a NFT Collection users can mint from |

### Placeholder

**Functions :** `createPlaceholder()`, `buyPlaceholder()`

| Function     | Instructions | Accounts | Action |
| :---        |    :----:   |    :----:   |    :----:   |
| **`createPlaceholder()`**     | id, uri | admin, adminState, collection, placeholder, mint, auth, rent, token2022Program, protocol, systemProgram | creates a placeholder nft from collection |
| **`buyPlaceholder()`**  | n/a | payer, buyer, collection, collectionOwner, buyerPlaceholderMintAta, placeholder, placeholderMint, auth, associatedTokenProgram, tokenProgram, protocol, systemProgram | invokes sol transfer from signer and transfers placeholder nft to them |
| **`airdropPlaceholder()`**  | n/a | payer, buyer, collection, collectionOwner, buyerPlaceholderMintAta, placeholder, placeholderMint, auth, associatedTokenProgram, tokenProgram, protocol, systemProgram | only executable by collection owner, transfers placeholder to input wallet address bypassing the mint price and only paying admin fee |

### NFT

**Functions :** `createNft()`, `transferNft()`

| Function     | Instructions | Accounts | Action |
| :---        |    :----:   |    :----:   |    :----:   |
| **`createNft()`**     | id, url, name, attributes[] | admin, adminState, collection, nft, nftMint, auth, rent, token2022Program, protocol, systemProgram | creates a AI nft from collection |
| **`transferNft()`**  | n/a | payer, buyer, buyerNftMintAta, nft, nftMint, collection, auth, buyerPlaceholderMintAta, placeholder, placeholderMint, placeholderMintAuthority, associatedTokenProgram, tokenProgram, token2022Program, protocol, systemProgram | transfers AI nft to user while burning placeholder nft in same instruction  |

### Helpers

The following code snippets are from functions that pull specific data from the Sol Factory program:

```ts
// RETURNS ALL COLLECTION ACCOUNTS AND THEIR DECODED INFO
async function getAllCollections() {
    const memcmp_filter: MemcmpFilter = {
      memcmp: {
        offset: 8,
        bytes: collectionRefKey.toBase58()
      }
    };
    const get_accounts_config: GetProgramAccountsConfig = {
        commitment: "confirmed",
        filters: [memcmp_filter]
    };

    const all_collections = await connection.getProgramAccounts(
      programId, 
      get_accounts_config
    );

    const _collection_decode = all_collections.map((collection) => {
        try {
            const decode = program.coder.accounts.decode("collection", collection.account.data);
            console.log('decode', decode)
            return decode;
        } catch (error) {
            console.log('error', error)
            return null;
        }
    })

    return all_collections;
  }
```

```ts
// RETURNS AI IMAGE GENERATION URL FROM SPECIFIC COLLECTION
async function getCollectionUrl(collection: PublicKey) {
    const collection_data = await connection.getAccountInfo(collection);
    const collection_decode = program.coder.accounts.decode("collection", collection_data.data);
    return collection_decode.url;
};
```

## Help

Please reach out on Telegram or Twitter.

## Authors

Contributors names and contact info

[Matt Weichel](https://github.com/maweiche)

## Version History

* 0.2
    * Various bug fixes and optimizations
    * See [commit change]() or See [release history]()
* 0.1
    * Initial Release

## License

This project is licensed under the [NAME HERE] License - see the LICENSE.md file for details

## Acknowledgments

Inspiration, code snippets, etc.
* [awesome-readme](https://github.com/)






## TO DO
- AIRDROP AS COLLECTION OWNER FN