import { PublicKey } from "@solana/web3.js";
import { PROGRAM_ADDRESS as TM_PROGRAM_ADDRESS } from "@metaplex-foundation/mpl-token-metadata";
import { PROGRAM_ADDRESS as TRIFLE_PROGRAM_ADDRESS } from "../../../js/src/generated";
import { EscrowAuthority } from "./utils";

export const findEscrowConstraintModelPda = async (
  creator: PublicKey,
  name: string,
) => {
  return await PublicKey.findProgramAddress(
    [Buffer.from("escrow"), creator.toBuffer(), Buffer.from(name)],
    new PublicKey(TRIFLE_PROGRAM_ADDRESS),
  );
};

export const findTriflePda = async (mint: PublicKey, authority: PublicKey) => {
  return await PublicKey.findProgramAddress(
    [Buffer.from("trifle"), mint.toBuffer(), authority.toBuffer()],
    new PublicKey(TRIFLE_PROGRAM_ADDRESS),
  );
};

export const findEscrowPda = async (
  mint: PublicKey,
  authority: EscrowAuthority,
  creator?: PublicKey,
) => {
  const seeds = [
    Buffer.from("metadata"),
    new PublicKey(TM_PROGRAM_ADDRESS).toBuffer(),
    mint.toBuffer(),
    Uint8Array.from([authority]),
  ];

  if (authority == EscrowAuthority.Creator) {
    if (creator) {
      seeds.push(creator.toBuffer());
    } else {
      throw new Error("Creator is required");
    }
  }

  seeds.push(Buffer.from("escrow"));
  return await PublicKey.findProgramAddress(
    seeds,
    new PublicKey(TM_PROGRAM_ADDRESS),
  );
};
