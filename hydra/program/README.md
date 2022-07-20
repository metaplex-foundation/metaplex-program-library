# Hydra

Collective account pooling, fan out wallet, dao treasury, all of the things you need to FAN OUT

## Setup && Development

To get started the basic steps are to install all the JS dependencies, cargo dependencies will be
installed automatically when you run `anchor build`. This package uses the latest yarn, which is
yarn 3 and I will accept no hate for using the latest yarn :)

```
yarn
```

There are three components in this repo..

1. Program
2. SDK
3. Docs

### Program Development

The Hydra smart contract is written with anchor but we have changed the development flow slightly to
allow for what we think is a better SDK. Using `anchor build` you get the BPF so file and the IDL.
You then run the following to generate the SDK and spin up a local validator when you want to test
your changes.

```shell
yarn run api:gen
yarn run amman:start
```

Now you have a validator running your newly compiled SO file and any other programs you have listed
in the `.ammanrc.cjs`.

Currently we require the local validator to have the `Token Metadata` program from `Metaplex`. We
suggest you clone the `Metaplex Program Library` and put it in the same root folder as hydra like
this:

```shell
 /root-folder
    /hydra
    /metaplex-program-library
```

To build the latest token metadata program you will need to do the following:

```shell
cd metaplex-program-library/token-metadata/program
cargo build-bpf --bpf-out-dir ../../target/deploy/
```

Now your top level MPL target folder will have the token metadata `so` file that `amman` can find
and deploy to the local validator.

After accomplishing this your workflow to build new code will be:

```shell
anchor build //Optional as the next commant runs anchor build for you
yarn run api:gen
yarn run amman:start
```

### SDK Development

Now that you know how to build the program, building the SDK is easy. All you need to do is run:

```shell
yarn run watch
```

This will watch your `packages/sdk` folder for any changes, which means it will build the SDK for
use in apps or tests when the auto generated code from the api generator is saved in the folder.

#### Testing

Now that you have the ability to build the program, generated sdk and the rest of the sdk, running
tests is a breeze. With the local validator up and running you can simply run.

```shell
yarn test
```

Using ts-node/register you can even setup breakpoints and debuggers on your IDE to debug your tests.
If you dont have the local validator running do:

```shell
yarn run amman:start
yarn test
```

Or if you are like Noah or Will and are extremely lazy and can't be bothered to run more than one
command run `yarn run mega-test`

### Docs Development

```shell
cd packages/docs
yarn start
```
