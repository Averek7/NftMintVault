import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NftMintVault } from "../target/types/nft_mint_vault";
import { walletAdapterIdentity } from "@metaplex-foundation/umi-signer-wallet-adapters";
import {
  createAssociatedTokenAccount,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import {
  findMasterEditionPda,
  findMetadataPda,
  mplTokenMetadata,
  MPL_TOKEN_METADATA_PROGRAM_ID,
} from "@metaplex-foundation/mpl-token-metadata";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { publicKey } from "@metaplex-foundation/umi";

import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import * as spl from "@solana/spl-token";
import { Connection } from "@solana/web3.js";
import { min } from "bn.js";

describe("mint-nft", async () => {
  // Configured the client to use the devnet cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.NftMintVault as Program<NftMintVault>;

  const signer = provider.wallet;
  const umi = createUmi("https://api.devnet.solana.com")
    .use(walletAdapterIdentity(signer))
    .use(mplTokenMetadata());

  const mint = anchor.web3.Keypair.generate();
  console.log(`mint: ${mint.publicKey.toBase58()}`);
  const vault = anchor.web3.Keypair.generate();
  console.log(`vault: ${vault.publicKey.toBase58()}`);

  const ata = await spl.getAssociatedTokenAddress(
    mint.publicKey,
    signer.publicKey
  );

  const vaultTokenAccount = await spl.getAssociatedTokenAddress(
    mint.publicKey,
    vault.publicKey
  );

  // derive the metadata account
  let metadataAccount = findMetadataPda(umi, {
    mint: publicKey(mint.publicKey),
  })[0];

  //derive the master edition pda
  let masterEditionAccount = findMasterEditionPda(umi, {
    mint: publicKey(mint.publicKey),
  })[0];

  const metadata = {
    name: "Kobeni",
    symbol: "kBN",
    uri: "https://raw.githubusercontent.com/687c/solana-nft-native-client/main/metadata.json",
  };

  it("mints nft!", async () => {
    const tx = await program.methods
      .initNft(metadata.name, metadata.symbol, metadata.uri)
      .accounts({
        signer: provider.publicKey,
        mint: mint.publicKey,
        ata,
        metadataAccount,
        masterEditionAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenMetadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([mint])
      .rpc();

    console.log(
      `mint nft tx: https://explorer.solana.com/tx/${tx}?cluster=devnet`
    );
    console.log(
      `minted nft: https://explorer.solana.com/address/${mint.publicKey}?cluster=devnet`
    );
  });

  it("creates vault!", async () => {
    const tx = await program.methods
      .createVault(mint.publicKey)
      .accounts({
        vault: vault.publicKey,
        owner: provider.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([vault])
      .rpc();

    console.log(
      `create vault tx: https://explorer.solana.com/tx/${tx}?cluster=devnet`
    );
  });

  it("locks nft in vault!", async () => {
    const tx = await program.methods
      .lockNft()
      .accounts({
        vault: vault.publicKey,
        owner: provider.publicKey,
        nftTokenAccount: ata,
        vaultTokenAccount: vaultTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([])
      .rpc();

    console.log(
      `Lock NFT transaction: https://explorer.solana.com/tx/${tx}?cluster=devnet`
    );
  });

  it("executes swap!", async () => {
    const swap = anchor.web3.Keypair.generate();

    const buyerTokenAccount = await getAssociatedTokenAddress(
      mint.publicKey,
      provider.publicKey
    );

    // Execute the swap
    const tx = await program.methods
      .executeSwap()
      .accounts({
        swap: swap.publicKey,
        buyer: provider.publicKey,
        seller: ata,
        nftTokenAccount: ata,
        buyerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log(
      `execute swap tx: https://explorer.solana.com/tx/${tx}?cluster=devnet`
    );
  });
});
