// -----------------
// Example: Life of Vault, showing the different stages of a vault
// -----------------

// Make sure to have a local validator running to try this as is, i.e. via `yarn amman:start`

import { Connection, Keypair, sendAndConfirmTransaction, Transaction } from '@solana/web3.js';
import { airdrop, LOCALHOST, TokenBalances } from '@metaplex-foundation/amman';
import { strict as assert } from 'assert';

// These help us identify public keys, make sure to run with
// `DEBUG=vault:* ts-node ./examples/...` to log them
import { addressLabels } from '../test/utils';

import { fundedPayer } from './helpers';

import {
  Vault,
  ActivateVaultAccounts,
  activateVault,
  SetAuthorityInstructionAccounts,
  createSetAuthorityInstruction,
} from '../src/mpl-token-vault';
import { initVault } from './init-vault';
import { addTokenToVault } from './add-token-to-inactive-vault.single-transaction';
import { pdaForVault } from '../src/common/helpers';

// Could be devnet/mainnet, depending on your use case
const host = LOCALHOST;
async function main() {
  console.log('+++++++ Ex: life-of-vault.ts  +++++++');
  const connection = new Connection(host, 'confirmed');

  // This is the payer account funding the vault which should have sufficient amount of SOL
  const payerPair = await fundedPayer(connection);
  // Authority of the vault which controls it via token-vault instructions
  const vaultAuthorityPair = Keypair.generate();

  // -----------------
  // 1. Initialize the Vault
  //    follow the `initVault` call inside `./init-vault.ts` for more details
  // -----------------
  const initVaultAccounts = await initVault(
    connection,
    {
      payer: payerPair,
      vaultAuthority: vaultAuthorityPair,
      allowFurtherShareCreation: true,
    },
    addressLabels,
  );
  const { vault, authority: vaultAuthority, fractionMint, fractionTreasury } = initVaultAccounts;

  // -----------------
  // 2. While still inactive we can add tokens to the vault
  //    follow the `addTokenToVault` call inside `./add-token-to-inactive-vault.single-transaction.ts` for more details
  // -----------------
  await addTokenToVault(
    connection,
    payerPair,
    vaultAuthorityPair,
    initVaultAccounts.vault,
    addressLabels,
  );

  // -----------------
  // 3. Activate the vault which has the following consequences
  //
  // - no more tokens can be added to the vault
  // - unless we allowed this during vault initialization (we did) no more shares can be created
  // -----------------
  {
    const accounts: ActivateVaultAccounts = {
      vault,
      vaultAuthority,
      fractionMint,
      fractionTreasury,
    };

    const NUMBER_OF_SHARES = 10;
    const activateVaultIx = await activateVault(vault, accounts, NUMBER_OF_SHARES);
    const tx = new Transaction().add(activateVaultIx);
    const signers = [payerPair, vaultAuthorityPair];
    const sig = await sendAndConfirmTransaction(connection, tx, signers);

    // We can now verify that the NUMBER_OF_SHARES were transferred to the `fractionMintAuthority`
    // as part of the activate vault transaction
    const fractionMintAuthority = await pdaForVault(initVaultAccounts.vault);
    addressLabels.addLabels({ fractionMintAuthority });
    await TokenBalances.forTransaction(connection, sig, addressLabels).dump();
  }

  // -----------------
  // 4.Even though the vault is active we can still change the vault authority
  // -----------------
  {
    const [newAuthority] = addressLabels.genKeypair('newAuthority');
    await airdrop(connection, newAuthority, 1);

    const accounts: SetAuthorityInstructionAccounts = {
      vault,
      currentAuthority: vaultAuthority,
      newAuthority,
    };
    const setAuthorityIx = createSetAuthorityInstruction(accounts);
    const tx = new Transaction().add(setAuthorityIx);

    await sendAndConfirmTransaction(connection, tx, [payerPair, vaultAuthorityPair]);

    // We can now verify that the vault authority was indeed updated
    const vaultAccountInfo = await connection.getAccountInfo(vault);
    assert(vaultAccountInfo != null);
    const [vaultAccount] = Vault.fromAccountInfo(vaultAccountInfo);
    console.log({ vaultWithUpdatedAuthority: vaultAccount.pretty() });

    // Let's verify that the authority was changed as we expect
    assert(vaultAccount.authority.equals(newAuthority));
  }
}

if (module === require.main) {
  main()
    .then(() => process.exit(0))
    .catch((err: any) => {
      console.error(err);
      process.exit(1);
    });
}
