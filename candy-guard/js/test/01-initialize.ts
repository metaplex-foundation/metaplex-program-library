import test from 'tape'
import spok from 'spok'
import { InitTransactions, killStuckProcess } from './setup/'
import { CandyGuard } from '../src/generated'
import { spokSameBignum, spokSamePubkey } from './utils'
import { BN } from 'bn.js'

const API = new InitTransactions()

killStuckProcess()

test('initialize: new candy guard (no guards)', async (t) => {
    const { fstTxHandler, payerPair, connection } = await API.payer();

    const data = {
        botTax: null,
        liveDate: null,
        lamports: null,
        splToken: null,
        thirdPartySigner: null,
        whitelist: null,
        gatekeeper: null,
        endSettings: null
    };

    const { tx: transaction, candyGuard: address } = await API.initialize(
        t,
        data,
        payerPair,
        fstTxHandler
    );
    // executes the transaction
    await transaction.assertSuccess(t);
    // retrieves the created candy machine
    const candyGuard = await CandyGuard.fromAccountAddress(connection, address);

    spok(t, candyGuard, {
        features: spokSameBignum(0),
        authority: spokSamePubkey(payerPair.publicKey),
    });
})

test('initialize: new candy guard (with guards)', async (t) => {
    const { fstTxHandler, payerPair, connection } = await API.payer();

    const data = {
        botTax: {
            lamports: new BN(100000000),
            lastInstruction: true
        },
        liveDate: {
            date: null
        },
        lamports: {
            amount: new BN(100000000)
        },
        splToken: null,
        thirdPartySigner: null,
        whitelist: null,
        gatekeeper: null,
        endSettings: null
    };

    const { tx: transaction, candyGuard: address } = await API.initialize(
        t,
        data,
        payerPair,
        fstTxHandler
    );
    // executes the transaction
    await transaction.assertSuccess(t);
    // retrieves the created candy machine
    const candyGuard = await CandyGuard.fromAccountAddress(connection, address);

    spok(t, candyGuard, {
        // bot_tax (b001) + live_date (b010) + lamports_charge (b100)
        features: spokSameBignum(7),
        authority: spokSamePubkey(payerPair.publicKey),
    });
})
