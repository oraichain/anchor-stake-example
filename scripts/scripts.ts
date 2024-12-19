import { BN, Program, web3 } from "@coral-xyz/anchor";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { Vault } from "../target/types/vault";
import {
  CLUSTER,
  CONFIG,
  STAKE_CONFIG_SEED,
  STAKE_DETAIL_SEED,
  STAKER_INFO_SEED,
  VAULT_SEED,
} from "./constant";
import { execTx } from "./util";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import "dotenv/config";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import * as anchor from "@coral-xyz/anchor";

const globalConfig = CONFIG[CLUSTER];
let solConnection: Connection = null;
let program: Program<Vault> = null;
let payer: NodeWallet = null;
let payerKeypair: Keypair;

export const setClusterConfig = async () => {
  let rpc = process.env.SOLANA_RPC_ENDPOINT;
  const keypair = Keypair.fromSecretKey(
    bs58.decode(process.env.SOLANA_PRIVATE_KEY)
  );
  if (!rpc) {
    solConnection = new web3.Connection(web3.clusterApiUrl(CLUSTER));
  } else {
    solConnection = new web3.Connection(rpc);
  }

  payerKeypair = keypair;
  payer = new NodeWallet(payerKeypair);

  console.log("Wallet Address: ", payer.publicKey.toBase58());

  anchor.setProvider(
    new anchor.AnchorProvider(solConnection, payer, {
      skipPreflight: true,
      commitment: "confirmed",
    })
  );

  // Generate the program client from IDL.
  program = anchor.workspace.Vault as Program<Vault>;
};

const createConfigTx = async () => {
  await setClusterConfig();
  const tx = await program.methods
    .initialize()
    .accounts({
      signer: payerKeypair.publicKey,
      stakeCurrencyMint: globalConfig.STAKE_CURRENCY_MINT,
    })
    .transaction();

  tx.feePayer = payerKeypair.publicKey;
  tx.recentBlockhash = (await solConnection.getLatestBlockhash()).blockhash;
  await execTx(tx, solConnection, payer);
};

const createVault = async (lockPeriod: number) => {
  await setClusterConfig();

  const tx = await program.methods
    .createVault(new BN(lockPeriod))
    .accounts({
      authority: payer.publicKey,
      stakeCurrencyMint: globalConfig.STAKE_CURRENCY_MINT,
    })
    .signers([payer.payer])
    .transaction();

  tx.feePayer = payerKeypair.publicKey;
  tx.recentBlockhash = (await solConnection.getLatestBlockhash()).blockhash;
  await execTx(tx, solConnection, payer);
};

const stake = async (lockPeriod: number, amount: number) => {
  await setClusterConfig();
  const stakeCurrencyMint = globalConfig.STAKE_CURRENCY_MINT;
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
  let [stakerInfoPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from(STAKER_INFO_SEED),
      vaultPda.toBytes(),
      payer.publicKey.toBytes(),
    ],
    program.programId
  );
  let currentId = 0;
  try {
    let stakerInfo = await program.account.stakerInfo.fetch(stakerInfoPda);
    currentId = stakerInfo.currentId.toNumber();
  } catch (error) {}

  let [userStakeDetailPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from(STAKE_DETAIL_SEED),
      stakerInfoPda.toBytes(),
      new BN(currentId + 1).toBuffer("le", 8),
    ],
    program.programId
  );

  const tx = await program.methods
    .stake(new BN(lockPeriod), new BN(amount))
    .accounts({
      signer: payer.publicKey,
      stakeCurrencyMint: stakeCurrencyMint,
      stakeDetailPda: userStakeDetailPda,
    })
    .transaction();

  tx.feePayer = payerKeypair.publicKey;
  tx.recentBlockhash = (await solConnection.getLatestBlockhash()).blockhash;
  await execTx(tx, solConnection, payer);
};

const unStake = async (lockPeriod: number, id: number, amount: number) => {
  await setClusterConfig();
  const stakeCurrencyMint = globalConfig.STAKE_CURRENCY_MINT;

  const tx = await program.methods
    .destake(new BN(id), new BN(lockPeriod), new BN(amount))
    .accounts({
      signer: payer.publicKey,
      stakeCurrencyMint: stakeCurrencyMint,
    })
    .transaction();

  tx.feePayer = payerKeypair.publicKey;
  tx.recentBlockhash = (await solConnection.getLatestBlockhash()).blockhash;
  await execTx(tx, solConnection, payer);
};
