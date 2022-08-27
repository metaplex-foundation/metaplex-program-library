import test from 'tape'
import { InitTransactions, killStuckProcess } from './setup/'

const init = new InitTransactions()

killStuckProcess()

test('mint (authority)', async (t) => {
    const { fstTxHandler, payerPair, connection } = await init.payer();
    const items = 10;

    const data = {
        itemsAvailable: items,
        symbol: "CORE",
        sellerFeeBasisPoints: 500,
        maxSupply: 0,
        isMutable: true,
        retainAuthority: true,
        creators: [{
            address: payerPair.publicKey,
            verified: false,
            percentageShare: 100
        }],
        configLineSettings: {
            prefixName: "TEST ",
            nameLength: 10,
            prefixUri: "https://arweave.net/",
            uriLength: 50,
            isSequential: false
        },
        hiddenSettings: null
    };

    const address = await init.createCandyMachine(t, payerPair, data, fstTxHandler, connection);
    const lines = [];

    for (let i = 0; i < items; i++) {
        const line = {
            name: `NFT #${i + 1}`,
            uri: "uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ"
        };

        lines[i] = line;
    }

    await init.addConfigLines(t, address, payerPair, lines, fstTxHandler);

    await init.mintFromCandyMachine(
        t,
        address,
        payerPair,
        fstTxHandler,
        connection
    );
})

test('mint (minter)', async (t) => {
    const { fstTxHandler, payerPair, connection } = await init.payer();
    const items = 10;

    const data = {
        itemsAvailable: items,
        symbol: "CORE",
        sellerFeeBasisPoints: 500,
        maxSupply: 0,
        isMutable: true,
        retainAuthority: true,
        creators: [{
            address: payerPair.publicKey,
            verified: false,
            percentageShare: 100
        }],
        configLineSettings: {
            prefixName: "TEST ",
            nameLength: 10,
            prefixUri: "https://arweave.net/",
            uriLength: 50,
            isSequential: false
        },
        hiddenSettings: null
    };

    const address = await init.createCandyMachine(t, payerPair, data, fstTxHandler, connection);
    const lines = [];

    for (let i = 0; i < items; i++) {
        const line = {
            name: `NFT #${i + 1}`,
            uri: "uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ"
        };

        lines[i] = line;
    }

    await init.addConfigLines(t, address, payerPair, lines, fstTxHandler);

    // keypair of the minter
    const { fstTxHandler: minterHandler, minterPair, connection: minterConnection } = await init.minter();

    try {
        await init.mintFromCandyMachine(
            t,
            address,
            minterPair,
            minterHandler,
            minterConnection
        );
        t.fail('only authority is allowed to mint');
    } catch {
        // we are expecting an error
        t.ok(minterPair, 'minter is not the candy machine authority');
    }
})
