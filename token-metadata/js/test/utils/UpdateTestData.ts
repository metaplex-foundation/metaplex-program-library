import { PublicKey } from '@solana/web3.js';
import {
  TokenStandard,
  Data,
  Collection,
  Uses,
  CollectionDetails,
  ProgrammableConfig,
  DelegateState,
} from '../../src/generated';

export class UpdateTestData {
  newUpdateAuthority: PublicKey;
  data: Data;
  primarySaleHappened: boolean;
  isMutable: boolean;
  tokenStandard: TokenStandard;
  collection: Collection;
  uses: Uses;
  collectionDetails: CollectionDetails;
  programmableConfig: ProgrammableConfig;
  delegateState: DelegateState;

  constructor() {
    this.newUpdateAuthority = null;
    this.data = null;
    this.primarySaleHappened = null;
    this.isMutable = null;
    this.tokenStandard = null;
    this.collection = null;
    this.uses = null;
    this.collectionDetails = null;
    this.programmableConfig = null;
    this.delegateState = null;
  }
}
