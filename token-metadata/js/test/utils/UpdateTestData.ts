import { PublicKey } from '@solana/web3.js';
import {
  TokenStandard,
  Data,
  Collection,
  Uses,
  CollectionDetails,
  DelegateState,
  ProgrammableConfigOpt,
  AuthorizationData,
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
  programmableConfigOpt: ProgrammableConfigOpt;
  delegateState: DelegateState;
  authorizationData: AuthorizationData;
  config: ProgrammableConfigOpt = { __kind: 'Unchanged' };

  constructor() {
    this.newUpdateAuthority = null;
    this.data = null;
    this.primarySaleHappened = null;
    this.isMutable = null;
    this.tokenStandard = null;
    this.collection = null;
    this.uses = null;
    this.collectionDetails = null;
    this.delegateState = null;
    this.authorizationData = null;
    this.programmableConfigOpt = this.config;
  }
}
