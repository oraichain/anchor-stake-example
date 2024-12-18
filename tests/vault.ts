import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
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
import {
  STAKE_CONFIG_SEED,
  STAKE_DETAIL_SEED,
  STAKER_INFO_SEED,
  VAULT_SEED,
} from "./constants";
import { assert } from "chai";
import { setTimeout } from "timers/promises";
import { Vault } from "../target/types/vault";

describe("vault", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const payer = provider.wallet as anchor.Wallet;
  const user2 = anchor.web3.Keypair.generate();
  const lockPeriod = 3;

  let stakeCurrencyMint: PublicKey;

  const program = anchor.workspace.Vault as Program<Vault>;
  const connection = program.provider.connection;

  // create tx map config
  before(async () => {
    await Promise.all(
      [payer, user2].map(async (keypair) => {
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
  });

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods
      .initialize()
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
    assert.equal(
      configAccount.stakeCurrencyMint.toBase58(),
      stakeCurrencyMint.toBase58()
    );
  });

  it("Create vault", async () => {
    // case 1: unauthorized
    try {
      await program.methods
        .createVault(new BN(lockPeriod))
        .accounts({
          authority: user2.publicKey,
          stakeCurrencyMint: stakeCurrencyMint,
        })
        .signers([user2])
        .rpc();
    } catch (error) {
      assert.include(JSON.stringify(error), "IncorrectAuthority");
    }

    // case 2: happy case

    const tx = await program.methods
      .createVault(new BN(lockPeriod))
      .accounts({
        authority: payer.publicKey,
        stakeCurrencyMint: stakeCurrencyMint,
      })
      .signers([payer.payer])
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
        new BN(lockPeriod).toBuffer("le", 8),
      ],
      program.programId
    );
    const vault = await program.account.vault.fetch(vaultPda);
    assert.equal(vault.lockPeriod.toNumber(), lockPeriod);
    assert.equal(vault.totalStaked.toNumber(), 0);
    assert.equal(vault.vaultConfig.toBase58(), configPda.toBase58());
    assert.equal(vault.version, 1);
  });

  it("It stake", async () => {
    let userStakeTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      stakeCurrencyMint,
      payer.publicKey
    );

    await mintTo(
      connection,
      payer.payer,
      stakeCurrencyMint,
      userStakeTokenAccount.address,
      payer.payer,
      20000000
    );

    console.log(
      "bal",
      (await getAccount(connection, userStakeTokenAccount.address)).amount
    );

    let [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from(STAKE_CONFIG_SEED), stakeCurrencyMint.toBytes()],
      program.programId
    );
    let [vaultPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from(VAULT_SEED),
        configPda.toBytes(),
        new BN(lockPeriod).toBuffer("le", 8),
      ],
      program.programId
    );

    // validate user's vault stake info
    let [userStakePda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from(STAKER_INFO_SEED),
        vaultPda.toBytes(),
        payer.publicKey.toBytes(),
      ],
      program.programId
    );
    // validate
    let [userStakeDetailPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from(STAKE_DETAIL_SEED),
        userStakePda.toBytes(),
        new BN(1).toBuffer("le", 8),
      ],
      program.programId
    );

    // case 1: first time stake, stake detail id = 0, stake info current id = 1, should increase stake value
    const tx = await program.methods
      .stake(new BN(lockPeriod), new BN(5))
      .accounts({
        signer: payer.publicKey,
        stakeCurrencyMint: stakeCurrencyMint,
        stakeDetailPda: userStakeDetailPda,
      })
      .rpc();
    console.log("Your transaction signature stake", tx);

    // validate overall vault
    let vaultInfo = await program.account.vault.fetch(vaultPda);
    assert.equal(vaultInfo.totalStaked.toNumber(), 5);

    // validate user's vault stake info
    let userStakeInfo = await program.account.stakerInfo.fetch(userStakePda);
    assert.equal(userStakeInfo.currentId.toNumber(), 1);
    assert.equal(userStakeInfo.totalStake.toNumber(), 5);

    // validate
    let userStakeDetail = await program.account.stakeDetail.fetch(
      userStakeDetailPda
    );

    assert.equal(userStakeDetail.id.toNumber(), 1);
    assert.equal(userStakeDetail.stakeAmount.toNumber(), 5);

    // case 2: stake more, but incorrect stake id
    try {
      let [userStakeDetailPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from(STAKE_DETAIL_SEED),
          userStakePda.toBytes(),
          new BN(0).toBuffer("le", 8),
        ],
        program.programId
      );
      await program.methods
        .stake(new BN(lockPeriod), new BN(50))
        .accounts({
          signer: payer.publicKey,
          stakeCurrencyMint: stakeCurrencyMint,
          stakeDetailPda: userStakeDetailPda,
        })
        .rpc();
    } catch (error) {
      assert.include(JSON.stringify(error), "ConstraintSeeds.");
    }

    [userStakeDetailPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from(STAKE_DETAIL_SEED),
        userStakePda.toBytes(),
        new BN(2).toBuffer("le", 8),
      ],
      program.programId
    );
    // case 3: stake more -> different stake detail
    await program.methods
      .stake(new BN(lockPeriod), new BN(50))
      .accounts({
        signer: payer.publicKey,
        stakeCurrencyMint: stakeCurrencyMint,
        stakeDetailPda: userStakeDetailPda,
      })
      .rpc();

    // validate overall vault
    vaultInfo = await program.account.vault.fetch(vaultPda);
    assert.equal(vaultInfo.totalStaked.toNumber(), 55);

    // validate user stake vault
    userStakeInfo = await program.account.stakerInfo.fetch(userStakePda);
    assert.equal(userStakeInfo.totalStake.toNumber(), 55);
    assert.equal(userStakeInfo.currentId.toNumber(), 2);

    // validate user stake detail
    userStakeDetail = await program.account.stakeDetail.fetch(
      userStakeDetailPda
    );

    assert.equal(userStakeDetail.id.toNumber(), 2);
    assert.equal(userStakeDetail.stakeAmount.toNumber(), 50);
  });

  // it("It unstake before reach soft cap", async () => {
  //   let [configPda] = PublicKey.findProgramAddressSync(
  //     [Buffer.from(STAKE_CONFIG_SEED), stakeCurrencyMint.toBytes()],
  //     program.programId
  //   );
  //   let [vaultPda] = PublicKey.findProgramAddressSync(
  //     [
  //       Buffer.from(VAULT_SEED),
  //       configPda.toBytes(),
  //       rewardCurrencyMint.toBytes(),
  //     ],
  //     program.programId
  //   );
  //   let [userStakePda] = PublicKey.findProgramAddressSync(
  //     [
  //       Buffer.from(STAKE_INFO_SEED),
  //       vaultPda.toBytes(),
  //       payer.publicKey.toBytes(),
  //     ],
  //     program.programId
  //   );
  //   let userStakeBefore = await program.account.stakeInfo.fetch(userStakePda);

  //   // try unstake
  //   // case 1: error: The unbonding time is not over yet.
  //   let willThrow = false;
  //   try {
  //     await program.methods
  //       .destake(new BN(10))
  //       .accounts({
  //         signer: payer.publicKey,
  //         stakeCurrencyMint: stakeCurrencyMint,
  //         rewardCurrencyMint: rewardCurrencyMint,
  //       })
  //       .rpc();
  //   } catch (error) {
  //     willThrow = true;
  //     assert.include(error.toString(), "UnbondingTimeNotOverYet");
  //   }
  //   assert.equal(willThrow, true);

  //   // case 2:  success
  //   await setTimeout((lockPeriod + 1) * 1000);
  //   await program.methods
  //     .destake(new BN(10))
  //     .accounts({
  //       signer: payer.publicKey,
  //       stakeCurrencyMint: stakeCurrencyMint,
  //       rewardCurrencyMint: rewardCurrencyMint,
  //     })
  //     .rpc();
  //   let userStakeAfter = await program.account.stakeInfo.fetch(userStakePda);
  //   assert.equal(
  //     userStakeBefore.stakeAmount.toNumber() - 10,
  //     userStakeAfter.stakeAmount.toNumber()
  //   );
  // });
});
