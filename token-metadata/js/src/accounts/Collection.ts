import {
    Borsh,
    StringPublicKey,
} from '@metaplex-foundation/mpl-core';

type Args = { key: StringPublicKey; verified: boolean };
export class Collection extends Borsh.Data<Args> {
    static readonly SCHEMA = Collection.struct([
        ['key', 'pubkeyAsString'],
        ['verified', 'u8'],
    ]);
    key: StringPublicKey;
    verified: boolean;

    constructor(args: Args) {
        super(args);
        this.key = args.key
        this.verified = args.verified;
    }
}
