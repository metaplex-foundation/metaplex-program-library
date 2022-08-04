import * as anchor from "@project-serum/anchor";
import { keccak_256 } from "js-sha3";
import { BN, AnchorProvider, Program } from "@project-serum/anchor";
import { Bubblegum } from "../target/types/bubblegum";
import { Gummyroll } from "../target/types/gummyroll";
import {
  PROGRAM_ID as TOKEN_METADATA_PROGRAM_ID,
  metadataBeet,
  Metadata,
  Data,
  TokenStandard,
} from "@metaplex-foundation/mpl-token-metadata";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  Transaction,
  Connection as web3Connection,
  SYSVAR_RENT_PUBKEY,
  Connection,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { assert } from "chai";
import {
  createMintV1Instruction,
  createDecompressV1Instruction,
  createTransferInstruction,
  createDelegateInstruction,
  createRedeemInstruction,
  createCancelRedeemInstruction,
  MetadataArgs,
<<<<<<< HEAD
  createRequestMintAuthorityInstruction,
  createApproveMintAuthorityRequestInstruction,
  createCloseMintRequestInstruction,
  MintRequest,
  TreeConfig,
  createSetTreeDelegateInstruction,
  createCreateDefaultMintRequestInstruction,
} from "../sdk/bubblegum/src/generated";

=======
  TokenProgramVersion,
  Version,
  Creator,
  getBubblegumAuthorityPDA,
  getCreateTreeIxs,
  getNonceCount,
  getVoucherPDA,
  computeDataHash,
  computeCreatorHash
} from "@sorend-solana/bubblegum";
>>>>>>> main
import { buildTree, Tree } from "./merkle-tree";
import {
  getRootOfOnChainMerkleRoot,
  assertOnChainMerkleRollProperties,
<<<<<<< HEAD
} from "../sdk/gummyroll";
=======
  createTransferAuthorityIx,
  createAllocTreeIx,
} from "@sorend-solana/gummyroll";
>>>>>>> main
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Token,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
<<<<<<< HEAD
import {
  TokenProgramVersion,
  Version,
  Creator,
} from "../sdk/bubblegum/src/generated";
import {
  CANDY_WRAPPER_PROGRAM_ID,
  execute,
  bufferToArray,
  num16ToBuffer,
  trimStringPadding,
} from "../sdk/utils";
import {
  computeDataHash,
  computeCreatorHash,
  getBubblegumAuthorityPDA,
  getCreateTreeIxs,
  getMintRequestPDA,
  getNonceCount,
  getVoucherPDA,
  assertOnChainMintRequest,
  assertOnChainTreeAuthority,
  getDefaultMintRequestPDA,
} from "../sdk/bubblegum/src/convenience";
// import { getBubblegumAuthorityPDA, getCreateTreeIxs, getMintRequestPDA, getNonceCount, getVoucherPDA, assertOnChainMintRequest, assertOnChainTreeAuthority } from "../sdk/bubblegum/src/convenience";
=======
import { CANDY_WRAPPER_PROGRAM_ID, execute, bufferToArray, strToByteArray, arrayEquals, trimStringPadding } from "@sorend-solana/utils";
>>>>>>> main

// @ts-ignore
let Bubblegum;
// @ts-ignore
let GummyrollProgramId;

