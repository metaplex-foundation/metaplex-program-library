import { expose } from 'comlink';
import { WasmConfig } from '../../contexts/WasmContext';

import init, {
  elgamal_decrypt_u32,
} from '../privateMetadata/private_metadata_js';

const decrypt = async (
  elgamalKeypair: any,
  chunk: Uint8Array,
) => {
  return new Promise(async resolve => {
    // can't pass methods to webworker through the `postMessage` stuff so
    // re-init the wasm...
    await init();
    resolve(elgamal_decrypt_u32(
      elgamalKeypair,
      { bytes: [...chunk] },
    ));
  });
}

const exports = {
  decrypt,
};

export type DecryptWorker = typeof exports;

expose(exports);
