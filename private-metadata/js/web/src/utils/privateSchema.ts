import { BinaryReader, BinaryWriter, deserializeUnchecked } from 'borsh';
import base58 from 'bs58';

export enum PrivateMetadataKey {
    Uninitialized = 0,
    PrivateMetadataAccountV1 = 1,
    CipherKeyTransferBufferV1  = 2,
}

type StringPublicKey = string;
type ElgamalPk = Uint8Array;
type ElgamalCipherText = Uint8Array;

export class PrivateMetadataAccount {
    key: PrivateMetadataKey;

    /// The corresponding SPL Token Mint
    mint: StringPublicKey;

    /// The public key associated with ElGamal encryption
    elgamalPk: ElgamalPk;

    /// 192-bit AES cipher key encrypted with elgamal_pk
    /// ElGamalCiphertext encrypted 4-byte chunks so 6 chunks total
    encryptedCipherKey: ElgamalCipherText;

    /// URI of encrypted asset
    uri: string;

    constructor(args: {
      mint: StringPublicKey,
      elgamalPk: ElgamalPk,
      encryptedCipherKey: ElgamalCipherText,
      uri: string,
    }) {
      this.key = PrivateMetadataKey.PrivateMetadataAccountV1;
      this.mint = args.mint;
      this.elgamalPk = args.elgamalPk;
      this.encryptedCipherKey = args.encryptedCipherKey;
      this.uri = args.uri;
    }
}

export class CipherKeyTransferBuffer {
    key: PrivateMetadataKey;

    /// Bit mask of updated chunks
    updated: number;

    /// Source pubkey. Should match the currently encrypted elgamal_pk
    authority: StringPublicKey;

    /// Account that will have its encrypted key updated
    privateMetadataKey: StringPublicKey;

    /// Destination public key
    elgamalPk: ElgamalPk;

    /// 192-bit AES cipher key encrypted with elgamal_pk
    encryptedCipherKey: ElgamalCipherText;

    constructor(args: {
      updated: number,
      authority: StringPublicKey,
      privateMetadataKey: StringPublicKey,
      elgamalPk: ElgamalPk,
      encryptedCipherKey: ElgamalCipherText,
    }) {
      this.key = PrivateMetadataKey.CipherKeyTransferBufferV1;
      this.updated = args.updated;
      this.authority = args.authority;
      this.privateMetadataKey = args.privateMetadataKey;
      this.elgamalPk = args.elgamalPk;
      this.encryptedCipherKey = args.encryptedCipherKey;
    }
}

export const PRIVATE_METADATA_SCHEMA = new Map<any, any>([
  [
    PrivateMetadataAccount,
    {
      kind: 'struct',
      fields: [
        ['key', 'u8'],
        ['mint', 'pubkeyAsString'],
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
        ['authority', 'pubkeyAsString'],
        ['privateMetadataKey', 'pubkeyAsString'],
        ['elgamalPk', 'elgamalPk'],
        ['encryptedCipherKey', 'encryptedCipherKey'],
      ],
    },
  ],
]);

const PRIVATE_METADATA_REPLACE = new RegExp('\u0000', 'g');

export const decodePrivateMetadata = (
  buffer: Buffer
): PrivateMetadataAccount => {
  const ret = deserializeUnchecked(
    PRIVATE_METADATA_SCHEMA,
    PrivateMetadataAccount,
    buffer,
  ) as PrivateMetadataAccount;
  ret.uri = ret.uri.replace(PRIVATE_METADATA_REPLACE, '');
  return ret;
};

export const decodeTransferBuffer = (
  buffer: Buffer
): CipherKeyTransferBuffer => {
  const ret = deserializeUnchecked(
    PRIVATE_METADATA_SCHEMA,
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
