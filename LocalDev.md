
# Running this thing locally

Recommend using or building your own solana development Docker container.
Install Docker, and docker-compose.
If you don't want to build your own dev container - you can use this one I built and
deployed on DockerHub if not have an AVX2 enabled CPU. NOT A PRODUCTION CONTAINER.
`dmitryr117/anchor-noavx2:0.24.2`

# Steps to Install

IMPORTANT. Make sure to fallow first setup instructions in: https://github.com/dmitryr117/solana-local-env
Readme.md file to set up docker solana development environment.


Installing Metaplex Development Environment:

1. Have to be inside `appdev` directory `cd appdev`.

2. Git clone metaplex program library: `git clone https://github.com/dmitryr117/metaplex-program-library.git`

3. Go inside cloned directory and switch git branch and pull for changes:
```
cd metaplex-program-library
git checkout local-dev-env
git pull
```

4. Go up two directory levels `cd ../..`

5. Check that you are inside same directory as `docker-compose.yml` file using `ls` command.

6. Start docker containers using `docker-compose up -d` command. Can take a few minutes to load 
container images if this is your first time running this command.

7. Sign into docker container from your terminal using: `docker exec -ti soldev /bin/bash` command.
Your terminal look should might change to signify that you are inside container.

8. cd into `cd /appdev/metaplex-program-library`

9. Run `yarn install`

10. Now time to transform original environment into development environment. `yarn set.dev.env`
Wait for command to complete. Sometimes had compilation failure when compiling RUST packages.
`yarn set.dev.env` command calls `cargo build` in the end. So if it fails when compiling RUST
pacjages - run `cargo build` again.

11. Compile Smart Contracts `yarn compile.contracts`. Same compilation issues happen here sometimes
as well. If something gets a compilation error - juist rerun the `yarn compile.contracts` again.

12. Need to start a Solana Test Validator now, so open another terminal and log into same docker
container again. `docker exec -ti soldev /bin/bash`. Make sure you are in root directory with `pwd`,
and run `solana-test-validator` should see validator starting chowing blockchain information with
number of blocks increasing.

13. Switch to first terminal window, make sure you are still in `/appdev/metaplex-program-library`,
and run `yarn deploy.contracts`. This will fund a generic `/wallets/metaplex.key.json` wallet with
100 SOL, and upload all 10 Solana programs / smart contracts to your local solana test network.

14. Build and upload NPM packages. When working with NPM packages same as working with smart contracts
developers need a local repository for testing and local development. Hence Verdaccio. We have
Verdaccio running in a separatte container already configured to store `@metaplex-foundattion/*` packages
locally. Run `yarn publish.npms` command to process and publish your packages to verdaccio.

15. If you ever need to remove these packages - open a terminal, go to directory where you have
the `docker-compose.yml` file, and run `sudo rm -rf /verdaccio/storage/@metaplex-foundation`. This
will remove your local @metaplex-foundation packages.

16. After you are done developing, and want to push your code updates - make sure to transform your workspace
back to original. Make sure you are inside your `soldev` container inside `/appdev/metaplex-program-library` 
And run `yarn unset.dev.env`. After it completes - you can do `git status`, `git add .`
and other commands without your public keys trying to overwrite original keys in repository. 

That's it. Don't forget to turn off your `solana-test-validator` in another terminal with Ctrl-C when you are 
done with it.


# Known Issues

Sometimes when building - have to clear registry and rebuild packages
rm -rf /root/.cargo/registry/src/*
rm -rf target/debug
rm -rf target/release
rm -rf /opt/solana/bin/sdk/bpf


# NPM and YARN

Sometimes to set up js packages will have to clear cache.
```
yarn cache clean --all
yarn install
```

# Verdacio Yarn issue Workaround

Temporary workaround:

Login with NPM
Copy-paste generated token from `~/.npmrc to ~/.yarnrc.yml.`

npm adduser --registry https://registry.npmjs.org/

```
cat ~/.npmrc
npm.my-project.pro/:_authToken="GAOEuaeouaoEUo+u3=="

cat ~/.yarnrc.yml
npmRegistries:
  "https://npm.my-project.pro":
    npmAuthToken: GAOEuaeouaoEUo+u3==
```

Then yarn npm publish for verdaccio works fine.
Make sure to either delete packages from verdacio storage, or update package versions in
package.json files to update packages. 
