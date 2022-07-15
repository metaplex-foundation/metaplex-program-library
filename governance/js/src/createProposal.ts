#! /usr/bin/env node
import {
  AccountMetaData,
  getGovernanceProgramVersion,
  getTokenOwnerRecordAddress,
  Governance,
  GovernanceAccountParser,
  InstructionData,
  Realm,
  VoteType,
  withAddSignatory,
  withCreateProposal,
  withInsertTransaction,
  withSignOffProposal,
} from '@solana/spl-governance';
import {
  Cluster,
  clusterApiUrl,
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
} from '@solana/web3.js';
import './borshFill';
import { createUpgradeInstruction } from './createUpgradeInstruction';

const DEFAULT_GOVERNANCE_PROGRAM_ID = 'GovER5Lthms3bLBqWub97yVrMmEogzX7xNjdXpPPCVZw';

async function run() {
  // If running locally, make sure to set environment vars - example in .env.dev.
  const programId = new PublicKey(process.env.PROGRAM_ID!);
  const governanceProgramId = new PublicKey(
    process.env.GOVERNANCE_PROGRAM_ID || DEFAULT_GOVERNANCE_PROGRAM_ID,
  );
  const bufferKey = new PublicKey(process.env.BUFFER!);
  const governanceKey = new PublicKey(process.env.GOVERNANCE_KEY!);
  const network = process.env.NETWORK!;
  const signatory = process.env.SIGNATORY && new PublicKey(process.env.SIGNATORY);
  const wallet = Keypair.fromSecretKey(
    Buffer.from(
      JSON.parse(
        require('fs').readFileSync(process.env.WALLET!, {
          encoding: 'utf-8',
        }),
      ),
    ),
  );
  const connection = new Connection(
    network.startsWith('http') ? network : clusterApiUrl(network as Cluster),
    {},
  );

  const tx = new Transaction();
  const instructions: TransactionInstruction[] = [];
  const info = await connection.getAccountInfo(governanceKey);
  const gov = GovernanceAccountParser(Governance)(governanceKey, info!).account;
  const realmKey = gov.realm;
  const realmInfo = await connection.getAccountInfo(realmKey);
  const realm = GovernanceAccountParser(Realm)(governanceKey, realmInfo!).account;
  PublicKey.prototype.toString = PublicKey.prototype.toBase58;

  // decide between community and council mint; default to community
  const governanceMint = process.env.GOVERNANCE_MINT
    ? process.env.GOVERNANCE_MINT === realm.communityMint.toBase58()
      ? (realm.communityMint as PublicKey)
      : (realm.config.councilMint as PublicKey)
    : (realm.communityMint as PublicKey);

  // todo: withCreateTokenOwnerRecord if DNE
  // todo: withDepositGoverningTokens if insufficient tokens deposited

  // Must have sufficient governance mint tokens deposited to create a proposal
  const tokenOwnerRecord = await getTokenOwnerRecordAddress(
    governanceProgramId,
    realmKey,
    governanceMint,
    wallet.publicKey,
  );

  const version = await getGovernanceProgramVersion(connection, governanceProgramId);

  // V2 Approve/Deny configuration
  const proposal = await withCreateProposal(
    instructions,
    governanceProgramId,
    version,
    realmKey,
    governanceKey,
    tokenOwnerRecord,
    process.env.NAME!,
    process.env.DESCRIPTION!,
    governanceMint,
    wallet.publicKey,
    gov.proposalCount,
    VoteType.SINGLE_CHOICE,
    ['Approve'],
    true,
    wallet.publicKey,
  );

  // If signatory provided, add it. Otherwise add ourselves and sign off immediately
  const signatoryRecord = await withAddSignatory(
    instructions,
    governanceProgramId,
    version,
    proposal,
    tokenOwnerRecord,
    wallet.publicKey,
    signatory ? signatory : wallet.publicKey,
    wallet.publicKey,
  );

  const upgradeIx = await createUpgradeInstruction(
    programId,
    bufferKey,
    governanceKey,
    wallet.publicKey,
  );

  await withInsertTransaction(
    instructions,
    governanceProgramId,
    version,
    governanceKey,
    proposal,
    tokenOwnerRecord,
    wallet.publicKey,
    0,
    0,
    0,
    [
      new InstructionData({
        programId: upgradeIx.programId,
        accounts: upgradeIx.keys.map((key) => new AccountMetaData(key)),
        data: upgradeIx.data,
      }),
    ],
    wallet.publicKey,
  );

  if (!signatory) {
    await withSignOffProposal(
      instructions,
      governanceProgramId,
      version,
      realmKey,
      governanceKey,
      proposal,
      wallet.publicKey,
      signatoryRecord,
      undefined,
    );
  }

  tx.add(...instructions);
  tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
  tx.sign(wallet);
  console.log(
    'TX signtaure: ',
    await connection.sendRawTransaction(tx.serialize(), { skipPreflight: true }),
  );
  console.log('Proposal: ', proposal.toBase58());
}

run().catch((e) => {
  console.error(e);
  console.error(e.stack);
  process.exit(1);
});
