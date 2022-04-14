# Sugar: A Candy Machine CLI

## Quick Start

Set up your Solana CLI config with an RPC url and a keypair:

```bash
solana config set --url <rpc_url> --keypair <path_to_keypair_file>
```

Sugar will then use these settings by default if you don't specify them as CLI options, allowing commands to be much simpler. 

Create a directory with a folder named "assets" where you store your json file and asset file pairs with the naming convention 0.json, 0.<ext>, 1.json, 1.<ext>, etc. where the extension is `.png`, `.json`, etc.

Set up your config file, `Bundlr` or `AWS` upload methods are currently supported:

```bash
sugar create-config
```

Check that your assets folder contains valid metadata:

```bash
sugar validate
```

Upload your assets to the selected storage:

```bash
sugar upload
```

Once the cache file is successfully populated, deploy your items to Solana:

```bash
sugar deploy
```

Verify that all items in your cache file have been successfully written on-chain:

```bash
sugar verify
```
