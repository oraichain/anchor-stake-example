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
import { STAKE_CONFIG_SEED, STAKE_INFO_SEED, VAULT_SEED } from "./constants";
import { assert } from "chai";
import { setTimeout } from "timers/promises";

describe("fungstake", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const payer = provider.wallet as anchor.Wallet;
  const lockPeriod = 3;
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

    // get vault
    let [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from(STAKE_CONFIG_SEED), stakeCurrencyMint.toBytes()],
      program.programId
    );
    let [vaultPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from(VAULT_SEED),
        configPda.toBytes(),
        rewardCurrencyMint.toBytes(),
      ],
      program.programId
    );
    const vault = await program.account.vault.fetch(vaultPda);
    assert.equal(
      vault.rewardCurrencyMint.toBase58(),
      rewardCurrencyMint.toBase58()
    );
    assert.equal(vault.totalStaked.toNumber(), 0);
    assert.equal(vault.endTime.toNumber(), 0);
    assert.equal(vault.reachSoftCap, false);
    assert.equal(vault.totalReward.toNumber(), 0);
    assert.equal(vault.reachTge, false);
  });

  it("It stake before reach soft cap", async () => {
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
      20000000
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

    let [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from(STAKE_CONFIG_SEED), stakeCurrencyMint.toBytes()],
      program.programId
    );
    let [vaultPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from(VAULT_SEED),
        configPda.toBytes(),
        rewardCurrencyMint.toBytes(),
      ],
      program.programId
    );
    let [userStakePda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from(STAKE_INFO_SEED),
        vaultPda.toBytes(),
        payer.publicKey.toBytes(),
      ],
      program.programId
    );
    let userStakeInfo = await program.account.stakeInfo.fetch(userStakePda);
    assert.equal(userStakeInfo.stakeAmount.toNumber(), 5);
    assert.equal(userStakeInfo.snapshotAmount.toNumber(), 5);

    // stake more
    await program.methods
      .stake(new BN(50))
      .accounts({
        signer: payer.publicKey,
        stakeCurrencyMint: stakeCurrencyMint,
        rewardCurrencyMint: rewardCurrencyMint,
      })
      .rpc();
    userStakeInfo = await program.account.stakeInfo.fetch(userStakePda);
    assert.equal(userStakeInfo.stakeAmount.toNumber(), 55);
    assert.equal(userStakeInfo.snapshotAmount.toNumber(), 55);

    const vault = await program.account.vault.fetch(vaultPda);
    assert.equal(vault.totalStaked.toNumber(), 55);
  });

  it("It unstake before reach soft cap", async () => {
    let [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from(STAKE_CONFIG_SEED), stakeCurrencyMint.toBytes()],
      program.programId
    );
    let [vaultPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from(VAULT_SEED),
        configPda.toBytes(),
        rewardCurrencyMint.toBytes(),
      ],
      program.programId
    );
    let [userStakePda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from(STAKE_INFO_SEED),
        vaultPda.toBytes(),
        payer.publicKey.toBytes(),
      ],
      program.programId
    );
    let userStakeBefore = await program.account.stakeInfo.fetch(userStakePda);

    // try unstake
    // case 1: error: The unbonding time is not over yet.
    let willThrow = false;
    try {
      await program.methods
        .destake(new BN(10))
        .accounts({
          signer: payer.publicKey,
          stakeCurrencyMint: stakeCurrencyMint,
          rewardCurrencyMint: rewardCurrencyMint,
        })
        .rpc();
    } catch (error) {
      willThrow = true;
      assert.include(error.toString(), "UnbondingTimeNotOverYet");
    }
    assert.equal(willThrow, true);

    // case 2:  success
    await setTimeout((lockPeriod + 1) * 1000);
    await program.methods
      .destake(new BN(10))
      .accounts({
        signer: payer.publicKey,
        stakeCurrencyMint: stakeCurrencyMint,
        rewardCurrencyMint: rewardCurrencyMint,
      })
      .rpc();
    let userStakeAfter = await program.account.stakeInfo.fetch(userStakePda);
    assert.equal(
      userStakeBefore.stakeAmount.toNumber() - 10,
      userStakeAfter.stakeAmount.toNumber()
    );
  });
});
