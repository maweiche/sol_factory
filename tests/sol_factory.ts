import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { IDL, SolFactory } from "../target/types/sol_factory";

import {
  PublicKey,
  Ed25519Program,
  SystemProgram,
  ComputeBudgetProgram,
  sendAndConfirmTransaction,
  Keypair,
  Transaction,
  Connection,
  GetProgramAccountsConfig,
  DataSizeFilter,
  MemcmpFilter,
  TransactionInstruction,
  VersionedTransaction,
  TransactionMessage,
  LAMPORTS_PER_SOL
} from "@solana/web3.js";

import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync } from "@solana/spl-token";

describe("sol_factory", () => {
  const wallet = anchor.Wallet.local();
  console.log('local wallet', wallet.publicKey.toBase58());

  const buyer_keypair = require('../test-wallet/keypair.json')
  const buyer = Keypair.fromSecretKey(Uint8Array.from(buyer_keypair))
  console.log('buyer', buyer.publicKey.toBase58());

  const collection_keypair = require('../test-wallet/keypair3.json')
  const collection_wallet = Keypair.fromSecretKey(Uint8Array.from(collection_keypair))
  console.log('collection_wallet', collection_wallet.publicKey.toBase58()); 


  const provider = anchor.getProvider();

  // const connection = new Connection("https://api.devnet.solana.com", "finalized"); // DEVNET
  const connection = new Connection("http://localhost:8899", "finalized"); // LOCALHOST
  
  const programId = new PublicKey("4Fj9kuGYLye3pwCBYaXbuzocEy22gPWT5TcJVJ6JauUt");

  const program = new anchor.Program<SolFactory>(IDL, programId, provider);
  const collectionRefKey = new PublicKey("mwUt7aCktvBeSm8bry6TvqEcNSUGtxByKCbBKfkxAzA");
  // Helpers
  function wait(ms: number) {
    return new Promise( resolve => setTimeout(resolve, ms) );
  }

  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block
    })
    return signature
  }

  const log = async(signature: string): Promise<string> => {
    console.log(`Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`);
    return signature;
  }

  const protocol = PublicKey.findProgramAddressSync([Buffer.from('protocol')], program.programId)[0];

  
  const collection = PublicKey.findProgramAddressSync([Buffer.from('collection'), collection_wallet.publicKey.toBuffer()], program.programId)[0];

  //  console.log('all_program_accounts', all_program_accounts)
  const id = Math.floor(Math.random() * 100000);
  const placeholder = PublicKey.findProgramAddressSync([Buffer.from('placeholder'), collection.toBuffer(), new anchor.BN(id).toBuffer("le", 8)], program.programId)[0];
  const placeholder_mint = PublicKey.findProgramAddressSync([Buffer.from('mint'), placeholder.toBuffer()], program.programId)[0];

  const nft = PublicKey.findProgramAddressSync([Buffer.from('ainft'), collection.toBuffer(), new anchor.BN(id).toBuffer("le", 8)], program.programId)[0];
  const nft_mint = PublicKey.findProgramAddressSync([Buffer.from('mint'), nft.toBuffer()], program.programId)[0];
  
  const auth = PublicKey.findProgramAddressSync([Buffer.from('auth')], program.programId)[0];
  const adminState = PublicKey.findProgramAddressSync([Buffer.from('admin_state'), wallet.publicKey.toBuffer()], program.programId)[0];
  
  
  
  const buyer_collection = PublicKey.findProgramAddressSync([Buffer.from('collection'), buyer.publicKey.toBuffer()], program.programId)[0];
  const buyer_placeholder = PublicKey.findProgramAddressSync([Buffer.from('placeholder'), buyer_collection.toBuffer(), new anchor.BN(id).toBuffer("le", 8)], program.programId)[0];
  const buyer_placeholder_mint = PublicKey.findProgramAddressSync([Buffer.from('mint'), buyer_placeholder.toBuffer()], program.programId)[0];
  const buyer_collection_nft = PublicKey.findProgramAddressSync([Buffer.from('ainft'), buyer_collection.toBuffer(), new anchor.BN(id).toBuffer("le", 8)], program.programId)[0];
  const buyer_collection_nft_mint = PublicKey.findProgramAddressSync([Buffer.from('mint'), buyer_collection_nft.toBuffer()], program.programId)[0];
  let buyerPlaceholderAta = getAssociatedTokenAddressSync(placeholder_mint, buyer.publicKey, false, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID)
  let buyerNftAta = getAssociatedTokenAddressSync(nft_mint, buyer.publicKey, false, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID)


  async function getCollectionUrl(collection: PublicKey) {
    const collection_data = await connection.getAccountInfo(collection);
    const collection_decode = program.coder.accounts.decode("collection", collection_data.data);
    // console.log('collection_decode', collection_decode)
    return collection_decode.url;
  };

  async function getAllCollections() {
    const size_filter: DataSizeFilter = {
      dataSize: 245
    };
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
    // console.log('all_collections', all_collections)

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

    // console.log('_collection_decode', _collection_decode)

    return all_collections;
  }

  // getAllCollections();
  
  // it("Initialize lock on Protocol", async () => {
  //   const protocol = PublicKey.findProgramAddressSync([Buffer.from('protocol')], program.programId)[0];

  //   const transaction = new Transaction().add(
  //     await program.methods
  //     .initializeProtocolAccount()
  //     .accounts({
  //       admin: wallet.publicKey,
  //       protocol: protocol,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .instruction()
  //   );
    
  //   await sendAndConfirmTransaction(connection, transaction, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);
  // });


  // it("Change the lock on the Protocol", async () => {
  //   const protocol = PublicKey.findProgramAddressSync([Buffer.from('protocol')], program.programId)[0];

  //   const transaction = new Transaction().add(
  //     await program.methods
  //     .lockProtocol()
  //     .accounts({
  //       admin: wallet.publicKey,
  //       protocol: protocol,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .instruction()
  //   );
    
  //   await sendAndConfirmTransaction(connection, transaction, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);
  // });

  // it ("Initialize a new Admin", async () => {
  //   const username = "MATTW";  // 5 characters MAX

  //   const createAdminIx = await program.methods
  //     .initializeAdminAccount(username)
  //     .accounts({
  //       admin: wallet.publicKey,
  //       adminState: null,
  //       newAdmin: wallet.publicKey,
  //       newAdminState: adminState,
  //       protocol: protocol,
  //       systemProgram: SystemProgram.programId, //TYPE: PublicKey
  //     })
  //     .instruction()

  //   const tx = new anchor.web3.Transaction().add(createAdminIx);
  //   await sendAndConfirmTransaction(connection, tx, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);
  // });

  // it("Create Collection", async () => {
  //   console.log('collection to string****', collection.toString())
  //   console.log('placeholder to string', placeholder.toString())
  //   console.log('placeholder mint to string', placeholder_mint.toString())
  //   console.log('nft to string', nft.toString())
  //   console.log('auth to string', auth.toString())
  //   console.log('adminState to string', adminState.toString())
  //   console.log('buyerPlaceholderAta to string', buyerPlaceholderAta.toString())
  //   const name = "Test 69 Collection";
  //   const symbol = "TT6969";
  //   const sale_start_time = new anchor.BN(Date.now() * 1000); // 1 second from now
  //   const max_supply = new anchor.BN(100);
  //   const price = new anchor.BN(10);
  //   const whitelist_price = new anchor.BN(0);
  //   const stable_id = "TST2333232131";
  //   const reference = "TST4571";
  //   const date_i64 = new anchor.BN(Date.now() * 1000); // 1 second from now
  //   const yesterday_date_i64 = new anchor.BN(Date.now() * 1000 - 86400000);
    
  //   const url = "https://amin.stable-dilution.art/nft/item/generation/3/11/0xf75e77b4EfD56476708792066753AC428eB0c21c";

    // create a function to generate 100 public keys and return them in an array
  //   async function generatePublicKeys() {
  //     let publicKeys = [];
  //     for (let i = 0; i < 5; i++) {
  //       const keypair = Keypair.generate();
  //       publicKeys.push(keypair.publicKey);
  //     }
  //     return publicKeys;
  //   }

  //   const publicKeys = await generatePublicKeys();
  //   try{

  //     const createCollectionIx = await program.methods
  //       .createCollection(
  //         collectionRefKey,
  //         name,
  //         symbol,
  //         url,
  //         date_i64,
  //         max_supply,
  //         price,
  //         stable_id,
  //       )
  //       .accounts({
  //         admin: wallet.publicKey,
  //         owner: collection_wallet.publicKey,
  //         collection: collection,
  //         adminState,
  //         protocol: protocol,
  //         systemProgram: SystemProgram.programId,
  //       })
  //       .instruction()

  //     const tx = new anchor.web3.Transaction().add(createCollectionIx);
  //     // tx.partialSign(collection_wallet);
  //     // console.log('tx partial sign', tx)
  //     await sendAndConfirmTransaction(connection, tx, [wallet.payer, collection_wallet], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);

  //     // getAllCollections();
  //   } catch (error) {
  //     console.log('error', error)
  //   }
  // });

  // it("Create Placeholder", async () => {
  //   const modifyComputeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 });
  //   console.log('FEE PAYER SOL BALANCE TO START: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));
  //   console.log('BUYER SOL BALANCE TO START: ', (await connection.getBalance(buyer.publicKey)) / LAMPORTS_PER_SOL);
  //   console.log('COLLECTION WALLET SOL BALANCE TO START: ', (await connection.getBalance(collection_wallet.publicKey)) / LAMPORTS_PER_SOL);
  //   const createListingIx = await program.methods
  //   .createPlaceholder(
  //     new anchor.BN(id),
  //     "https://gateway.irys.xyz/-mpn67FnEePrsoKez4f6Dvjb1aMcH1CqCdZX0NCyHK8",
  //   )
  //   .accounts({
  //     admin: wallet.publicKey,
  //     adminState,   
  //     collection: collection,
  //     placeholder: placeholder,
  //     mint: placeholder_mint,
  //     auth,
  //     rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //     token2022Program: TOKEN_2022_PROGRAM_ID,
  //     protocol: protocol,
  //     systemProgram: SystemProgram.programId,
  //   })
  //   .instruction()
  //   const instructions: TransactionInstruction[] = [
  //     modifyComputeUnitIx,
  //     createListingIx
  //   ];
  //   // const blockhash = await connection
  //   //       .getLatestBlockhash({ commitment: 'max' })
  //   //       .then((res) => res.blockhash);
  //   //     const messageV0 = new TransactionMessage({
  //   //       payerKey: wallet.publicKey,
  //   //       recentBlockhash: blockhash,
  //   //       instructions
  //   //     }).compileToV0Message();
        
  //   //     const txn = new VersionedTransaction(messageV0);
    
  //   //     txn.sign([wallet.payer]);
    
  //   //     const txId = await connection.sendTransaction(
  //   //       txn
  //   //     );

  //   //     console.log(`Transaction ID: ${txId}`);


  //   const tx = new anchor.web3.Transaction().add(
  //     createListingIx,
  //     modifyComputeUnitIx
  //   );

  //   await sendAndConfirmTransaction(connection, tx, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);
  // });

  // it("Buy Placeholder", async () => {
  //   console.log('FEE PAYER SOL BALANCE TO START: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));
  //   console.log('BUYER SOL BALANCE TO START: ', ((await connection.getBalance(buyer.publicKey)) / LAMPORTS_PER_SOL));
  //   console.log('COLLECTION WALLET SOL BALANCE TO START: ', ((await connection.getBalance(collection_wallet.publicKey)) / LAMPORTS_PER_SOL));
  //   const transaction = new Transaction().add(
  //     await program.methods
  //     .buyPlaceholder()
  //     .accounts({
  //       payer: wallet.publicKey,
  //       buyer: buyer.publicKey,
  //       collection: collection,
  //       collectionOwner: collection_wallet.publicKey,
  //       buyerMintAta: buyerPlaceholderAta,
  //       placeholder: placeholder,
  //       mint: placeholder_mint,
  //       auth,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       token2022Program: TOKEN_2022_PROGRAM_ID,
  //       protocol: protocol,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .instruction()
  //   );
    
  //   await sendAndConfirmTransaction(connection, transaction, [buyer, wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);
    

  //   console.log('FEE PAYER SOL BALANCE AFTER: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));
  //   console.log('BUYER SOL BALANCE AFTER: ', ((await connection.getBalance(buyer.publicKey)) / LAMPORTS_PER_SOL));
  //   console.log('COLLECTION WALLET SOL BALANCE AFTER: ', ((await connection.getBalance(collection_wallet.publicKey)) / LAMPORTS_PER_SOL));
  // });

  // it("Create Nft", async () => {
  //   // ADD IN THE FETCH OF THE URL FROM THE DECODED COLLECTION DATA
  //   const url = await getCollectionUrl(collection);
  //   console.log('FEE PAYER SOL BALANCE TO START: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));

  //   const modifyComputeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 });

  //   // url for fetch: https://amin.stable-dilution.art/nft/item/generation/3/11/0xf75e77b4EfD56476708792066753AC428eB0c21c
  //   // headers: "x-authorization: Bearer "
  //   // const url =  "https://amin.stable-dilution.art/nft/item/generation/3/11/0xf75e77b4EfD56476708792066753AC428eB0c21c"
  //   const nft_data = await fetch(url, {
  //     headers: {
  //       "x-authorization" : "Bearer "
  //     }
  //   })

  //   const metadata_json: any = await nft_data.json(); 

  //   const attributes = metadata_json.attributes.map((attr: any) => {
  //     return {key: attr.trait_type, value: attr.value}
  //   })

  //   const areweave_metadata: any = await fetch(metadata_json.metadataUrl)
  //   const areweave_json = await areweave_metadata.json()

  //   const nft_name = areweave_json.name;

  //   const createListingIx = await program.methods
  //   .createNft(
  //     new anchor.BN(id),
  //     metadata_json.metadataUrl,
  //     nft_name,
  //     attributes,
  //   )
  //   .accounts({
  //     admin: wallet.publicKey,
  //     adminState,   
  //     collection: collection,
  //     nft: nft,
  //     mint: nft_mint,
  //     auth,
  //     rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //     token2022Program: TOKEN_2022_PROGRAM_ID,
  //     protocol: protocol,
  //     systemProgram: SystemProgram.programId,
  //   })
  //   .instruction()

  //   const tx = new anchor.web3.Transaction().add(modifyComputeUnitIx).add(createListingIx);
  //   await sendAndConfirmTransaction(connection, tx, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);

  //   console.log('FEE PAYER SOL BALANCE AFTER: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));
  // });

  // it("Create Nft", async () => {
  //   // SHOULD FAIL BECAUSE 2 NFTS CANNOT HAVE THE SAME ID
  //   console.log('FEE PAYER SOL BALANCE TO START: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));

  //   // ADD IN THE FETCH OF THE URL FROM THE DECODED COLLECTION DATA
  //   const url = await getCollectionUrl(collection);
    
  //   const modifyComputeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 });

  //   // url for fetch: https://amin.stable-dilution.art/nft/item/generation/3/11/0xf75e77b4EfD56476708792066753AC428eB0c21c
  //   // headers: "x-authorization: Bearer "
  //   // const url =  "https://amin.stable-dilution.art/nft/item/generation/3/11/0xf75e77b4EfD56476708792066753AC428eB0c21c"
  //   const nft_data = await fetch(url, {
  //     headers: {
  //       "x-authorization" : "Bearer "
  //     }
  //   })

  //   const metadata_json: any = await nft_data.json(); 

  //   const attributes = metadata_json.attributes.map((attr: any) => {
  //     return {key: attr.trait_type, value: attr.value}
  //   })

  //   const areweave_metadata: any = await fetch(metadata_json.metadataUrl)
  //   const areweave_json = await areweave_metadata.json()

  //   const nft_name = areweave_json.name;

  //   const createListingIx = await program.methods
  //   .createNft(
  //     new anchor.BN(id),
  //     metadata_json.metadataUrl,
  //     nft_name,
  //     attributes,
  //   )
  //   .accounts({
  //     admin: wallet.publicKey,
  //     adminState,   
  //     collection: collection,
  //     nft: nft,
  //     mint: nft_mint,
  //     auth,
  //     rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //     token2022Program: TOKEN_2022_PROGRAM_ID,
  //     protocol: protocol,
  //     systemProgram: SystemProgram.programId,
  //   })
  //   .instruction()

  //   const tx = new anchor.web3.Transaction().add(modifyComputeUnitIx).add(createListingIx);
  //   await sendAndConfirmTransaction(connection, tx, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);

  //   console.log('FEE PAYER SOL BALANCE AFTER: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));
  // });

  // it("Transfer Nft and Burn Placeholder", async () => {
  //   console.log('FEE PAYER SOL BALANCE TO START: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));

  //   const transaction = new Transaction().add(
  //     await program.methods
  //     .transferNft()
  //     .accounts({
  //       payer: wallet.publicKey,
  //       buyer: buyer.publicKey,
  //       buyerMintAta: buyerNftAta,
  //       nft: nft,
  //       mint: nft_mint,
  //       collection: collection,
  //       auth,
  //       buyerPlaceholderMintAta: buyerPlaceholderAta,
  //       placeholder: placeholder,
  //       placeholderMint: placeholder_mint,
  //       placeholderMintAuthority: wallet.publicKey,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       token2022Program: TOKEN_2022_PROGRAM_ID,
  //       protocol: protocol,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .instruction()
  //   );
  //   transaction.feePayer = wallet.publicKey;
    
  //   await sendAndConfirmTransaction(connection, transaction, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);

  //   console.log('FEE PAYER SOL BALANCE AFTER: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));
  // });

  it("Should Complete everything in one txn", async () => {
    console.log('FEE PAYER SOL BALANCE TO START SINGLE TXN: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));
    console.log('BUYER SOL BALANCE TO START SINGLE TXN: ', ((await connection.getBalance(buyer.publicKey)) / LAMPORTS_PER_SOL));
    console.log('COLLECTION WALLET SOL BALANCE TO START SINGLE TXN: ', ((await connection.getBalance(collection_wallet.publicKey)) / LAMPORTS_PER_SOL));

    // airdrop placeholder to buyer
    const ed25519Ix = Ed25519Program.createInstructionWithPrivateKey({
      privateKey: collection_keypair.secretKey,
      message: Buffer.from("wallet address here"),
    });

    const modifyComputeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 600_000 });
    const createPlaceholderIx = await program.methods
      .createPlaceholder(
        new anchor.BN(id),
        "https://gateway.irys.xyz/-mpn67FnEePrsoKez4f6Dvjb1aMcH1CqCdZX0NCyHK8",
      )
      .accounts({
        admin: wallet.publicKey,
        adminState,   
        collection: collection,
        placeholder: placeholder,
        mint: placeholder_mint,
        auth,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        protocol: protocol,
        systemProgram: SystemProgram.programId,
      })  
      .instruction()
    
    const buyPlaceholderIx = await program.methods
      .buyPlaceholder()
      .accounts({
        payer: wallet.publicKey,
        buyer: buyer.publicKey,
        collection: collection,
        collectionOwner: collection_wallet.publicKey,
        buyerMintAta: buyerPlaceholderAta,
        placeholder: placeholder,
        mint: placeholder_mint,
        auth,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        protocol: protocol,
        systemProgram: SystemProgram.programId,
      })
      .instruction()
    
      // ADD IN THE FETCH OF THE URL FROM THE DECODED COLLECTION DATA
      const url = await getCollectionUrl(collection);

      // url for fetch: https://amin.stable-dilution.art/nft/item/generation/3/11/0xf75e77b4EfD56476708792066753AC428eB0c21c
      // headers: "x-authorization: Bearer "
      // const url =  "https://amin.stable-dilution.art/nft/item/generation/3/11/0xf75e77b4EfD56476708792066753AC428eB0c21c"
      const nft_data = await fetch(url, {
        headers: {
          "x-authorization" : "Bearer "
        }
      })

      const metadata_json: any = await nft_data.json(); 

      const attributes = metadata_json.attributes.map((attr: any) => {
        return {key: attr.trait_type, value: attr.value}
      })

      const areweave_metadata: any = await fetch(metadata_json.metadataUrl)
      const areweave_json = await areweave_metadata.json()

      const nft_name = areweave_json.name;

      const createNftIx = await program.methods
        .createNft(
          new anchor.BN(id),
          metadata_json.metadataUrl,
          nft_name,
          attributes,
        )
        .accounts({
          admin: wallet.publicKey,
          adminState,   
          collection: collection,
          nft: nft,
          mint: nft_mint,
          auth,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          protocol: protocol,
          systemProgram: SystemProgram.programId,
        })
        .instruction()

        const transferNftIx = await program.methods
          .transferNft()
          .accounts({
            payer: wallet.publicKey,
            buyer: buyer.publicKey,
            buyerMintAta: buyerNftAta,
            nft: nft,
            mint: nft_mint,
            collection: collection,
            auth,
            buyerPlaceholderMintAta: buyerPlaceholderAta,
            placeholder: placeholder,
            placeholderMint: placeholder_mint,
            placeholderMintAuthority: wallet.publicKey,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            token2022Program: TOKEN_2022_PROGRAM_ID,
            protocol: protocol,
            systemProgram: SystemProgram.programId,
          })
          .instruction()

        const instructions: TransactionInstruction[] = [
          modifyComputeUnitIx,
          createPlaceholderIx,
          buyPlaceholderIx,
          createNftIx,
          transferNftIx
        ];
        const blockhash = await connection
          .getLatestBlockhash({ commitment: 'max' })
          .then((res) => res.blockhash);
        const messageV0 = new TransactionMessage({
          payerKey: buyer.publicKey,
          recentBlockhash: blockhash,
          instructions
        }).compileToV0Message();
        
        const txn = new VersionedTransaction(messageV0);

        txn.sign([wallet.payer]);
        txn.sign([buyer]);
        const txId = await connection.sendTransaction(
          txn
        );

        console.log(`Transaction ID: ${txId}`);

        console.log('BUYER SOL BALANCE AFTER SINGLE TXN: ', ((await connection.getBalance(buyer.publicKey)) / LAMPORTS_PER_SOL));
  })  
});