describe("bubblegum", function () {
  // Configure the client to use the local cluster.
  let offChainTree: Tree;
  let treeAuthority: PublicKey;
  let merkleRollKeypair: Keypair;
  let mintRequest: PublicKey;

  let payer: Keypair;
  let destination: Keypair;
  let delegateKey: Keypair;
  let connection: Connection;
  let wallet: NodeWallet;

  const MAX_SIZE = 64;
  const MAX_DEPTH = 20;

  async function createTreeOnChain(
    payer: Keypair,
    destination: Keypair,
    delegate: Keypair
  ): Promise<[Keypair, Tree, PublicKey, PublicKey]> {
    const merkleRollKeypair = Keypair.generate();

    await Bubblegum.provider.connection.confirmTransaction(
      await Bubblegum.provider.connection.requestAirdrop(payer.publicKey, 2e9),
      "confirmed"
    );
    await Bubblegum.provider.connection.confirmTransaction(
      await Bubblegum.provider.connection.requestAirdrop(
        destination.publicKey,
        2e9
      ),
      "confirmed"
    );
    await Bubblegum.provider.connection.confirmTransaction(
      await Bubblegum.provider.connection.requestAirdrop(
        delegate.publicKey,
        2e9
      ),
      "confirmed"
    );
    const leaves = Array(2 ** MAX_DEPTH).fill(Buffer.alloc(32));
    const tree = buildTree(leaves);
    let ixs = await getCreateTreeIxs(
      Bubblegum.provider.connection,
      MAX_DEPTH,
      MAX_SIZE,
      0,
      payer.publicKey,
      merkleRollKeypair.publicKey,
      payer.publicKey
    );
    const authority = await getBubblegumAuthorityPDA(
      merkleRollKeypair.publicKey
    );
    const defaultMintRequestKey = await getDefaultMintRequestPDA(
      merkleRollKeypair.publicKey
    );
    let defaultMintRequestIx = createCreateDefaultMintRequestInstruction(
      {
        merkleSlab: merkleRollKeypair.publicKey,
        creator: payer.publicKey,
        payer: payer.publicKey,
        treeAuthority: authority,
        mintAuthorityRequest: defaultMintRequestKey,
      },
      { mintCapacity: 2 ** (MAX_DEPTH - 1) },
      Bubblegum.programId
    );
    ixs.push(defaultMintRequestIx);
    let approveDefaultMintRequestIx =
      createApproveMintAuthorityRequestInstruction(
        {
          merkleSlab: merkleRollKeypair.publicKey,
          treeDelegate: payer.publicKey,
          treeAuthority: authority,
          mintAuthorityRequest: defaultMintRequestKey,
        },
        { numMintsToApprove: 2 ** (MAX_DEPTH - 1) },
        Bubblegum.programId
      );
    ixs.push(approveDefaultMintRequestIx);
    await execute(Bubblegum.provider, ixs, [payer, merkleRollKeypair]);

    await assertOnChainMerkleRollProperties(
      Bubblegum.provider.connection,
      MAX_DEPTH,
      MAX_SIZE,
      authority,
      new PublicKey(tree.root),
      merkleRollKeypair.publicKey
    );

    return [merkleRollKeypair, tree, authority, defaultMintRequestKey];
  }

  const getMetadata = async (
    mint: anchor.web3.PublicKey
  ): Promise<anchor.web3.PublicKey> => {
    return (
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("metadata"),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          mint.toBuffer(),
        ],
        TOKEN_METADATA_PROGRAM_ID
      )
    )[0];
  };

  const getMasterEdition = async (
    mint: anchor.web3.PublicKey
  ): Promise<anchor.web3.PublicKey> => {
    return (
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("metadata"),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          mint.toBuffer(),
          Buffer.from("edition"),
        ],
        TOKEN_METADATA_PROGRAM_ID
      )
    )[0];
  };

  const assertMetadataMatch = (
    onChainMetadata: Metadata,
    mintMetadataArgs: MetadataArgs,
    expectedMintAuthority: PublicKey
  ) => {
    const assertDataMatch = (onChainData: Data, expectedData: Data) => {
      assert(
        trimStringPadding(onChainData.name) === expectedData.name,
        "names mismatched"
      );
      assert(
        trimStringPadding(onChainData.symbol) === expectedData.symbol,
        "symbols mismatched"
      );
      assert(
        trimStringPadding(onChainData.uri) === expectedData.uri,
        "uris mismatched"
      );
      assert(
        onChainData.sellerFeeBasisPoints === expectedData.sellerFeeBasisPoints
      );
      onChainData.creators?.forEach((creator, index) => {
        if (index === onChainData.creators.length - 1) {
          assert(
            creator.address.equals(expectedMintAuthority),
            "Creator address mismatch"
          );
          assert(creator.share === 0, "Creator share mismatch");
          assert(creator.verified === true, "Creator verified mismatch");
        } else {
          assert(
            creator.address.equals(expectedData.creators[index].address),
            "Creator address mismatch"
          );
          assert(
            creator.share === expectedData.creators[index].share,
            "Creator share mismatch"
          );
          assert(
            creator.verified === expectedData.creators[index].verified,
            "Creator verified mismatch"
          );
        }
      });
    };

    // Assert that data fields match
    assertDataMatch(onChainMetadata.data, {
      name: mintMetadataArgs.name,
      uri: mintMetadataArgs.uri,
      symbol: mintMetadataArgs.symbol,
      creators: mintMetadataArgs.creators,
      sellerFeeBasisPoints: mintMetadataArgs.sellerFeeBasisPoints,
    });

    // Assert that collections match
    assert(
      !onChainMetadata.collection
        ? onChainMetadata.collection === null
        : onChainMetadata.collection.key.equals(
            mintMetadataArgs.collection.key
          ) &&
            onChainMetadata.collection.verified ===
              mintMetadataArgs.collection.verified,
      "Collections did not match"
    );

    // Assert remaining properties match. TODO: at some point some of these comparrisons may need to be updated to work for non-null values
    assert(
      onChainMetadata.isMutable === mintMetadataArgs.isMutable,
      "isMutable did not match"
    );
    assert(
      onChainMetadata.primarySaleHappened ===
        mintMetadataArgs.primarySaleHappened,
      "primary sale mismatch"
    );
    assert(
      onChainMetadata.tokenStandard === TokenStandard.NonFungible,
      "token standard mismatch"
    );
    assert(
      onChainMetadata.updateAuthority.equals(expectedMintAuthority),
      "mint authority mismatch"
    );
    assert(onChainMetadata.uses === mintMetadataArgs.uses, "uses mismatch");
  };

  beforeEach(async function () {
    payer = Keypair.generate();
    destination = Keypair.generate();
    delegateKey = Keypair.generate();
    connection = new web3Connection("http://localhost:8899", {
      commitment: "confirmed",
    });
    wallet = new NodeWallet(payer);
    anchor.setProvider(
      new AnchorProvider(connection, wallet, {
        commitment: connection.commitment,
        skipPreflight: true,
      })
    );
    Bubblegum = anchor.workspace.Bubblegum as Program<Bubblegum>;
    GummyrollProgramId = anchor.workspace.Gummyroll.programId;

    let [
      computedMerkleRoll,
      computedOffChainTree,
      computedTreeAuthority,
      computedMintRequestKey,
    ] = await createTreeOnChain(payer, destination, delegateKey);
    merkleRollKeypair = computedMerkleRoll;
    offChainTree = computedOffChainTree;
    treeAuthority = computedTreeAuthority;
    mintRequest = computedMintRequestKey;
  });

  it("All operations work, metadata without creators", async function () {
    const metadata: MetadataArgs = {
      name: "test",
      symbol: "test",
      uri: "www.solana.com",
      sellerFeeBasisPoints: 0,
      primarySaleHappened: false,
      isMutable: false,
      editionNonce: null,
      tokenStandard: null,
      tokenProgramVersion: TokenProgramVersion.Original,
      collection: null,
      uses: null,
      creators: [],
    };
    console.log(" - Minting to tree");
    const mintIx = createMintV1Instruction(
      {
        mintAuthority: treeAuthority,
        // Dummy key
        mintAuthorityRequest: mintRequest,
        authority: treeAuthority,
        candyWrapper: CANDY_WRAPPER_PROGRAM_ID,
        gummyrollProgram: GummyrollProgramId,
        owner: payer.publicKey,
        delegate: payer.publicKey,
        merkleSlab: merkleRollKeypair.publicKey,
      },
      { message: metadata }
    );
    await execute(Bubblegum.provider, [mintIx], [payer], true);

    // Compute data hash
    const dataHash = computeDataHash(metadata.sellerFeeBasisPoints, mintIx);

    // Compute creator hash
    const creatorHash = computeCreatorHash([]);

    let onChainRoot = await getRootOfOnChainMerkleRoot(
      connection,
      merkleRollKeypair.publicKey
    );

    console.log(" - Transferring Ownership");
    const nonceCount = await getNonceCount(
      Bubblegum.provider.connection,
      merkleRollKeypair.publicKey
    );
    const leafNonce = nonceCount.sub(new BN(1));
    let transferIx = createTransferInstruction(
      {
        authority: treeAuthority,
        owner: payer.publicKey,
        delegate: payer.publicKey,
        newOwner: destination.publicKey,
        candyWrapper: CANDY_WRAPPER_PROGRAM_ID,
        gummyrollProgram: GummyrollProgramId,
        merkleSlab: merkleRollKeypair.publicKey,
      },
      {
        root: bufferToArray(onChainRoot),
        dataHash,
        creatorHash,
        nonce: leafNonce,
        index: 0,
      }
    );
    await execute(Bubblegum.provider, [transferIx], [payer], true);

    onChainRoot = await getRootOfOnChainMerkleRoot(
      connection,
      merkleRollKeypair.publicKey
    );

    console.log(" - Delegating Ownership");
    let delegateIx = await createDelegateInstruction(
      {
        authority: treeAuthority,
        owner: destination.publicKey,
        previousDelegate: destination.publicKey,
        newDelegate: delegateKey.publicKey,
        candyWrapper: CANDY_WRAPPER_PROGRAM_ID,
        gummyrollProgram: GummyrollProgramId,
        merkleSlab: merkleRollKeypair.publicKey,
      },
      {
        root: bufferToArray(onChainRoot),
        dataHash,
        creatorHash,
        nonce: leafNonce,
        index: 0,
      }
    );
    await execute(Bubblegum.provider, [delegateIx], [destination], true);

    onChainRoot = await getRootOfOnChainMerkleRoot(
      connection,
      merkleRollKeypair.publicKey
    );

    console.log(" - Transferring Ownership (through delegate)");
    let delTransferIx = createTransferInstruction(
      {
        authority: treeAuthority,
        owner: destination.publicKey,
        delegate: delegateKey.publicKey,
        newOwner: payer.publicKey,
        candyWrapper: CANDY_WRAPPER_PROGRAM_ID,
        gummyrollProgram: GummyrollProgramId,
        merkleSlab: merkleRollKeypair.publicKey,
      },
      {
        root: bufferToArray(onChainRoot),
        dataHash,
        creatorHash,
        nonce: leafNonce,
        index: 0,
      }
    );
    delTransferIx.keys[2].isSigner = true;
    await execute(Bubblegum.provider, [delTransferIx], [delegateKey], true);

    onChainRoot = await getRootOfOnChainMerkleRoot(
      connection,
      merkleRollKeypair.publicKey
    );

    let voucher = await getVoucherPDA(
      Bubblegum.provider.connection,
      merkleRollKeypair.publicKey,
      0
    );

    console.log(" - Redeeming Leaf", voucher.toBase58());
    let redeemIx = createRedeemInstruction(
      {
        authority: treeAuthority,
        owner: payer.publicKey,
        delegate: payer.publicKey,
        candyWrapper: CANDY_WRAPPER_PROGRAM_ID,
        gummyrollProgram: GummyrollProgramId,
        merkleSlab: merkleRollKeypair.publicKey,
        voucher: voucher,
      },
      {
        root: bufferToArray(onChainRoot),
        dataHash,
        creatorHash,
        nonce: new BN(0),
        index: 0,
      }
    );
    await execute(Bubblegum.provider, [redeemIx], [payer], true);

    console.log(" - Cancelling redeem (reinserting to tree)");

    const cancelRedeemIx = createCancelRedeemInstruction(
      {
        authority: treeAuthority,
        owner: payer.publicKey,
        candyWrapper: CANDY_WRAPPER_PROGRAM_ID,
        gummyrollProgram: GummyrollProgramId,
        merkleSlab: merkleRollKeypair.publicKey,
        voucher: voucher,
      },
      {
        root: bufferToArray(onChainRoot),
      }
    );
    await execute(Bubblegum.provider, [cancelRedeemIx], [payer], true);

    console.log(" - Decompressing leaf");

    redeemIx = createRedeemInstruction(
      {
        authority: treeAuthority,
        owner: payer.publicKey,
        delegate: payer.publicKey,
        candyWrapper: CANDY_WRAPPER_PROGRAM_ID,
        gummyrollProgram: GummyrollProgramId,
        merkleSlab: merkleRollKeypair.publicKey,
        voucher: voucher,
      },
      {
        root: bufferToArray(onChainRoot),
        dataHash,
        creatorHash,
        nonce: leafNonce,
        index: 0,
      }
    );
    await execute(Bubblegum.provider, [redeemIx], [payer], true);

    let [asset] = await PublicKey.findProgramAddress(
      [
        Buffer.from("asset"),
        merkleRollKeypair.publicKey.toBuffer(),
        leafNonce.toBuffer("le", 8),
      ],
      Bubblegum.programId
    );

    let [mintAuthority] = await PublicKey.findProgramAddress(
      [asset.toBuffer()],
      Bubblegum.programId
    );

    let decompressIx = createDecompressV1Instruction(
      {
        voucher: voucher,
        owner: payer.publicKey,
        tokenAccount: await Token.getAssociatedTokenAddress(
          ASSOCIATED_TOKEN_PROGRAM_ID,
          TOKEN_PROGRAM_ID,
          asset,
          payer.publicKey
        ),
        mint: asset,
        mintAuthority: mintAuthority,
        metadata: await getMetadata(asset),
        masterEdition: await getMasterEdition(asset),
        sysvarRent: SYSVAR_RENT_PUBKEY,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      },
      {
        metadata,
      }
    );
    await execute(Bubblegum.provider, [decompressIx], [payer], true);

    // Fetch the token metadata account and deserialize its data
    const metadataKey = await getMetadata(asset);
    const onChainNFTMetadataAccount = await connection.getAccountInfo(
      metadataKey,
      "confirmed"
    );
    const metadataForDecompressedNFT = metadataBeet.deserialize(
      onChainNFTMetadataAccount.data
    )[0];
    assertMetadataMatch(metadataForDecompressedNFT, metadata, mintAuthority);
  });
  it("Can mint and decompress with creators", async function () {
    const metadata: MetadataArgs = {
      name: "test",
      symbol: "test",
      uri: "www.solana.com",
      sellerFeeBasisPoints: 0,
      primarySaleHappened: false,
      isMutable: false,
      editionNonce: null,
      tokenStandard: null,
      tokenProgramVersion: TokenProgramVersion.Original,
      collection: null,
      uses: null,
      creators: [
        { address: Keypair.generate().publicKey, share: 20, verified: false },
        { address: Keypair.generate().publicKey, share: 20, verified: false },
        { address: Keypair.generate().publicKey, share: 20, verified: false },
        { address: Keypair.generate().publicKey, share: 40, verified: false },
      ],
    };

    console.log(" - Minting to tree");

    const mintIx = createMintV1Instruction(
      {
        mintAuthority: treeAuthority,
        mintAuthorityRequest: mintRequest,
        authority: treeAuthority,
        candyWrapper: CANDY_WRAPPER_PROGRAM_ID,
        gummyrollProgram: GummyrollProgramId,
        owner: payer.publicKey,
        delegate: payer.publicKey,
        merkleSlab: merkleRollKeypair.publicKey,
      },
      { message: metadata }
    );
    await execute(Bubblegum.provider, [mintIx], [], true);

    const dataHash = computeDataHash(metadata.sellerFeeBasisPoints, mintIx);
    const creatorHash = computeCreatorHash(metadata.creators);

    console.log(" - Decompressing leaf");

    let onChainRoot = await getRootOfOnChainMerkleRoot(
      connection,
      merkleRollKeypair.publicKey
    );

    let voucher = await getVoucherPDA(
      Bubblegum.provider.connection,
      merkleRollKeypair.publicKey,
      0
    );

    const nonceCount = await getNonceCount(
      Bubblegum.provider.connection,
      merkleRollKeypair.publicKey
    );
    const leafNonce = nonceCount.sub(new BN(1));

    let redeemIx = createRedeemInstruction(
      {
        authority: treeAuthority,
        owner: payer.publicKey,
        delegate: payer.publicKey,
        candyWrapper: CANDY_WRAPPER_PROGRAM_ID,
        gummyrollProgram: GummyrollProgramId,
        merkleSlab: merkleRollKeypair.publicKey,
        voucher: voucher,
      },
      {
        root: bufferToArray(onChainRoot),
        dataHash,
        creatorHash,
        nonce: leafNonce,
        index: 0,
      }
    );
    await execute(Bubblegum.provider, [redeemIx], [payer], true);

    let [asset] = await PublicKey.findProgramAddress(
      [
        Buffer.from("asset"),
        merkleRollKeypair.publicKey.toBuffer(),
        leafNonce.toBuffer("le", 8),
      ],
      Bubblegum.programId
    );

    let [mintAuthority] = await PublicKey.findProgramAddress(
      [asset.toBuffer()],
      Bubblegum.programId
    );

    let decompressIx = createDecompressV1Instruction(
      {
        voucher: voucher,
        owner: payer.publicKey,
        tokenAccount: await Token.getAssociatedTokenAddress(
          ASSOCIATED_TOKEN_PROGRAM_ID,
          TOKEN_PROGRAM_ID,
          asset,
          payer.publicKey
        ),
        mint: asset,
        mintAuthority: mintAuthority,
        metadata: await getMetadata(asset),
        masterEdition: await getMasterEdition(asset),
        sysvarRent: SYSVAR_RENT_PUBKEY,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      },
      {
        metadata,
      }
    );
    await execute(Bubblegum.provider, [decompressIx], [payer]);

    // Fetch the token metadata account and deserialize its data
    const onChainNFTMetadataAccount =
      await Bubblegum.provider.connection.getAccountInfo(
        await getMetadata(asset)
      );
    const metadataForDecompressedNFT = metadataBeet.deserialize(
      onChainNFTMetadataAccount.data
    )[0];
    assertMetadataMatch(metadataForDecompressedNFT, metadata, mintAuthority);
  });
  it("Mint to tree with delegate", async () => {
    console.log(" - Set tree delegate");
    const randomDelegate = Keypair.generate();
    await connection.requestAirdrop(randomDelegate.publicKey, LAMPORTS_PER_SOL);

    const setDelegateIx = createSetTreeDelegateInstruction({
      newDelegate: randomDelegate.publicKey,
      treeAuthority,
      merkleSlab: merkleRollKeypair.publicKey,
      creator: payer.publicKey,
    });
    await execute(Bubblegum.provider, [setDelegateIx], [payer], true);
    let expectedAuthorityState = TreeConfig.fromArgs({
      creator: payer.publicKey,
      delegate: randomDelegate.publicKey,
      totalMintCapacity: new BN(1 << MAX_DEPTH),
      numMintsApproved: new BN(1 << (MAX_DEPTH - 1)),
      numMinted: new BN(0),
    });
    await assertOnChainTreeAuthority(
      connection,
      expectedAuthorityState,
      treeAuthority
    );
  });

  it("Mint to tree with request", async () => {
    console.log(" - Create mint authority request for 1");
    const randomRequester = Keypair.generate();
    await connection.requestAirdrop(randomRequester.publicKey, 1e9);

    const requestPda = await getMintRequestPDA(
      merkleRollKeypair.publicKey,
      randomRequester.publicKey
    );
    const initRequestIx = createRequestMintAuthorityInstruction(
      {
        mintAuthority: randomRequester.publicKey,
        payer: randomRequester.publicKey,
        mintAuthorityRequest: requestPda,
        treeAuthority,
        merkleSlab: merkleRollKeypair.publicKey,
      },
      {
        mintCapacity: new BN(2),
      }
    );
    await execute(Bubblegum.provider, [initRequestIx], [randomRequester], true);
    let expectedMintRequestState = MintRequest.fromArgs({
      mintAuthority: randomRequester.publicKey,
      numMintsRequested: new BN(2),
      numMintsApproved: 0,
    });
    await assertOnChainMintRequest(
      connection,
      expectedMintRequestState,
      requestPda
    );
    let expectedAuthorityState = TreeConfig.fromArgs({
      creator: payer.publicKey,
      delegate: payer.publicKey,
      totalMintCapacity: new BN(1 << 20),
      numMintsApproved: new BN(1 << 19),
      numMinted: new BN(0),
    });
    await assertOnChainTreeAuthority(
      connection,
      expectedAuthorityState,
      treeAuthority
    );

    console.log(" - Approve mint authority request");
    const approveRequestIx = createApproveMintAuthorityRequestInstruction(
      {
        mintAuthorityRequest: requestPda,
        treeDelegate: payer.publicKey,
        merkleSlab: merkleRollKeypair.publicKey,
        treeAuthority,
      },
      {
        numMintsToApprove: new BN(2),
      }
    );
    await execute(Bubblegum.provider, [approveRequestIx], [payer], true);
    expectedMintRequestState = MintRequest.fromArgs({
      ...expectedMintRequestState,
      numMintsRequested: 0,
      numMintsApproved: new BN(2),
    });
    await assertOnChainMintRequest(
      connection,
      expectedMintRequestState,
      requestPda
    );
    expectedAuthorityState = TreeConfig.fromArgs({
      ...expectedAuthorityState,
      numMintsApproved: new BN((1 << (MAX_DEPTH - 1)) + 2),
    });
    await assertOnChainTreeAuthority(
      connection,
      expectedAuthorityState,
      treeAuthority
    );

    console.log(" - Mint with request");
    const metadata = {
      name: "test",
      symbol: "test",
      uri: "www.solana.com",
      sellerFeeBasisPoints: 0,
      primarySaleHappened: false,
      isMutable: false,
      editionNonce: null,
      tokenStandard: null,
      tokenProgramVersion: TokenProgramVersion.Original,
      collection: null,
      uses: null,
      creators: [],
    };
    console.log("About to mint")
    let mintWithRequestIx = createMintV1Instruction(
      {
        merkleSlab: merkleRollKeypair.publicKey,
        authority: treeAuthority,
        candyWrapper: CANDY_WRAPPER_PROGRAM_ID,
        gummyrollProgram: GummyrollProgramId,
        mintAuthority: randomRequester.publicKey,
        owner: randomRequester.publicKey,
        delegate: randomRequester.publicKey,
        mintAuthorityRequest: requestPda,
      },
      { message: metadata }
    );
    // Shakes fist at Anchor
    mintWithRequestIx.keys[0].isSigner = true;
    await execute(
      Bubblegum.provider,
      [mintWithRequestIx],
      [randomRequester],
      true,
      true,
    );
    console.log("Minted")
    expectedMintRequestState = MintRequest.fromArgs({
      ...expectedMintRequestState,
      numMintsApproved: new BN(1),
    });
    await assertOnChainMintRequest(
      connection,
      expectedMintRequestState,
      requestPda
    );
    expectedAuthorityState = TreeConfig.fromArgs({
      ...expectedAuthorityState,
      numMinted: new BN(1),
    });
    await assertOnChainTreeAuthority(
      connection,
      expectedAuthorityState,
      treeAuthority
    );

    console.log(" - Close mint authority request");
    const closeIx = createCloseMintRequestInstruction({
      mintAuthority: randomRequester.publicKey,
      treeAuthority,
      mintAuthorityRequest: requestPda,
      merkleSlab: merkleRollKeypair.publicKey,
    });
    await execute(Bubblegum.provider, [closeIx], [randomRequester], true);
    const requestInfo = await connection.getAccountInfo(requestPda);
    assert(
      !requestInfo,
      `Request info should have been closed, but is instead ${requestInfo}`
    );
    let numMintsApproved = new BN(expectedAuthorityState.numMintsApproved).toNumber();
    expectedAuthorityState = TreeConfig.fromArgs({
      ...expectedAuthorityState,
      numMintsApproved: numMintsApproved - 1
    });
    await assertOnChainTreeAuthority(
      connection,
      expectedAuthorityState,
      treeAuthority
    );
  });
});
