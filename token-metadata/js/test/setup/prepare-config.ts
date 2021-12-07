import fs from 'fs';
import { LOCALHOST } from '../utils';
import { solanaConfigPath } from './setup-utils';

const config = `---
json_rpc_url: "${LOCALHOST}"
websocket_url: ""
commitment: confirmed
`;

export function prepareConfig() {
  fs.writeFileSync(solanaConfigPath, config, 'utf8');
}
