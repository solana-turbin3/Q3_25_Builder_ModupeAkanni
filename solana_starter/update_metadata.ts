import wallet from "./turbin3-wallet.json";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  fetchMetadataFromSeeds,
  updateMetadataAccountV2,
  UpdateMetadataAccountV2InstructionAccounts,
  UpdateMetadataAccountV2InstructionArgs,
  DataV2Args,
} from "@metaplex-foundation/mpl-token-metadata";
import {
  createSignerFromKeypair,
  signerIdentity,
  publicKey,
} from "@metaplex-foundation/umi";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";

// Define our Mint address
const mint = publicKey("E7qSkYuSmNcvfq4AcZURVN7RBys8fWmBqg961CrXCHVM");

// Create a UMI connection
const umi = createUmi("https://api.devnet.solana.com");
const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);
umi.use(signerIdentity(createSignerFromKeypair(umi, keypair)));

(async () => {
  try {
    // Start here

    const initialMetadata = await fetchMetadataFromSeeds(umi, { mint });

    let accounts: UpdateMetadataAccountV2InstructionAccounts = {
      metadata: initialMetadata.publicKey,
      updateAuthority: signer,
    };

    let data: DataV2Args = {
      name: "Goke",
      symbol: "GKT",
      uri: "https://gist.githubusercontent.com/Goketech/190889411d78c7ebdd3891e95bc52751/raw/f66d98c43933a68d4f2e858a5788dbac441980aa/nft.json",
      sellerFeeBasisPoints: 500,
      creators: initialMetadata.creators,
      collection: initialMetadata.collection,
      uses: initialMetadata.uses,
    };

    let args: UpdateMetadataAccountV2InstructionArgs = {
      data,
      isMutable: true,
    };

    let tx = updateMetadataAccountV2(umi, {
      ...accounts,
      ...args,
    });

    let result = await tx.sendAndConfirm(umi);
    console.log(bs58.encode(result.signature));
  } catch (e) {
    console.error(`Oops, something went wrong: ${e}`);
  }
})();
