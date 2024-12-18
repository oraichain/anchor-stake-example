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
import { STAKE_CONFIG_SEED } from "./constants";
import { assert } from "chai";

describe("fungstake", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const payer = provider.wallet as anchor.Wallet;
  const lockPeriod = 10000;
  const lockExtendTime = 100;
  const softCap = 10000;

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
      .initialize(lockPeriod, lockExtendTime, new BN(softCap))
      .accounts({
        signer: payer.publicKey,
        stakeCurrencyMint: stakeCurrencyMint,
      })
      .rpc();
    console.log("Your transaction signature", tx);
    // get config
    let [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from(STAKE_CONFIG_SEED), stakeCurrencyMint.toBytes()],
      program.programId
    );
    const configAccount = await program.account.stakeConfig.fetch(configPda);
    assert.equal(
      configAccount.authority.toBase58(),
      payer.publicKey.toBase58()
    );
    assert.equal(
      configAccount.stakeCurrencyMint.toBase58(),
      stakeCurrencyMint.toBase58()
    );
    assert.equal(configAccount.lockPeriod, lockPeriod);
    assert.equal(configAccount.lockExtendTime, lockExtendTime);
    assert.equal(configAccount.softCap.toString(), softCap.toString());
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
