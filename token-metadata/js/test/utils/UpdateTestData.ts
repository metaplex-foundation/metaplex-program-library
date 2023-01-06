import { PublicKey } from '@solana/web3.js';
import {
  Data,
  AuthorizationData,
  CollectionToggle,
  UsesToggle,
  CollectionDetailsToggle,
  RuleSetToggle,
} from '../../src/generated';

export class UpdateTestData {
  newUpdateAuthority: PublicKey;
  data: Data;
  primarySaleHappened: boolean;
  isMutable: boolean;
  collection: CollectionToggle;
  uses: UsesToggle;
  collectionDetails: CollectionDetailsToggle;
  ruleSet: RuleSetToggle;
  authorizationData: AuthorizationData;

  constructor() {
    this.newUpdateAuthority = null;
    this.data = null;
    this.primarySaleHappened = null;
    this.isMutable = null;
    this.collection = { __kind: 'None' };
    this.uses = { __kind: 'None' };
    this.collectionDetails = { __kind: 'None' };
    this.authorizationData = null;
    this.ruleSet = { __kind: 'None' };
  }
}
