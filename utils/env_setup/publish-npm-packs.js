const util = require('util');
const exec = util.promisify(require('child_process').exec);

const PROGRAM_ROOT = process.env.PROGRAM_ROOT; // "/sol/metaplex/program-library";
const ENV_SETUP_PATH = `${PROGRAM_ROOT}/utils/env_setup`

const PACK_DIRS = [
  "core/js",
  "auction/js",
  "auction-house/js",
  "candy-machine/js",
  "token-metadata/js",
  "token-vault/js",
  "token-entangler/js",
  "gumdrop/js",
  "fixed-price-sale/js",
  "metaplex/js"
];

(async () => {
  for ( const pack of PACK_DIRS ) {
    const js_dir = `${PROGRAM_ROOT}/${pack}`;
    //console.log(js_dir);
    const {stdout, stderr} = await exec(`/bin/bash ${ENV_SETUP_PATH}/helpers/push-js-packs.sh ${js_dir}`);
    console.log(stdout.trim());
  }
})();
