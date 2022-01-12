import { BinaryReader, BinaryWriter, deserializeUnchecked } from 'borsh';
import base58 from 'bs58';

export enum StealthKey {
    Uninitialized = 0,
    StealthAccountV1 = 1,
    CipherKeyTransferBufferV1  = 2,
}

type StringPublicKey = string;
type ElgamalPk = Uint8Array;
type ElgamalCipherText = Uint8Array;

export class StealthAccount {
    key: StealthKey;

    /// The corresponding SPL Token Mint
    mint: StringPublicKey;

    /// The signing key associated with `elgamal_pk`
    walletPk: StringPublicKey;

    /// The public key associated with ElGamal encryption
    elgamalPk: ElgamalPk;

    /// 192-bit AES cipher key encrypted with elgamal_pk
    /// ElGamalCiphertext encrypted 4-byte chunks so 6 chunks total
    encryptedCipherKey: ElgamalCipherText;

    /// URI of encrypted asset
    uri: string;

    constructor(args: {
      mint: StringPublicKey,
      walletPk: StringPublicKey,
      elgamalPk: ElgamalPk,
      encryptedCipherKey: ElgamalCipherText,
      uri: string,
    }) {
      this.key = StealthKey.StealthAccountV1;
      this.mint = args.mint;
      this.walletPk = args.walletPk;
      this.elgamalPk = args.elgamalPk;
      this.encryptedCipherKey = args.encryptedCipherKey;
      this.uri = args.uri;
    }
}

export class CipherKeyTransferBuffer {
    key: StealthKey;

    /// Bit mask of updated chunks
    updated: number;

    /// Account that will have its encrypted key updated
    stealthKey: StringPublicKey;

    /// The destination signing key associated with `elgamal_pk`
    walletPk: StringPublicKey;

    /// Destination public key
    elgamalPk: ElgamalPk;

    /// 192-bit AES cipher key encrypted with elgamal_pk
    encryptedCipherKey: ElgamalCipherText;

    constructor(args: {
      updated: number,
      stealthKey: StringPublicKey,
      walletPk: StringPublicKey,
      elgamalPk: ElgamalPk,
      encryptedCipherKey: ElgamalCipherText,
    }) {
      this.key = StealthKey.CipherKeyTransferBufferV1;
      this.updated = args.updated;
      this.stealthKey = args.stealthKey;
      this.walletPk = args.walletPk;
      this.elgamalPk = args.elgamalPk;
      this.encryptedCipherKey = args.encryptedCipherKey;
    }
}

export const STEALTH_SCHEMA = new Map<any, any>([
  [
    StealthAccount,
    {
      kind: 'struct',
      fields: [
        ['key', 'u8'],
        ['mint', 'pubkeyAsString'],
        ['walletPk', 'pubkeyAsString'],
        ['elgamalPk', 'elgamalPk'],
        ['encryptedCipherKey', 'encryptedCipherKey'],
        ['uri', 'uriString'],
      ],
    },
  ],
  [
    CipherKeyTransferBuffer,
    {
      kind: 'struct',
      fields: [
        ['key', 'u8'],
        ['updated', 'u8'],
        ['stealthKey', 'pubkeyAsString'],
        ['walletPk', 'pubkeyAsString'],
        ['elgamalPk', 'elgamalPk'],
        ['encryptedCipherKey', 'encryptedCipherKey'],
      ],
    },
  ],
]);

const STEALTH_REPLACE = new RegExp('\u0000', 'g');

export const decodeStealth = (
  buffer: Buffer
): StealthAccount => {
  const ret = deserializeUnchecked(
    STEALTH_SCHEMA,
    StealthAccount,
    buffer,
  ) as StealthAccount;
  ret.uri = ret.uri.replace(STEALTH_REPLACE, '');
  return ret;
};

export const decodeTransferBuffer = (
  buffer: Buffer
): CipherKeyTransferBuffer => {
  const ret = deserializeUnchecked(
    STEALTH_SCHEMA,
    CipherKeyTransferBuffer,
    buffer,
  ) as CipherKeyTransferBuffer;
  return ret;
};

export const extendBorsh = () => {
  (BinaryReader.prototype as any).readElgamalPk = function () {
    const reader = this as unknown as BinaryReader;
    return reader.readFixedArray(32);
  };

  (BinaryWriter.prototype as any).writeElgamalPk = function (
    value: ElgamalPk,
  ) {
    const writer = this as unknown as BinaryWriter;
    writer.writeFixedArray(value);
  };

  (BinaryReader.prototype as any).readEncryptedCipherKey = function () {
    const reader = this as unknown as BinaryReader;
    return reader.readFixedArray(64);
  };

  (BinaryWriter.prototype as any).writeEncryptedCipherKey = function (
    value: ElgamalCipherText,
  ) {
    const writer = this as unknown as BinaryWriter;
    writer.writeFixedArray(value);
  };

  (BinaryReader.prototype as any).readPubkeyAsString = function () {
    const reader = this as unknown as BinaryReader;
    const array = reader.readFixedArray(32);
    return base58.encode(array) as StringPublicKey;
  };

  (BinaryWriter.prototype as any).writePubkeyAsString = function (
    value: StringPublicKey,
  ) {
    const writer = this as unknown as BinaryWriter;
    writer.writeFixedArray(base58.decode(value));
  };

  (BinaryReader.prototype as any).readUriString= function () {
    const reader = this as unknown as BinaryReader;
    const array = reader.readFixedArray(100);
    return Buffer.from(array).toString();
  };
};

extendBorsh();
