import fs from 'fs';
import path from 'path';
import { LOCALHOST } from 'test/utils';

const configPath = path.join(__dirname, '..', 'config', 'solana-validator.yml');

const config = `---
json_rpc_url: "${LOCALHOST}"
websocket_url: ""
commitment: confirmed
`;

export function prepareConfig() {
  fs.writeFileSync(configPath, config, 'utf8');
}
