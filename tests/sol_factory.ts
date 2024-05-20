import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { IDL, SolFactory } from "../target/types/sol_factory";

import {
  PublicKey,
  SystemProgram,
  ComputeBudgetProgram,
  sendAndConfirmTransaction,
  Keypair,
  Transaction,
  Connection,
} from "@solana/web3.js";

import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync } from "@solana/spl-token";

describe("sol_factory", () => {
  const wallet = anchor.Wallet.local();
  const provider = anchor.getProvider();
  const connection = new Connection("http://127.0.0.1:8899", "finalized");
  const programId = new PublicKey("J41nXQ21D73QFAVpkD2FVVvAMEhRM37Z517NHhPzj6aV");

  const program = new anchor.Program<SolFactory>(IDL, programId, provider);

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

  const collection = PublicKey.findProgramAddressSync([Buffer.from('collection'), wallet.publicKey.toBuffer()], program.programId)[0];
    
  const id = Math.floor(Math.random() * 100000);
  const placeholder = PublicKey.findProgramAddressSync([Buffer.from('placeholder'), collection.toBuffer(), new anchor.BN(id).toBuffer("le", 8)], program.programId)[0];
  const placeholder_mint = PublicKey.findProgramAddressSync([Buffer.from('mint'), placeholder.toBuffer()], program.programId)[0];

  const nft = PublicKey.findProgramAddressSync([Buffer.from('ainft'), collection.toBuffer(), new anchor.BN(id).toBuffer("le", 8)], program.programId)[0];
  const nft_mint = PublicKey.findProgramAddressSync([Buffer.from('mint'), nft.toBuffer()], program.programId)[0];
  
  const auth = PublicKey.findProgramAddressSync([Buffer.from('auth')], program.programId)[0];
  const adminState = PublicKey.findProgramAddressSync([Buffer.from('admin_state'), wallet.publicKey.toBuffer()], program.programId)[0];

  const buyer = Keypair.generate();
  let buyerPlaceholderAta = getAssociatedTokenAddressSync(placeholder_mint, buyer.publicKey, false, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID)
  let buyerNftAta = getAssociatedTokenAddressSync(nft_mint, buyer.publicKey, false, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID)

  // it ("Initialize a new Admin", async () => {
  //   const username = "MW";

  //   const createAdminIx = await program.methods
  //   .initializeAdminAccount(username)
  //   .accounts({
  //     admin: wallet.publicKey,
  //     adminState: null,
  //     newAdmin: wallet.publicKey,
  //     newAdminState: adminState,
  //     systemProgram: SystemProgram.programId, //TYPE: PublicKey
  //   })
  //   .instruction()

  //   const tx = new anchor.web3.Transaction().add(createAdminIx);
  //   await sendAndConfirmTransaction(connection, tx, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);
  // });

  // it("Create Collection", async () => {
  //   const name = "Test Collection";
  //   const symbol = "TST";
  //   const sale_start_time = new anchor.BN(0);
  //   const max_supply = new anchor.BN(100);
  //   const price = new anchor.BN(100);
  //   const stable_id = "TST";
  //   const reference = "TST123";

  //   const createWatchIx = await program.methods
  //   .createCollection(
  //     name,
  //     symbol,
  //     sale_start_time,
  //     max_supply,
  //     price,
  //     stable_id,
  //     reference
  //   )
  //   .accounts({
  //     owner: wallet.publicKey,
  //     collection,
  //     systemProgram: SystemProgram.programId,
  //   })
  //   .instruction()

  //   const tx = new anchor.web3.Transaction().add(createWatchIx);
  //   await sendAndConfirmTransaction(connection, tx, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);
  // });

  it("Create Placeholder", async () => {
    const modifyComputeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 });
    
    const createListingIx = await program.methods
    .createPlaceholder(
      new anchor.BN(id),
      "https://www.example.com",
    )
    .accounts({
      admin: wallet.publicKey,
      adminState,   
      collection,
      placeholder,
      mint: placeholder_mint,
      auth,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      token2022Program: TOKEN_2022_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .instruction()

    const tx = new anchor.web3.Transaction().add(modifyComputeUnitIx).add(createListingIx);
    await sendAndConfirmTransaction(connection, tx, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);
  });

  it("Buy Placeholder", async () => {
    // airdrop 200SOL to buyer
    await connection.requestAirdrop(buyer.publicKey, 200 * 10 ** 9);
    await wait(1000);

    const transaction = new Transaction().add(
      await program.methods
      .buyPlaceholder()
      .accounts({
        payer: wallet.publicKey,
        buyer: buyer.publicKey,
        collection,
        buyerMintAta: buyerPlaceholderAta,
        placeholder,
        mint: placeholder_mint,
        auth,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .instruction()
    );
    transaction.feePayer = wallet.publicKey;
    
    await sendAndConfirmTransaction(connection, transaction, [buyer, wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);
  });

  // it("Create Nft", async () => {
  //   const modifyComputeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 });
  //   const nft_name = "Test NFT";

  //   const attributes = [
  //     {key: "season", value: "winter"},
  //     {key: "camera angle", value: "low angle"},
  //     {key: "theme", value: "nichijo"},
  //     {key: "seed", value: "3808958`1"},
  //     {key: "model", value: "gpt-3"},
  //   ];
     
  //   const createListingIx = await program.methods
  //   .createNft(
  //     new anchor.BN(id),
  //     nft_name,
  //     "https://www.example.com",
  //     attributes,
  //   )
  //   .accounts({
  //     admin: wallet.publicKey,
  //     adminState,   
  //     collection,
  //     nft,
  //     mint: nft_mint,
  //     auth,
  //     rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //     token2022Program: TOKEN_2022_PROGRAM_ID,
  //     systemProgram: SystemProgram.programId,
  //   })
  //   .instruction()

  //   const tx = new anchor.web3.Transaction().add(modifyComputeUnitIx).add(createListingIx);
  //   await sendAndConfirmTransaction(connection, tx, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);
  // });

  // it("Transfer Nft", async () => {
  //   // airdrop 2SOL to buyer
  //   await connection.requestAirdrop(buyer.publicKey, 2 * 10 ** 9);
  //   await wait(1000);

  //   const transaction = new Transaction().add(
  //     await program.methods
  //     .transferNft()
  //     .accounts({
  //       payer: wallet.publicKey,
  //       buyer: buyer.publicKey,
  //       buyerMintAta: buyerNftAta,
  //       nft,
  //       mint: nft_mint,
  //       auth,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       token2022Program: TOKEN_2022_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .instruction()
  //   );
  //   transaction.feePayer = wallet.publicKey;
    
  //   await sendAndConfirmTransaction(connection, transaction, [buyer, wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);
  // });

  it("Burn Placeholder", async () => {

    const transaction = new Transaction().add(
      await program.methods
      .burnPlaceholder()
      .accounts({
        buyer: buyer.publicKey,
        buyerMintAta: buyerPlaceholderAta,
        placeholder,
        placeholderMint: placeholder_mint,
        authority: auth,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        mint: placeholder_mint,
        mintAuthority: wallet.publicKey
      })
      .instruction()
    );
    
    await sendAndConfirmTransaction(connection, transaction, [wallet.payer], {commitment: "finalized", skipPreflight: true}).then(confirm).then(log);
  });
});


