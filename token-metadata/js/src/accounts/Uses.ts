import {
    Borsh,
    Account,
    ERROR_INVALID_ACCOUNT_DATA,
    ERROR_INVALID_OWNER,
    AnyPublicKey,
} from '@metaplex-foundation/mpl-core';
import { AccountInfo, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import { Edition } from './Edition';
import { MetadataKey, MetadataProgram } from '../MetadataProgram';
import { Buffer } from 'buffer';

import { StringPublicKey } from "@metaplex-foundation/mpl-core";
import { UseMethod } from '.';

type Args = { useMethod: UseMethod; total: number, remaining: number };
export class Uses extends Borsh.Data<Args> {
    static readonly SCHEMA = Uses.struct([
        ['useMethod', 'u8'],
        ['total', 'u64'],
        ['remaining', 'u64'],
    ]);
    useMethod: UseMethod;
    /// Points at MasterEdition struct
    total: number;
    remaining: number;

    constructor(args: Args) {
        super(args);
        this.useMethod = args.useMethod
        this.total = args.total;
        this.remaining = args.remaining;
    }
}