# Sugar: A Candy Machine CLI

Sugar is the next iteration of the Metaplex Candy Machine CLI. It has been written from the ground up and includes several improvements:

- better peformace for upload of media/metadata files and deploy of the candy machine &mdash; these operations take advantage of multi-threaded systems to significantly speed up the computational time needed;
- simplified build and installation procedures taking advantage of `cargo` package management, including a binary distributable package ready to use;
- robust error handling and validation of inputs, including improvements to config and cache files, leading to more informative error messages.

> **Note:** This is an alpha release of Sugar. Use at your own risk. The current version supports only systems running macOS, Linux, or another Unix-like OS.

## Installation

### Binaries

Binaries for the supported OS can be found at:
- [Sugar Releases](https://github.com/metaplex-foundation/sugar/releases)

To use one of the binaries, download the version for your OS and unzip the binary. Then copy to a folder in your file system (preferable a folder in your `PATH` environment variable). If you have Rust installed we recommend putting it in `~/.cargo/bin`, otherwise `/usr/local/bin` is a good place for it on Linux and macOS. Once the binary is at that location your OS will find it automatically and you will be able to run the `sugar` binary from any directory in your file system as a normal command line application.
It is recommended to rename the downloaded binary (e.g., `sugar-ubuntu-latest` or `sugar-macos-latest`) to `sugar` for simplicitly &mdash; the remainder of this guide assumes that the binary is called `sugar`.

### Build From Source

In order to build Sugar from the source code, you will need to have [Rust](https://www.rust-lang.org/tools/install) installed in your system. It is recommended to install Rust using `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After the installation completes, running:

```bash
rustc --version
```

should print the version of the Rust compiler. If the command fails, check if the `~/.cargo/bin` directory is in your `PATH` environment variable.

The next step is to clone Sugar repository:

```bash
git clone https://github.com/metaplex-foundation/sugar.git
```

This will create a directory `sugar` with the lastest code from the repository. Switch to the newly created directory:

```bash
cd sugar
```

Then, you can build and install the binary to `~/.cargo/bin`:

```bash
cargo install --locked --path ./
```

As long as `./cargo/bin` is in your `PATH` environment variable, you will be able to execute `sugar` from any directory in your file system.

> **Note:** You need to execute `cargo install` from Sugar souce code root directory &mdash; the directory where the `Cargo.toml` is located.

## Quick Start

Set up your Solana CLI config with an RPC url and a keypair:

```bash
solana config set --url <rpc url> --keypair <path to keypair file>
```

Sugar will then use these settings by default if you don't specify them as CLI options, allowing commands to be much simpler. If you need help setting up Solana CLI and creating a `devnet` wallet, check the [Candy Machine v2 documentation](http://docs.metaplex.com/candy-machine-v2/getting-started#solana-wallet).

Create a folder named `assets` to store your json and media file pairs with the naming convention 0.json, 0.<ext>, 1.json, 1.<ext>, etc., where the extension is `.png`, `.jpg`, etc. This is the same format described in the [Candy Machine v2 documentation](http://docs.metaplex.com/candy-machine-v2/preparing-assets).

You can then use the `launch` command to start an interative process to create your config file and deploy a Candy Machine to Solana:

```bash
sugar launch
```

At the end of the execution of the `launch` command, the Candy Machine will be deployed on-chain.

## Working with Sugar

Sugar contains a collection of commands for creating and managing Metaplex Candy Machines. The complete list of commands can be viewed by running:

```bash
sugar
```

This will display a list of commands and their short description:

```bash
USAGE:
    sugar [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help                     Print help information
    -l, --log-level <LOG_LEVEL>    Log level: trace, debug, info, warn, error, off
    -V, --version                  Print version information

SUBCOMMANDS:
    create-config    Interactive process to create the config file
    deploy           Deploy cache items into candy machine config on-chain
    help             Print this message or the help of the given subcommand(s)
    launch           Create a candy machine deployment from assets
    mint             Mint one NFT from candy machine
    show             Show the on-chain config of an existing candy machine
    update           Update the candy machine config on-chain
    upload           Upload assets to storage and creates the cache config
    validate         Validate JSON metadata files
    verify           Verify uploaded data
    withdraw         Withdraw funds from candy machine account closing it
```

To get more information about a particular command (e.g., `deploy`), use the `help` command:

```bash
sugar help deploy
```

The list of options together with a short description will be displayed:
 
```bash
Deploy cache items into candy machine config on-chain

USAGE:
    sugar deploy [OPTIONS]

OPTIONS:
    -c, --config <CONFIG>          Path to the config file, defaults to "config.json" [default:
                                   config.json]
        --cache <CACHE>            Path to the cache file, defaults to "cache.json" [default:
                                   cache.json]
    -h, --help                     Print help information
    -k, --keypair <KEYPAIR>        Path to the keypair file, uses Sol config or defaults to
                                   "~/.config/solana/id.json"
    -l, --log-level <LOG_LEVEL>    Log level: trace, debug, info, warn, error, off
    -r, --rpc-url <RPC_URL>        RPC Url
```

> **Note:** This guide assumes that you set up your RPC url and a keypair using Solana CLI config, as described in the `Quick Start` section above.

### Configuration

Sugar uses a JSON configuration file to deploy and interact with a Candy Machine. The configuration file is largely similar to the [existing Candy Machine v2 configuration file](http://docs.metaplex.com/candy-machine-v2/configuration), but there are important differences.

A minimum configuration file looks like this:

```json
{
    "price": 1.0,
    "number": 10,
    "symbol": "SR",
    "sellerFeeBasisPoints": 500,
    "gatekeeper": null,
    "solTreasuryAccount": "<TREASURY WALLET ADDRESS>",
    "splTokenAccount": null,
    "splToken": null,
    "goLiveDate": "2022-04-22T00:00:00Z",
    "endSettings": null,
    "whitelistMintSettings": null,
    "hiddenSettings": null,
    "uploadMethod": "bundlr",
    "awsS3Bucket": null,
    "retainAuthority": true,
    "isMutable": true,
    "creators": [
    {
      "address": "<CREATOR 1 WALLET ADDRESS>",
      "share": 50
    },
    {
      "address": "<CREATOR n WALLET ADDRESS>",
      "share": 50
    }
  ]
}
```

The main differences with the previous configuration file are:
- **goLiveDate**: this needs to be specified using [RFC 3339 standard](https://datatracker.ietf.org/doc/html/rfc3339). In most cases, the format used will be "yyyy-mm-dd`T`hh:mm:ss`Z`", where `T` is the separator between the *full-date* and *full-time* and `Z` is the timezone offset from UTC (use `Z` or `+00:00` for UTC time);
- **retainAuthority**: this is similar to the previous *noRetainAuthority* property, but provides a clearer meaning&mdash;you should specify **true** to indicate that the candy machine retains the update authority for each mint (most common case) or **false** to transfer the authority to the minter;
- **isMutable**: this is similar to the previous *noMutable* property, but provides a clearer meaning&mdash;you should specify **yes** to indicate that the metadata is mutable (most common case) or **no** to prevent updates to the metadata;
- **creators**: specifies the list of creators and their percentage share of the royalties&mdash; at least one creator must be specified (up to a maximum of four) and the sum of shares must add up to `100`. This information used to be located on each metadata file, but has been deprecated since Token Metadata Standard v1.1.0 and therefore needs to be specfied in the configuration file. The list of creators will be the same to all NFTs minted from the Candy Machine.

#### Upload Methods

There are currently two upload (storage) methods available in Sugar: `"bundlr"` and `"aws"`.

##### Bundlr

Uploads to [Arweave](https://www.arweave.org/) using [Bundlr Network](https://bundlr.network/) and payments are made in `SOL`.

> **Note:** Files are only stored for 7 days when uploaded with Bundlr on `devnet`.

##### Amazon (AWS) S3

Uploads files to Amazon S3 storage. When using the `"aws"`, you need to specify the bucket name `"awsS3Bucket"` in the configuration file and set up the credentials in your system. In most cases, this will involve creating a file `~/.aws/credentials` with the following properties:

```bash
[default]
aws_access_key_id=<ACCESS KEY ID>
aws_secret_access_key=<SECRET ACCESS KEY>
region=<REGION>
```

It is also important to set up the ACL permission of the bucket correctly to enable `"public-read"` and apply Cross-Origin Resource Sharing (CORS) rules to enable content access requested from a different origin (necessary to enable wallets and blockchain explorers load the metadata/media files). More information about these configurations can be found at:
- [Bucket policy examples](https://docs.aws.amazon.com/AmazonS3/latest/userguide/example-bucket-policies.html)
- [CORS configuration](https://aws.amazon.com/premiumsupport/knowledge-center/s3-configure-cors/)

### Preparing Your Assets

The preparation of the assets is similar to the instructions provided in the [Candy Machine v2 documentation](http://docs.metaplex.com/candy-machine-v2/preparing-assets). By default, Sugar loads media/metadata files from an `assets` folder in the directory where the command has been executed, but the name of the folder can be specified as a command-line parameter.

### Deploying a Candy Machine

Apart from the `launch` command, discussed in the `Quick Start` section above, Sugar provide commands to manage the whole process of deployment of a Candy Machine, from the validation of assets to withdrawing funds and closing a Candy Machine account.

In this section we will cover the commands involved in deploying a Candy Machine in the oder that they should be executed.

#### 1. `create-config`

By default, Sugar looks for a `config.json` file in the current directory to load the Candy Machine configuration &mdash; the configuration file name can be specified with a `-c` or `--config` option.

You can either create this file manually, following the instructions above, or use the `create-config` command:

```bash
sugar create-config
```

Executing the command starts an interative process consisting in a sequence of prompts to gather information about all configuration options. At the end of it, a configuration file is saved (default to `config.json`) or its content is displayed on screen. To specify a custom file name, use the option `-c`:

```bash
sugar create-config -c my-config.json
```

#### 2. `validate`

The `validate` command is used to check that all files in the assets folder are in the correct format:

```bash
sugar validate
```

if your assest are in a folder named `assets` or:

```bash
sugar validate <ASSETS_DIR>
```

to specify a custom asset `<ASSETS DIR>` folder name.

> **Note:** It is important to validate your assets before the upload to avoid having to repeat the upload process.

#### 3. `upload`

The `upload` command uploads assets to the specified storage and creates the cache file for the Candy Machine:

```bash
sugar upload
```

if your assest are in a folder named `assets` or:

```bash
sugar upload <ASSETS DIR>
```

There is also the option to specify the path for the configuration file with the `-c` option (default `config.json`) and the name of the cache file with the option `--cache` (default `cache.json`).

The `upload` command can be resumed (re-run) at any point in case the upload is not completed successfully &mdash; only files that have not yet being uploaded are processed. It also automatically detects when the content of media/metadata files change and re-uploads them, updating the cache file accordingly. In other words, if you need to change a file, you only need to copy the new (modified) file to your assets folder and re-run the `upload` command. There is no need to manually edit the cache file.

#### 4. `deploy`

Once all assets are uploaded and the cache file is successfully created, you are ready to deploy your items to Solana:

```bash
sugar deploy
```

The `deploy` command will write the information of your cache file to the Candy Machine account on-chain. This effectively creates the Cancy Machine and displays its on-chain ID &mdash; use this ID to query its information on-chain using an [explorer](https://explorer.solana.com/). You can specify the path for the configuration file with the `-c` option (default `config.json`) and the name of the cache file with the option `--cache` (default `cache.json`) in case you are not using the default names.


After a succesful deploy, the Candy Machine is ready to be minted according to its `goLiveDate` and `whitelistMintSettings`.

> **Note:** The authority wallet (the one used to create the Candy Machine) can mint bypassing the `goLiveDate` setting.

#### 5. `verify`

The `verify` command checks that all items in your cache file have been successfully written on-chain:

```bash
sugar verify
```

if you are using the default cache file name (`cache.json`) or:

```bash
sugar verify --cache <CACHE>
```

to specify a different cache file path. If you deploy has been succesfully, the verification return no errors. At this point, you can set up your [minting webpage](http://docs.metaplex.com/candy-machine-v2/mint-frontend) to allow your community the chance to mint.

### Other Commands

Sugar includes other commands to manage a Candy Machine.

#### `mint`

The `mint` command mints NFTs from a Candy Machine from the command-line.

```bash
sugar mint
```

if you are using the default cache file name (`cache.json`) or:

```bash
sugar mint --cache <CACHE>
```

to specify a different cache file path. You can specify the number of NFTs to mint using the option `-n`:

```bash
sugar mint -n 10
```

The above command will mint 10 NFTs from the Candy Machine.

> **Note:** It is not possible to mint tokens from the command line if you have `gatekeeper` settings enabled. If you would like to mint tokens, update the `goLiveDate` to `null` and temporarily disable the `gatekeeper` settings.

#### `show`

The `show` command displays the on-chain config of an existing candy machine:

```bash
sugar show <CANDY MACHINE>
```

where the `<CANDY MACHINE>` is the Candy Machine ID &mdash; the ID given by the `deploy` command.

#### `update`

The `update` command is used to modify the current configuration of a Candy Machine. Most configuration settings can be updated in a CMv2 with a single command, with the exception of:
- `number` of items in the Candy Machine can only be updated when `hiddenSettings` are being used;
- switching to use `hiddenSettings` is only possible if the `number` of items is equal to `0`. After the switch, you will be able to update the `number` of items.

To update the configuration, modify your `config.json` (or equivalent) file and execute:

```bash
sugar update
```

if you are using the default cache file name (`cache.json`) and configuration file (`config.json`). Otherwise, use:

```bash
sugar update -c <CONFIG> --cache <CACHE>
```

where `<CONFIG>` is the path to the configuration file and `<CACHE>` is the path to the cache file.

> You need to be careful when updating a live Candy Machine, since setting a wrong value will immediately affect its functionality.

#### `withdraw`

When the mint from a Candy Machine is complete, it is possible to recover the funds used to pay rent for the data stored on-chain. To initiate the withdraw:

```bash
sugar withdraw <CANDY MACHINE>
```

where the `<CANDY MACHINE>` is the Candy Machine ID &mdash; the ID given by the `deploy` command. It is possible to withdraw funds from all Candy Machines associated with the current keypair:

```bash
sugar withdraw
```

or list all Candy Machines and their associated funds from the current keypair:

```bash
sugar withdraw --list
```

> You should not withdraw the rent of a live Candy Machine, as the Candy Machine will stop working when you drain its account.

## Further Reading

The [Candy Machine v2 documentation](http://docs.metaplex.com/candy-machine-v2/introduction) provides a more detailed explanation of each step of the deploy of a Candy Machine. Although there a differences in Sugar commands, the overall process is similar.
