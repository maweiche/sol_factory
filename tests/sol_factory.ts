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
  LAMPORTS_PER_SOL,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  AddressLookupTableProgram,
  GetProgramAccountsFilter,
} from "@solana/web3.js";

import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync } from "@solana/spl-token";

describe("sol_factory", () => {
  const wallet = anchor.Wallet.local();
  const _keypair = require('../test-wallet/keypair2.json')
  const _wallet = Keypair.fromSecretKey(Uint8Array.from(_keypair))
  console.log('local wallet', wallet.publicKey.toBase58());

  const buyer_keypair = require('../test-wallet/keypair.json')
  const buyer = Keypair.fromSecretKey(Uint8Array.from(buyer_keypair))
  console.log('buyer', buyer.publicKey.toBase58());

  const collection_keypair = require('../test-wallet/keypair3.json')
  const collection_wallet = Keypair.fromSecretKey(Uint8Array.from(collection_keypair))
  console.log('collection_wallet', collection_wallet.publicKey.toBase58()); 


  const provider = anchor.getProvider();

  const connection = new Connection("https://api.devnet.solana.com", "finalized"); // DEVNET
  // const connection = new Connection("http://localhost:8899", "finalized"); // LOCALHOST
  
  const programId = new PublicKey("8AETe8uj6pAeDBrNVWvwXFSywjdGjdVapeQADgoWqNH");

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

  // async function getPlaceholderIds(user: PublicKey) {
  //   // parse the user's wallet to get the placeholder nfts
  //   // the placeholder nfts will all have the same updateAuthority: '3wNUZtJwkQCUn8X5Uxx7DZrokYPRAegUrWYheCknQteK'
    
  //   const memcmp_filter: MemcmpFilter = {
  //     memcmp: {
  //       offset: 52,
  //       bytes: collectionRefKey.toBase58()
  //     }
  //   };
  //   const filters:GetProgramAccountsFilter[] = [
  //     {
  //       dataSize: 170,    //size of account (bytes)
  //     },
  //     {
  //       memcmp: {
  //         offset: 32,     //location of our query in the account (bytes)
  //         bytes: buyer.publicKey.toBase58(),  //our search criteria, a base58 encoded string
  //       }            
  //     }
  //  ];

  //   const get_accounts_config: GetProgramAccountsConfig = {
  //       commitment: "confirmed",
  //       filters: [...filters]
  //   };

  async function getCollectionUrl(collection: PublicKey) {
    const collection_data = await connection.getAccountInfo(collection);
    const collection_decode = program.coder.accounts.decode("collection", collection_data.data);
    // console.log('collection_decode', collection_decode)
    return {
      url: collection_decode.url,
      count: collection_decode.totalSupply
    }
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

    console.log('_collection_decode', _collection_decode)

    const sale_start_time = _collection_decode[0].saleStartTime;
    console.log('sale start time', sale_start_time)
    // convert it from bn to date
    const sale_start_date = sale_start_time.toNumber();
    console.log('sale start date', sale_start_date)
    console.log('current time', Date.now())
    return all_collections;
  }

  getAllCollections();

  

  async function createAndSendV0Tx(txInstructions: TransactionInstruction[]) {
    // Step 1 - Fetch Latest Blockhash
    let latestBlockhash = await connection.getLatestBlockhash('finalized');
    console.log("   ✅ - Fetched latest blockhash. Last valid height:", latestBlockhash.lastValidBlockHeight);

    // Step 2 - Generate Transaction Message
    const messageV0 = new TransactionMessage({
        payerKey: wallet.publicKey,
        recentBlockhash: latestBlockhash.blockhash,
        instructions: txInstructions
    }).compileToV0Message();
    console.log("   ✅ - Compiled transaction message");
    const transaction = new VersionedTransaction(messageV0);

    // Step 3 - Sign your transaction with the required `Signers`
    transaction.sign([wallet.payer]);
    console.log("   ✅ - Transaction Signed");

    // Step 4 - Send our v0 transaction to the cluster
    const txid = await connection.sendTransaction(transaction, { skipPreflight: true });
    console.log('Transaction ID:', txid)
    console.log("   ✅ - Transaction sent to network");

    // Step 5 - Confirm Transaction 
    const confirmation = await connection.confirmTransaction({
        signature: txid,
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight
    });
    if (confirmation.value.err) { throw new Error("   ❌ - Transaction not confirmed.") }
    console.log('🎉 Transaction succesfully confirmed!', '\n', `https://explorer.solana.com/tx/${txid}?cluster=devnet`);
  }



  // async function createLookupTable() {
  //   // Step 1 - Get a lookup table address and create lookup table instruction
  //   const [lookupTableInst, lookupTableAddress] =
  //       AddressLookupTableProgram.createLookupTable({
  //           authority: wallet.publicKey,
  //           payer: wallet.publicKey,
  //           recentSlot: await connection.getSlot(),
  //       });

  //   // Step 2 - Log Lookup Table Address
  //   console.log("Lookup Table Address:", lookupTableAddress.toBase58());

  //   // Step 3 - Generate a transaction and send it to the network
  //   const tx_id = await createAndSendV0Tx([lookupTableInst]);
  //   console.log("Transaction ID:", tx_id);
  //   return lookupTableAddress;
  // }

  
  // async function addAddressesToTable() {
    
  //   const LOOKUP_TABLE_ADDRESS = await createLookupTable();
    

  //   console.log("Lookup Table Address to extend:", LOOKUP_TABLE_ADDRESS.toBase58());
  //   // Step 1 - Create Transaction Instruction
  //   const addAddressesInstruction = AddressLookupTableProgram.extendLookupTable({
  //       payer: wallet.publicKey,
  //       authority: wallet.publicKey,
  //       lookupTable: LOOKUP_TABLE_ADDRESS,
  //       addresses: [
  //           collectionRefKey,
  //           protocol,
  //           auth,
  //           adminState,
  //       ],
  //   });
  //   // Step 2 - Generate a transaction and send it to the network
  //   await createAndSendV0Tx([addAddressesInstruction]);
  //   console.log(`Lookup Table Entries: `,`https://explorer.solana.com/address/${LOOKUP_TABLE_ADDRESS.toString()}/entries?cluster=devnet`) 

  //   const lookupTable = (await connection.getAddressLookupTable(LOOKUP_TABLE_ADDRESS)).value;

  //   console.log("Lookup Table Entries:", lookupTable);
  // }
  
  // addAddressesToTable();

  
  
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

  // it ("Initialize a second Admin", async () => {
  //   const username = "BAD";  // 5 characters MAX
  //   const newAdminState = PublicKey.findProgramAddressSync([Buffer.from('admin_state'), collection_wallet.publicKey.toBuffer()], program.programId)[0];
  //   const createAdminIx = await program.methods
  //     .initializeAdminAccount(username)
  //     .accounts({
  //       admin: wallet.publicKey,
  //       adminState: adminState,
  //       newAdmin: collection_wallet.publicKey,
  //       newAdminState: newAdminState,
  //       protocol: protocol,
  //       systemProgram: SystemProgram.programId, //TYPE: PublicKey
  //     })
  //     .instruction()

  //   const tx = new anchor.web3.Transaction().add(createAdminIx);
  //   await sendAndConfirmTransaction(connection, tx, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);
  // });

  // it ("Remove second Admin", async () => {
  //   const badAdmin = collection_wallet.publicKey;
  //   const badAdminState = PublicKey.findProgramAddressSync([Buffer.from('admin_state'), badAdmin.toBuffer()], program.programId)[0];
  //   const createAdminIx = await program.methods
  //     .removeAdminAccount()
  //     .accounts({
  //       admin: badAdmin,
  //       adminState: badAdminState,
  //       primaryAdmin: wallet.publicKey,
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
  //   const max_supply = new anchor.BN(100);
  //   const price = 1.5;
  //   const stable_id = "TST2333232131";
  //   const date_i64 = new anchor.BN(Math.floor(Date.now()/1000))
  //   const date_plus_10_days = new anchor.BN(Math.floor(Date.now()/1000) + 864000000);
    
  //   const url = "https://amin.stable-dilution.art/nft/item/generation/3";

  // // / fix 'https://amin.stable-dilution.art/nft/item/generation/3/'

  //   try{

  //     const createCollectionIx = await program.methods
  //       .createCollection(
  //         collectionRefKey,
  //         name,
  //         symbol,
  //         url,
  //         date_i64,
  //         date_plus_10_days,
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

  it("Create Placeholder", async () => {
    const modifyComputeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 });
    console.log('FEE PAYER SOL BALANCE TO START: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));
    console.log('BUYER SOL BALANCE TO START: ', (await connection.getBalance(buyer.publicKey)) / LAMPORTS_PER_SOL);
    console.log('COLLECTION WALLET SOL BALANCE TO START: ', (await connection.getBalance(collection_wallet.publicKey)) / LAMPORTS_PER_SOL);
    const createListingIx = await program.methods
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
    // const instructions: TransactionInstruction[] = [
    //   modifyComputeUnitIx,
    //   createListingIx
    // ];
    // const blockhash = await connection
    //       .getLatestBlockhash({ commitment: 'max' })
    //       .then((res) => res.blockhash);
    //     const messageV0 = new TransactionMessage({
    //       payerKey: wallet.publicKey,
    //       recentBlockhash: blockhash,
    //       instructions
    //     }).compileToV0Message();
        
    //     const txn = new VersionedTransaction(messageV0);
    
    //     txn.sign([wallet.payer]);
    
    //     const txId = await connection.sendTransaction(
    //       txn
    //     );

    //     console.log(`Transaction ID: ${txId}`);


    const tx = new anchor.web3.Transaction().add(
      createListingIx,
      modifyComputeUnitIx
    );

    await sendAndConfirmTransaction(connection, tx, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);
  });

  it("Buy Placeholder", async () => {
    console.log('FEE PAYER SOL BALANCE TO START: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));
    console.log('BUYER SOL BALANCE TO START: ', ((await connection.getBalance(buyer.publicKey)) / LAMPORTS_PER_SOL));
    console.log('COLLECTION WALLET SOL BALANCE TO START: ', ((await connection.getBalance(collection_wallet.publicKey)) / LAMPORTS_PER_SOL));
    console.log('buyerPlaceholderAta to string', buyerPlaceholderAta.toString())
    const transaction = new Transaction().add(
      await program.methods
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
    );
    
    await sendAndConfirmTransaction(connection, transaction, [buyer, wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);
    

    console.log('FEE PAYER SOL BALANCE AFTER: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));
    console.log('BUYER SOL BALANCE AFTER: ', ((await connection.getBalance(buyer.publicKey)) / LAMPORTS_PER_SOL));
    console.log('COLLECTION WALLET SOL BALANCE AFTER: ', ((await connection.getBalance(collection_wallet.publicKey)) / LAMPORTS_PER_SOL));
  });

  it("Create Nft", async () => {
    // ADD IN THE FETCH OF THE URL FROM THE DECODED COLLECTION DATA
    const { url, count } = await getCollectionUrl(collection);
    console.log('url', url)
    console.log('count', count)
    console.log('FEE PAYER SOL BALANCE TO START: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));

    const modifyComputeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 });

    // url for fetch: https://amin.stable-dilution.art/nft/item/generation/3/11/0xf75e77b4EfD56476708792066753AC428eB0c21c
    // headers: "x-authorization: Bearer "
    // const url =  "https://amin.stable-dilution.art/nft/item/generation/3/11/0xf75e77b4EfD56476708792066753AC428eB0c21c"
    console.log('url string: ', `${url}/${count}/${buyer.publicKey.toBase58()}`)
    const nft_data = await fetch(`${url}/${count}/${buyer.publicKey.toBase58()}`, {
      headers: {
        "x-authorization" : `Bearer ad4a356ddba9eff73cd627f69a481b8493ed975d7aac909eec4aaebdd9b506ef`
      }
    })
    console.log("nft_data", nft_data);
    const metadata_json: any = await nft_data.json(); 
    
    const attributes = metadata_json.attributes.map((attr: any) => {
      return {key: attr.trait_type, value: attr.value}
    })
    console.log('attributes', attributes)
    setTimeout(() => {
      console.log('metadata_json', metadata_json)
    }, 500)
    const areweave_metadata: any = await fetch(metadata_json.metadataUrl)
    console.log('areweave_metadata', areweave_metadata)
    const areweave_json = await areweave_metadata.json()

    const nft_name = areweave_json.name;
    console.log('nft_name', nft_name)
    const createListingIx = await program.methods
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

    const tx = new anchor.web3.Transaction().add(modifyComputeUnitIx).add(createListingIx);
    await sendAndConfirmTransaction(connection, tx, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);

    console.log('FEE PAYER SOL BALANCE AFTER: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));
  });

  it("Transfer Nft and Burn Placeholder", async () => {
    console.log('FEE PAYER SOL BALANCE TO START: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));

    const transaction = new Transaction().add(
      await program.methods
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
    );
    transaction.feePayer = wallet.publicKey;
    
    await sendAndConfirmTransaction(connection, transaction, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);

    console.log('FEE PAYER SOL BALANCE AFTER: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));
  });

});



  // it("Should Complete everything in one txn", async () => {
  //   console.log('FEE PAYER SOL BALANCE TO START SINGLE TXN: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));
  //   console.log('BUYER SOL BALANCE TO START SINGLE TXN: ', ((await connection.getBalance(buyer.publicKey)) / LAMPORTS_PER_SOL));
  //   console.log('COLLECTION WALLET SOL BALANCE TO START SINGLE TXN: ', ((await connection.getBalance(collection_wallet.publicKey)) / LAMPORTS_PER_SOL));

  //   // airdrop placeholder to buyer
    // const ed25519Ix = Ed25519Program.createInstructionWithPrivateKey({
    //   privateKey: collection_keypair.secretKey,
    //   message: Buffer.from("wallet address here"),
    // });

  //   const modifyComputeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 600_000 });
  //   const createPlaceholderIx = await program.methods
  //     .createPlaceholder(
  //       new anchor.BN(id),
  //       "https://gateway.irys.xyz/-mpn67FnEePrsoKez4f6Dvjb1aMcH1CqCdZX0NCyHK8",
  //     )
  //     .accounts({
  //       admin: wallet.publicKey,
  //       adminState,   
  //       collection: collection,
  //       placeholder: placeholder,
  //       mint: placeholder_mint,
  //       auth,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       token2022Program: TOKEN_2022_PROGRAM_ID,
  //       protocol: protocol,
  //       systemProgram: SystemProgram.programId,
  //     })  
  //     .instruction()
    
  //   const buyPlaceholderIx = await program.methods
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
    
  //     // ADD IN THE FETCH OF THE URL FROM THE DECODED COLLECTION DATA
  //     const url = await getCollectionUrl(collection);

  //     // url for fetch: https://amin.stable-dilution.art/nft/item/generation/3/11/0xf75e77b4EfD56476708792066753AC428eB0c21c
  //     // headers: "x-authorization: Bearer "
  //     // const url =  "https://amin.stable-dilution.art/nft/item/generation/3/11/0xf75e77b4EfD56476708792066753AC428eB0c21c"
  //     const nft_data = await fetch(url, {
  //       headers: {
  //         "x-authorization" : "Bearer "
  //       }
  //     })

  //     const metadata_json: any = await nft_data.json(); 
  //     console.log('metadata_json', metadata_json)
  //     const attributes = metadata_json.attributes.map((attr: any) => {
  //       return {key: attr.trait_type, value: attr.value}
  //     })

  //     const areweave_metadata: any = await fetch(metadata_json.metadataUrl)
  //     const areweave_json = await areweave_metadata.json()

  //     const nft_name = areweave_json.name;

  //     const createNftIx = await program.methods
  //       .createNft(
  //         new anchor.BN(id),
  //         metadata_json.metadataUrl,
  //         nft_name,
  //         attributes,
  //       )
  //       .accounts({
  //         admin: wallet.publicKey,
  //         adminState,   
  //         collection: collection,
  //         nft: nft,
  //         mint: nft_mint,
  //         auth,
  //         rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //         token2022Program: TOKEN_2022_PROGRAM_ID,
  //         protocol: protocol,
  //         systemProgram: SystemProgram.programId,
  //       })
  //       .instruction()

  //       const transferNftIx = await program.methods
  //         .transferNft()
  //         .accounts({
  //           payer: wallet.publicKey,
  //           buyer: buyer.publicKey,
  //           buyerMintAta: buyerNftAta,
  //           nft: nft,
  //           mint: nft_mint,
  //           collection: collection,
  //           auth,
  //           buyerPlaceholderMintAta: buyerPlaceholderAta,
  //           placeholder: placeholder,
  //           placeholderMint: placeholder_mint,
  //           placeholderMintAuthority: wallet.publicKey,
  //           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //           tokenProgram: TOKEN_PROGRAM_ID,
  //           token2022Program: TOKEN_2022_PROGRAM_ID,
  //           protocol: protocol,
  //           systemProgram: SystemProgram.programId,
  //         })
  //         .instruction()

  //       const instructions: TransactionInstruction[] = [
  //         modifyComputeUnitIx,
  //         createPlaceholderIx,
  //         buyPlaceholderIx,
  //         createNftIx,
  //         transferNftIx
  //       ];
  //       const blockhash = await connection
  //         .getLatestBlockhash({ commitment: 'max' })
  //         .then((res) => res.blockhash);
  //       const messageV0 = new TransactionMessage({
  //         payerKey: buyer.publicKey,
  //         recentBlockhash: blockhash,
  //         instructions
  //       }).compileToV0Message();
        
  //       const txn = new VersionedTransaction(messageV0);

  //       txn.sign([wallet.payer]);
  //       txn.sign([buyer]);
  //       const txId = await connection.sendTransaction(
  //         txn
  //       );

  //       const size_of_txn = txn.serialize().length;
  //       console.log('size of non airdrop txn', size_of_txn)
  //       console.log(`Transaction ID: ${txId}`);

  //       console.log('BUYER SOL BALANCE AFTER SINGLE TXN: ', ((await connection.getBalance(buyer.publicKey)) / LAMPORTS_PER_SOL));
  // })  

//   it("Should Complete everything in one txn as an Airdrop", async () => {
//     const LOOKUP_TABLE_ADDRESS = new PublicKey('HHDVCeTGwWHMqWNBqCxzWCPqfjcV7Gmji7bZsVjGpNjA');
//     const lookupTable = (await connection.getAddressLookupTable(LOOKUP_TABLE_ADDRESS)).value;
//     console.log('FEE PAYER SOL BALANCE TO START SINGLE TXN: ', ((await connection.getBalance(wallet.publicKey)) / LAMPORTS_PER_SOL));
//     console.log('BUYER SOL BALANCE TO START SINGLE TXN: ', ((await connection.getBalance(buyer.publicKey)) / LAMPORTS_PER_SOL));
//     console.log('COLLECTION WALLET SOL BALANCE TO START SINGLE TXN: ', ((await connection.getBalance(collection_wallet.publicKey)) / LAMPORTS_PER_SOL));

//     // airdrop placeholder to buyer
//     const ed25519Ix = Ed25519Program.createInstructionWithPrivateKey({
//       privateKey: collection_wallet.secretKey,
//       message: buyer.publicKey.toBuffer(),
//     });
//     // console.log('ed25519Ix', ed25519Ix)
//     const modifyComputeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 600_000 });
//     const createPlaceholderIx = await program.methods
//       .createPlaceholder(
//         new anchor.BN(id),
//         "https://gateway.irys.xyz/-mpn67FnEePrsoKez4f6Dvjb1aMcH1CqCdZX0NCyHK8",
//       )
//       .accounts({
//         admin: wallet.publicKey,
//         adminState,   
//         collection: collection,
//         placeholder: placeholder,
//         mint: placeholder_mint,
//         auth,
//         rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//         token2022Program: TOKEN_2022_PROGRAM_ID,
//         protocol: protocol,
//         systemProgram: SystemProgram.programId,
//       })  
//       .instruction()
    
//     const buyPlaceholderIx = await program.methods
//       .airdropPlaceholder()
//       .accounts({
//         payer: wallet.publicKey,
//         buyer: buyer.publicKey,
//         collection: collection,
//         collectionOwner: collection_wallet.publicKey,
//         buyerMintAta: buyerPlaceholderAta,
//         placeholder: placeholder,
//         mint: placeholder_mint,
//         auth: auth, //lookupTable.state.addresses[2],
//         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         token2022Program: TOKEN_2022_PROGRAM_ID,
//         protocol: protocol, //lookupTable.state.addresses[1],
//         systemProgram: SystemProgram.programId,
//         instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
//       })
//       .instruction()
    
//       // ADD IN THE FETCH OF THE URL FROM THE DECODED COLLECTION DATA
//       const url = await getCollectionUrl(collection);

//       // url for fetch: https://amin.stable-dilution.art/nft/item/generation/3/11/0xf75e77b4EfD56476708792066753AC428eB0c21c
//       // headers: "x-authorization: Bearer "
//       // const url =  "https://amin.stable-dilution.art/nft/item/generation/3/11/0xf75e77b4EfD56476708792066753AC428eB0c21c"
//       const nft_data = await fetch(url, {
//         headers: {
//           "x-authorization" : "Bearer "
//         }
//       })

//       const metadata_json: any = await nft_data.json(); 

//       const attributes = metadata_json.attributes.map((attr: any) => {
//         return {key: attr.trait_type, value: attr.value}
//       })

//       const areweave_metadata: any = await fetch(metadata_json.metadataUrl)
//       const areweave_json = await areweave_metadata.json()

//       const nft_name = areweave_json.name;

//       const createNftIx = await program.methods
//         .createNft(
//           new anchor.BN(id),
//           metadata_json.metadataUrl,
//           nft_name,
//           attributes,
//         )
//         .accounts({
//           admin: wallet.publicKey,
//           adminState: adminState, //lookupTable.state.addresses[3],   
//           collection: collection,
//           nft: nft,
//           mint: nft_mint,
//           auth: auth, //lookupTable.state.addresses[2],
//           rent: anchor.web3.SYSVAR_RENT_PUBKEY,
//           token2022Program: TOKEN_2022_PROGRAM_ID,
//           protocol: protocol, //lookupTable.state.addresses[1],
//           systemProgram: SystemProgram.programId,
//         })
//         .instruction()

//         const transferNftIx = await program.methods
//           .transferNft()
//           .accounts({
//             payer: wallet.publicKey,
//             buyer: buyer.publicKey,
//             buyerMintAta: buyerNftAta,
//             nft: nft,
//             mint: nft_mint,
//             collection: collection,
//             auth: auth, //lookupTable.state.addresses[2],
//             buyerPlaceholderMintAta: buyerPlaceholderAta,
//             placeholder: placeholder,
//             placeholderMint: placeholder_mint,
//             placeholderMintAuthority: wallet.publicKey,
//             associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//             tokenProgram: TOKEN_PROGRAM_ID,
//             token2022Program: TOKEN_2022_PROGRAM_ID,
//             protocol: protocol, //lookupTable.state.addresses[1],
//             systemProgram: SystemProgram.programId,
//           })
//           .instruction()

//           const instructions: TransactionInstruction[] = [
//             modifyComputeUnitIx,
//             createPlaceholderIx,
//             ed25519Ix,
//             buyPlaceholderIx,
//           ];
//         const blockhash = await connection
//           .getLatestBlockhash({ commitment: 'max' })
//           .then((res) => res.blockhash);
//         const messageV0 = new TransactionMessage({
//           payerKey: wallet.publicKey,
//           recentBlockhash: blockhash,
//           instructions
//         }).compileToV0Message();
//         const txn = new VersionedTransaction(messageV0);

//         const size_of_txn = txn.serialize().length;
//         console.log('size_of_txn', size_of_txn)


//         txn.sign([wallet.payer]);
//         txn.sign([collection_wallet]);

//         const txId = await connection.sendTransaction(
//           txn
//         );

//         console.log(`Transaction ID: ${txId}`);

//         console.log('BUYER SOL BALANCE AFTER SINGLE TXN: ', ((await connection.getBalance(buyer.publicKey)) / LAMPORTS_PER_SOL));
//       const instructions2: TransactionInstruction[] = [
//         createNftIx,
//         transferNftIx
//       ];

//       const blockhash2 = await connection
//           .getLatestBlockhash({ commitment: 'max' })
//           .then((res) => res.blockhash);
//         const messageV02 = new TransactionMessage({
//           payerKey: buyer.publicKey,
//           recentBlockhash: blockhash2,
//           instructions: instructions2
//         }).compileToV0Message([lookupTable]);
//         // console.log('messageV0', messageV02)

//         const txn2 = new VersionedTransaction(messageV02);


//         const size_of_txn2 = txn2.serialize().length;
//         console.log('size_of_txn2', size_of_txn2)


//         txn2.sign([wallet.payer]);
//         txn2.sign([buyer]);

//         const txId2 = await connection.sendTransaction(
//           txn2,
//           {
//             skipPreflight: true
//           }
//         );

//         console.log(`Transaction ID 2: ${txId2}`);

//   })  
