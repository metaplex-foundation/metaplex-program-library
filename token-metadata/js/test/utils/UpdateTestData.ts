import { PublicKey } from '@solana/web3.js';
import {
  AuthorizationData,
  TokenStandard,
  Data,
  Collection,
  Uses,
  CollectionDetails,
  ProgrammableConfig,
  DelegateState,
} from '../../src/generated';

export type UpdateTestData = {
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
  authorizationData: AuthorizationData;
};
