import * as React from "react";

import init, {
  elgamal_keypair_from_signature,
  elgamal_decrypt_u32,
} from '../utils/privateMetadata/private_metadata_js';

export interface WasmConfig {
  elgamalKeypairFromSignature: (signature: any) => any;
  elgamalDecryptU32: (elgamalKeypair: any, ciphertext: any) => any;
}

const WasmContext = React.createContext<WasmConfig | undefined>(undefined);

export function WasmProvider({ children }: { children: any }) {
  const [contextValue, setContextValue] = React.useState<WasmConfig | null>(null);

  React.useEffect(() => {
    const wrap = async () => {
      // TODO: figure out why reading functions of output don't work here...
      const bindings = await init();
      setContextValue({
        elgamalKeypairFromSignature: elgamal_keypair_from_signature,
        elgamalDecryptU32: elgamal_decrypt_u32,
      });
    };
    wrap();
  }, []);

  return (
    <WasmContext.Provider value={contextValue}>
      {children}
    </WasmContext.Provider>
  );
}

export function useWasmConfig() {
  const context = React.useContext(WasmContext);
  if (context === undefined) {
    throw new Error('WasmContext must be used with a WasmProvider');
  }
  return context;
}
