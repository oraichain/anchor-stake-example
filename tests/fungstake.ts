import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { Fungstake } from "../target/types/fungstake";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  TransactionConfirmationStrategy,
} from "@solana/web3.js";
import {
  createMint,
  getAccount,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

describe("fungstake", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const payer = provider.wallet as anchor.Wallet;

  let stakeCurrencyMint: PublicKey;
  let rewardCurrencyMint: PublicKey;

  const program = anchor.workspace.Fungstake as Program<Fungstake>;
  const connection = program.provider.connection;

  // create tx map config
  before(async () => {
    await Promise.all(
      [payer].map(async (keypair) => {
        return provider.connection
          .requestAirdrop(keypair.publicKey, 100 * LAMPORTS_PER_SOL)
          .then((sig) =>
            provider.connection.confirmTransaction(
              { signature: sig } as TransactionConfirmationStrategy,
              "processed"
            )
          );
      })
    );

    stakeCurrencyMint = await createMint(
      program.provider.connection,
      payer.payer,
      payer.publicKey,
      payer.publicKey,
      0
    );

    rewardCurrencyMint = await createMint(
      program.provider.connection,
      payer.payer,
      payer.publicKey,
      payer.publicKey,
      0
    );
  });

  it("Is initialized!", async () => {
    // Add your test here.

    const tx = await program.methods
      .initialize(10000, new BN(10000))
      .accounts({
        signer: payer.publicKey,
        stakeCurrencyMint: stakeCurrencyMint,
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });

  it("Create vault", async () => {
    const tx = await program.methods
      .createVault()
      .accounts({
        signer: payer.publicKey,
        stakeCurrencyMint: stakeCurrencyMint,
        rewardCurrencyMint: rewardCurrencyMint,
      })
      .rpc();
    console.log("Your transaction signature create vault", tx);
  });

  it("It stake", async () => {
    let userStakeTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      stakeCurrencyMint,
      payer.publicKey
    );

    console.log(
      "bal",
      (await getAccount(connection, userStakeTokenAccount.address)).amount
    );

    await mintTo(
      connection,
      payer.payer,
      stakeCurrencyMint,
      userStakeTokenAccount.address,
      payer.payer,
      200
    );

    const tx = await program.methods
      .stake(new BN(5))
      .accounts({
        signer: payer.publicKey,
        stakeCurrencyMint: stakeCurrencyMint,
        rewardCurrencyMint: rewardCurrencyMint,
      })
      .rpc();
    console.log("Your transaction signature stake", tx);
  });
});
