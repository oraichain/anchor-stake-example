import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Fungstake } from "../target/types/fungstake";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { getAccount, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";

describe("fungstake", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);

  const payer = provider.wallet as anchor.Wallet;

  const connection = new Connection("https://api.devnet.solana.com", "confirmed")

  const mintKeyPayer = Keypair.fromSecretKey(new Uint8Array([
    244, 110, 133,  75,  52, 197, 240, 160, 158, 119,  56,
    222, 216,  74,  79, 135, 167,   7, 188, 168, 176,  55,
    173,  95,  57,  28,  27,  32, 240, 152,  43, 151,  72,
    156, 202,  53,  68, 163, 155, 178, 233, 202,  52, 108,
     80,  11,  10, 225, 219,  20,  78, 109, 173, 232,  79,
     36, 237,  47,  51, 161, 241, 202,  95,  25
  ]));

  console.log("mintKeyPayer", mintKeyPayer);

  

  const program = anchor.workspace.Fungstake as Program<Fungstake>;

  it("Is initialized!", async () => {
    // Add your test here.

    let [vaultAccount] = PublicKey.findProgramAddressSync([Buffer.from('vault')], program.programId)

  
    const tx = await program.methods
    .initialize()
    .accounts({
      signer: payer.publicKey,
      tokenVaultAccount: vaultAccount,
      mint: mintKeyPayer.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID
    })
    .rpc();
    console.log("Your transaction signature", tx);
  });


  it("It stake", async () => {


    let userTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mintKeyPayer.publicKey,
      payer.publicKey
    );

    console.log('bal',(await getAccount(connection,userTokenAccount.address)).amount)

    await mintTo(
      connection,
      payer.payer,
      mintKeyPayer.publicKey,
      userTokenAccount.address,
      payer.payer,
      200
    )

    let [stakeInfoAccount] = PublicKey.findProgramAddressSync([Buffer.from('stake_info'), payer.publicKey.toBuffer()], program.programId);

    let [stakeAccount] = PublicKey.findProgramAddressSync([Buffer.from('token'), payer.publicKey.toBuffer()], program.programId);

    await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mintKeyPayer.publicKey,
      payer.publicKey
    );

    // Add your test here.
    let [vaultAccount] = PublicKey.findProgramAddressSync([Buffer.from('vault')], program.programId);
  
    const tx = await program.methods
    .stake(new anchor.BN(4))
    .signers([payer.payer])
    .accounts({
      signer: payer.publicKey,
      tokenVaultAccount: vaultAccount,
      mint: mintKeyPayer.publicKey,
      stakeInfoAccount: stakeInfoAccount,
      userTokenAccount: userTokenAccount.address,
      stakeAccount: stakeAccount,
      tokenProgram: TOKEN_PROGRAM_ID
    })
    .rpc();
    console.log("Your transaction signature", tx);
  });


  it("It De-stake", async () => {


    let userTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mintKeyPayer.publicKey,
      payer.publicKey
    );

    console.log('bal',(await getAccount(connection,userTokenAccount.address)).amount)

    // await mintTo(
    //   connection,
    //   payer.payer,
    //   mintKeyPayer.publicKey,
    //   userTokenAccount.address,
    //   payer.payer,
    //   200
    // )

    let [stakeInfoAccount] = PublicKey.findProgramAddressSync([Buffer.from('stake_info'), payer.publicKey.toBuffer()], program.programId);

    let [stakeAccount] = PublicKey.findProgramAddressSync([Buffer.from('token'), payer.publicKey.toBuffer()], program.programId);

    // await getOrCreateAssociatedTokenAccount(
    //   connection,
    //   payer.payer,
    //   mintKeyPayer.publicKey,
    //   payer.publicKey
    // );

    // Add your test here.
    let [vaultAccount] = PublicKey.findProgramAddressSync([Buffer.from('vault')], program.programId);


     await mintTo(
      connection,
      payer.payer,
      mintKeyPayer.publicKey,
      vaultAccount,
      payer.payer,
      30000000000
    )
  
    const tx = await program.methods
    .destake()
    .signers([payer.payer])
    .accounts({
      signer: payer.publicKey,
      tokenVaultAccount: vaultAccount,
      mint: mintKeyPayer.publicKey,
      stakeInfoAccount: stakeInfoAccount,
      userTokenAccount: userTokenAccount.address,
      stakeAccount: stakeAccount,
      tokenProgram: TOKEN_PROGRAM_ID
    })
    .rpc();
    console.log("Your transaction signature", tx);
  });



});
