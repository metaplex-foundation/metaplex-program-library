
// Run node ./ser_dev_addr.js - to set keypairs and pubkeys for local development.
// Run node ./ser_dev_addr.js reset - to reset pubkeys before git add .

const fs = require("fs/promises");
const util = require('util');
const exec = util.promisify(require('child_process').exec);
const args = process.argv.slice(2);

const PROGRAM_ROOT = process.env.PROGRAM_ROOT; // "/sol/metaplex/program-library";
const ENV_SETUP_PATH = `${PROGRAM_ROOT}/utils/env_setup`
const KEYRING_FILE = `${ENV_SETUP_PATH}/default_keyring.json`;

// Open and parse default keys JSON
const processAndGetKeyring = async () => {
  let keyring_data = await fs.readFile(KEYRING_FILE);
  const keyring = JSON.parse(keyring_data);

  // console.log(keyring);

  // Set up new keypair.json files, if not set.
  // Do not generate new keys on reset
  if (args[0] !== 'reset') {
    for ( const k in keyring ) {
      keypairPath = PROGRAM_ROOT + keyring[k].keypairPath;
      const { stdout, stderr } = await exec(`${ENV_SETUP_PATH}/helpers/gen-keypair.sh ${keypairPath}`);
      if (stderr) {
        console.log("Warning: ", stderr.trim());
      }
      console.log(stdout.trim());
      keyring[k]["devpubkey"] = stdout.trim();

    }
    // update and save keyring data JSON file
    keyring_data = JSON.stringify(keyring, null, 2);
    fs.writeFile(KEYRING_FILE, keyring_data);
  }
  return keyring;
}

const replacePubkeys = async ( keyring, srch_addr, repl_addr ) => {
  for (const k in keyring) {
    let { stdout, stderr } = await exec(`grep -rl ${keyring[k][srch_addr]} ${PROGRAM_ROOT}/.`);
    if (!!stderr) {
      throw Error("Error on grep");
    }
    stdout = stdout.trim();
    if (!stdout) {
      return;
    }
    const files = stdout.split(/\r?\n/);
    //console.log(lines);
    for ( const file of files ) {
      if ( 
        file.search("default_keyring.json") !== -1 
        || file.search('restore_program_ids.sh') !== -1
        ){
        console.log(`Do not change: ${file}`);
        continue;
      }
      console.log(file);
      let { stdout, stderr } = await exec(
        `sed -i "s/${keyring[k][srch_addr]}/${keyring[k][repl_addr]}/gm" ${file}`
      );
    }
  }
}

( async () => {
  const keyring = await processAndGetKeyring();
  let srch_addr = "pubkey";
  let repl_addr = "devpubkey";
  
  if (args[0] === 'reset') {
    await replacePubkeys( keyring, repl_addr, srch_addr );
    await exec(`cat ${ENV_SETUP_PATH}/default_keyring.json.backup > ${ENV_SETUP_PATH}/default_keyring.json`);
  } else {
    await replacePubkeys( keyring, srch_addr, repl_addr );
  }
} )();