import { BN } from "@coral-xyz/anchor";

import { PublicKey } from "@solana/web3.js";

export const CLUSTER = "devnet"; // mainnet-beta | devnet

export const CONFIG = {
  devnet: {
    STAKE_CURRENCY_MINT: new PublicKey(
      "3Ff7yUkQsbMzViXu7aAxAYsgpy31wY8R8TteE39FDuw4"
    ),
  },
};

export const VAULT_SEED = "staking_vault";
export const STAKE_CONFIG_SEED = "staking_config";
export const STAKER_INFO_SEED = "staker_info";
export const STAKE_INFO_SEED = "stake_info";
export const STAKE_DETAIL_SEED = "stake_detail";
