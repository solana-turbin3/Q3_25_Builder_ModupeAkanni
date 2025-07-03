import wallet from "./turbin3-wallet.json";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  createGenericFile,
  createSignerFromKeypair,
  signerIdentity,
} from "@metaplex-foundation/umi";
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys";

// Create a devnet connection
const umi = createUmi("https://api.devnet.solana.com");

let keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(irysUploader());
umi.use(signerIdentity(signer));

(async () => {
  try {
    // Follow this JSON structure
    // https://docs.metaplex.com/programs/token-metadata/changelog/v1.0#json-structure

    const image =
      "https://gateway.irys.xyz/3f2idExHxodbQZTxi4cvwxR1AL1d15PJnzGgkhKSFbTN";
    const metadata = {
      name: "Goke",
      symbol: "GKT",
      description: "GK NFT",
      image: image,
      attributes: [{ trait_type: "type", value: "legendary" }],
      properties: {
        files: [
          {
            type: "image/png",
            uri: image,
          },
        ],
      },
      creators: [
        {
          address: signer.publicKey.toString(),
          share: 100,
          verified: true,
        },
      ],
    };

    const genericFile = createGenericFile(
      JSON.stringify(metadata),
      "generug.png"
    );
    const [myUri] = await umi.uploader.upload([genericFile]);
    console.log("Your metadata URI: ", myUri);
  } catch (error) {
    console.log("Oops.. Something went wrong", error);
  }
})();
