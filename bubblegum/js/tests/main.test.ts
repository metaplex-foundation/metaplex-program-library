import {
    Connection,
    Keypair,
    LAMPORTS_PER_SOL,
    PublicKey,
    sendAndConfirmTransaction,
    SystemProgram,
    Transaction
} from '@solana/web3.js';

import {
    createCreateTreeInstruction,
    createMintV1Instruction,
    PROGRAM_ID as BUBBLEGUM_PROGRAM_ID,
    TokenProgramVersion,
    TokenStandard,
} from '../src/generated'

function keypairFromSeed(seed: string) {
    const expandedSeed = Uint8Array.from(Buffer.from(`${seed}                                           `));
    return Keypair.fromSeed(expandedSeed.slice(0, 32));
}

const LOG_WRAPPER_ID = new PublicKey("noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV");
const COMPRESSION_PROGRAM_ID = new PublicKey("cmtDvXumGCrqC1Age74AVPhSRVXJMd8PJS91L8KbNCK");

describe("Bubblegum tests", () => {
    const connection = new Connection("http://localhost:8899");
    const payerKeypair = keypairFromSeed("metaplex-test");
    const payer = payerKeypair.publicKey;

    beforeEach(async () => {
        await connection.requestAirdrop(payer, LAMPORTS_PER_SOL);
    })
    it("Can create a Bubblegum tree and mint to it", async () => {
        const merkleTreeKeypair = Keypair.generate();
        const merkleTree = merkleTreeKeypair.publicKey;

        // Hard code space until @solana/spl-account-compression released
        const space = 31800;
        const allocTreeIx = SystemProgram.createAccount({
            fromPubkey: payer,
            newAccountPubkey: merkleTree,
            lamports: await connection.getMinimumBalanceForRentExemption(space),
            space: space,
            programId: COMPRESSION_PROGRAM_ID,
        })
        const [treeAuthority, _bump] = await PublicKey.findProgramAddress([merkleTree.toBuffer()], BUBBLEGUM_PROGRAM_ID);
        const createTreeIx = createCreateTreeInstruction(
            {
                merkleTree,
                treeAuthority,
                treeCreator: payer,
                payer,
                logWrapper: LOG_WRAPPER_ID,
                compressionProgram: COMPRESSION_PROGRAM_ID,
            },
            {
                maxBufferSize: 64,
                maxDepth: 14
            },
            BUBBLEGUM_PROGRAM_ID,
        );

        const mintIx = createMintV1Instruction(
            {
                merkleTree,
                treeAuthority,
                treeDelegate: payer,
                payer,
                leafDelegate: payer,
                leafOwner: payer,
                compressionProgram: COMPRESSION_PROGRAM_ID,
                logWrapper: LOG_WRAPPER_ID,
            },
            {
                message: {
                    name: "Test Compressed NFT",
                    symbol: "TST",
                    uri: "https://metaplex.com",
                    creators: [],
                    editionNonce: 0,
                    tokenProgramVersion: TokenProgramVersion.Original,
                    tokenStandard: TokenStandard.Fungible,
                    uses: null,
                    collection: null,
                    primarySaleHappened: false,
                    sellerFeeBasisPoints: 0,
                    isMutable: false,
                }
            }
        );

        let tx = new Transaction().add(allocTreeIx).add(createTreeIx).add(mintIx);
        tx.feePayer = payer;
        let txId = await sendAndConfirmTransaction(connection, tx, [merkleTreeKeypair, payerKeypair], {
            commitment: "confirmed",
            skipPreflight: false
        })
        console.log(`Tx id: ${txId}`);
    })
})