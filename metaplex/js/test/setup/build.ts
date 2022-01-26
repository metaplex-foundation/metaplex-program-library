import { execSync } from 'child_process';
import { rmSync, mkdirSync } from 'fs';
import { tmpTestDir } from '../utils';
import path from 'path';

async function build() {
  const programs: string[] = [
    'auction/program',
    'token-metadata/program',
    'token-vault/program',
    'metaplex/program',
  ];
  rmSync(tmpTestDir, { recursive: true, force: true });
  mkdirSync(tmpTestDir);

  const currentDir = process.cwd();

  programs.forEach((directory) => {
    const dir = path.resolve(currentDir, `../../${directory}`);
    process.chdir(dir);
    execSync(`cargo build-bpf`);
  });

  process.chdir(currentDir);
}

build();
