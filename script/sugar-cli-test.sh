#!/bin/bash
#
# Sugar CLI - Candy Machine automated test
#
# To suppress prompts, you will need to set/export the following variables:
#
# ENV_URL="mainnet-beta"
# RPC="https://ssc-dao.genesysgo.net/"
# STORAGE="arweave-sol"
#
# ENV_URL="devnet"
# RPC="https://psytrbhymqlkfrhudd.dev.genesysgo.net:8899/"
# STORAGE="arweave"
#
# ITEMS=10
# MULTIPLE=0
#
# RESET="Y"
# EXT="png"
# CLOSE="Y"
# CHANGE="Y"
# TEST_IMAGE="Y"
#
# ARWEAVE_JWK="null"
# INFURA_ID="null"
# INFURA_SECRET="null"
# AWS_BUCKET="null"
#
# The custom RPC server option can be specified either by the flag -r <url>

CURRENT_DIR=$(pwd)
SCRIPT_DIR=$(cd -- $(dirname -- "${BASH_SOURCE[0]}") &>/dev/null && pwd)
PARENT_DIR="$(dirname "$SCRIPT_DIR")"
ASSETS_DIR=$CURRENT_DIR/assets
CACHE_DIR=$CURRENT_DIR
SUGAR_BIN="cargo run --bin sugar --"
SUGAR_LOG="sugar.log"
RESUME_FILE="$SCRIPT_DIR/.sugar_resume"

# Remote files to test the upload
PNG_MIN="https://arweave.net/N3LqmO6yURUK1JxV9MJtH8YeqppEtZhKuy3RB0Tqm3A/?ext=png"
PNG="https://arweave.net/yFoNLhe6cBK-wj0n_Wu-XuX7DC75VbMsNKwVbRSz4iQ?ext=png"
GIF="https://arweave.net/-cksjCg70nWw-NE8F-DDR4FGQNfQQrWONWm5TIGt6e8?ext=gif"
JPG="https://arweave.net/X5Czkw4R6EAq5kKW0VgX0oVjLlhn3MV2L0LId0PgZPQ?ext=jpg"
MP4="https://arweave.net/kM6fxv3Qj_Gcn8tcq9dU8wpZAXHNEWvEfVoIpRJzg8c/?ext=mp4"

# Metadata URL for large collection tests
METADATA_URL="https://arweave.net/uJSdJIsz_tYTcjUEWdeVSj0aR90K-hjDauATWZSi-tQ"
# Media hash (png) for large collection tests
MEDIA_HASH="209a200ebea39be9e9e7882da2bc5e652fb690e612abecb094dc13e06db84e54"

# output colours
RED() { echo $'\e[1;31m'$1$'\e[0m'; }
GRN() { echo $'\e[1;32m'$1$'\e[0m'; }
BLU() { echo $'\e[1;34m'$1$'\e[0m'; }
MAG() { echo $'\e[1;35m'$1$'\e[0m'; }
CYN() { echo $'\e[1;36m'$1$'\e[0m'; }

# default test templates
function default_settings() {
    MANUAL_CACHE="n"
    ITEMS=10
    MULTIPLE=1

    RESET="Y"
    EXT="png"
    CLOSE="Y"
    CHANGE="n"
    TEST_IMAGE="n"

    ARWEAVE_JWK="null"
    INFURA_ID="null"
    INFURA_SECRET="null"
    AWS_BUCKET="null"
}

function max_settings() {
    MANUAL_CACHE="Y"

    RESET="Y"
    EXT="png"
    CLOSE="Y"
    CHANGE="n"
    TEST_IMAGE="n"

    ARWEAVE_JWK="null"
    INFURA_ID="null"
    INFURA_SECRET="null"
    AWS_BUCKET="null"
}

function mainnet_env() {
    ENV_URL="mainnet-beta"
    RPC="https://ssc-dao.genesysgo.net/"
    STORAGE="bundlr"
}

function devnet_env() {
    ENV_URL="devnet"
    RPC="https://psytrbhymqlkfrhudd.dev.genesysgo.net:8899/"
    STORAGE="bundlr"
}

#-----------------------------------------------------------------------------#
# SETUP                                                                       #
#-----------------------------------------------------------------------------#

RESUME=0

echo ""
CYN "Sugar CLI - Candy Machine automated test"
CYN "----------------------------------------"

echo ""
CYN "Test template:"
echo "1. interactive"
echo "2. devnet (default)"
echo "3. mainnet-beta"
echo "4. devnet [manual cache]"

if [ -f "$RESUME_FILE" ]; then
    echo "5. resume previous run"
    echo -n "$(CYN "Select test template [1-5]") (default 'devnet'): "
else
    echo -n "$(CYN "Select test template [1-4]") (default 'devnet'): "
fi

read Template
case "$Template" in
    1)
        echo ""
        echo "[$(date "+%T")] Starting interactive test"
    ;;
    3)
        mainnet_env
        default_settings
    ;;
    4)
        devnet_env
        max_settings
    ;;
    5)
        source $RESUME_FILE
        RESUME=1
        RESET="n"
    ;;
    *)
        devnet_env
        default_settings
    ;;
esac

# Environment

if [ -z ${ENV_URL+x} ]; then
    ENV_URL="devnet"

    echo ""
    CYN "Environment:"
    echo "1. devnet (default)"
    echo "2. mainnet-beta"
    echo -n "$(CYN "Select the environment [1-2]") (default 'devnet'): "
    read Input
    case "$Input" in
        1) ENV_URL="devnet" ;;
        2) ENV_URL="mainnet-beta" ;;
    esac
fi

# RPC server can be specified from the command-line with the flag "-r"
# Otherwise the default public one will be used

if [ -z ${RPC+x} ]; then
    RPC="https://api.${ENV_URL}.solana.com"
fi

while getopts r: flag; do
    case "${flag}" in
        r) RPC=${OPTARG} ;;
    esac
done

# Storage

if [ -z ${STORAGE+x} ]; then
    STORAGE="bundlr"

    echo ""
    CYN "Storage type:"
    echo "1. bundlr (default)"
    echo "2. aws"
    echo  -n "$(CYN "Select the storage type [1-2]") (default 1): "
    read Input
    case "$Input" in
        1) STORAGE="bundlr" ;;
        2) STORAGE="aws" ;;
    esac
fi

if [ -z ${ARWEAVE_JWK+x} ]; then
    ARWEAVE_JWK="null"

    if [ "$STORAGE" = "arweave-bundle" ]; then
        echo -n $(CYN "Arweave JWK wallet file: ")
        read ARWEAVE_JWK
    fi
fi

if [ -z ${INFURA_ID+x} ]; then
    INFURA_ID="null"
    INFURA_SECRET="null"

    if [ "$STORAGE" = "ipfs" ]; then
        echo -n $(CYN "Infura Project ID: ")
        read INFURA_ID
        echo -n $(CYN "Infura Secret: ")
        read INFURA_SECRET
    fi
fi

if [ -z ${AWS_BUCKET+x} ]; then
    AWS_BUCKET="null"

    if [ "$STORAGE" = "aws" ]; then
        echo -n $(CYN "AWS bucket name: ")
        read AWS_BUCKET
    fi
fi

# Asset type

ANIMATION=0

if [ -z ${EXT+x} ]; then
    IMAGE=$PNG
    EXT="png"
    echo ""
    CYN "Asset type:"
    echo "1. PNG (default)"
    echo "2. JPG"
    echo "3. GIF"
    echo "4. MP4"
    echo -n "$(CYN "Select the file type [1-4]") (default 1): "
    read Input
    case "$Input" in
    1)
        IMAGE=$PNG
        EXT="png"
        ;;
    2)
        IMAGE=$JPG
        EXT="jpg"
        ;;
    3)
        IMAGE=$GIF
        EXT="gif"
        ;;
    4)
        IMAGE=$PNG
        EXT="png"
        ANIMATION=1
        ;;
    esac
else
    case "$EXT" in
    png)
        IMAGE=$PNG
        ;;
    png_min)
        IMAGE=$PNG_MIN
        EXT="png"
        ;;
    jpg)
        IMAGE=$JPG
        ;;
    gif)
        IMAGE=$GIF
        ;;
    mp4)
        IMAGE=$PNG
        EXT="png"
        ANIMATION=1
        ;;
    *)
        RED "[$(date "+%T")] Aborting: invalid asset type ${EXT}"
        exit 1
        ;;
    esac
fi

# Collection size

if [ -z ${ITEMS+x} ]; then
    echo ""
    echo -n "$(CYN "Number of items") (default 10): "
    read Number

    if [ -z "$Number" ]; then
        ITEMS=10
    else
        # make sure we are dealing with a number
        ITEMS=$(($Number + 0))
    fi
fi

# Test image.extension instead of index

if [ -z ${TEST_IMAGE+x} ]; then
    echo ""
    echo -n "$(CYN "Test image.ext replacement [Y/n]") (default 'Y'): "
    read TEST_IMAGE
    if [ -z "$TEST_IMAGE" ]; then
        TEST_IMAGE="Y"
    fi
fi

# Test reupload

if [ -z ${CHANGE+x} ]; then
    echo ""
    echo -n "$(CYN "Test re-deploy [Y/n]") (default 'Y'): "
    read CHANGE
    if [ -z "$CHANGE" ]; then
        CHANGE="Y"
    fi
fi

# Mint tokens

if [ -z ${MULTIPLE+x} ]; then
    echo ""
    echo -n "$(CYN "Number of tokens to mint") (default 1): "
    read Number

    if [ -z "$Number" ]; then
        MULTIPLE=1
    else
        # make sure we are dealing with a number
        MULTIPLE=$(($Number + 0))
    fi
fi

# Clean up

if [ -z ${RESET+x} ]; then
    echo ""
    echo -n "$(CYN "Remove previous cache and assets [Y/n]") (default 'Y'): "
    read RESET
    if [ -z "$RESET" ]; then
        RESET="Y"
    fi
fi

if [ -z ${CLOSE+x} ]; then
    echo ""
    echo -n "$(CYN "Close candy machine and withdraw funds at the end [Y/n]") (default 'Y'): "
    read CLOSE
    if [ -z "$CLOSE" ]; then
        CLOSE="Y"
    fi
fi

# Resume checkpoint

cat >$RESUME_FILE <<-EOM
#!/bin/bash

MANUAL_CACHE="$MANUAL_CACHE"
ITEMS=$ITEMS
MULTIPLE=$MULTIPLE

RESET="$RESET"
EXT="$EXT"
CLOSE="$CLOSE"
CHANGE="$CHANGE"
TEST_IMAGE="$TEST_IMAGE"

ARWEAVE_JWK="$ARWEAVE_JWK"
INFURA_ID="$INFURA_ID"
INFURA_SECRET="$INFURA_SECRET"
AWS_BUCKET="$AWS_BUCKET"

ENV_URL="$ENV_URL"
RPC="$RPC"
STORAGE="$STORAGE"
EOM

echo ""

#-----------------------------------------------------------------------------#
# SETTING UP                                                                  #
#-----------------------------------------------------------------------------#

# Wallet keypair file

WALLET_KEY="$(solana config get keypair | cut -d : -f 2)"
CACHE_NAME="sugar-test"
CACHE_FILE="$CACHE_DIR/cache-${CACHE_NAME}.json"
LAST_INDEX=$((ITEMS - 1))

TIMESTAMP=`date "+%d/%m/%y %T"`

# removes temporary files
function clean_up {
    rm $CONFIG_FILE 2>/dev/null
    rm -rf $ASSETS_DIR 2>/dev/null
    rm -rf $CACHE_FILE 2>/dev/null
    rm -rf $SUGAR_LOG 2>/dev/null
    rm -rf test_item 2>/dev/null
}

if [ "${RESET}" = "Y" ]; then
    echo "[$(date "+%T")] Removing previous cache and assets"
    clean_up
fi

# preparing the assets metadata
read -r -d $'\0' METADATA <<-EOM
{
    "name": "[$TIMESTAMP] Test #%s",
    "symbol": "TEST",
    "description": "Sugar CLI Test #%s",
    "seller_fee_basis_points": 500,
    "image": "%s"%b
    "attributes": [{"trait_type": "Flavour", "value": "Sugar"}],
    "properties": {
        "files": [
        {
            "uri": "%s",
            "type": "%s"
        }],
        "category": "Sugar Test"
    }
}
EOM

if [ $RESUME -eq 0 ]; then
    echo "[$(date "+%T")] Creating assets"

    # Creation of the collection. This will generate ITEMS x (json, image)
    # files in the ASSETS_DIR

    if [ ! -d $ASSETS_DIR ]; then
        mkdir $ASSETS_DIR
        # loads the animation asset
        if [ "$ANIMATION" -eq 1 ]; then
            curl -L -s $MP4 >"$ASSETS_DIR/template_animation.mp4"
            SIZE=$(wc -c "$ASSETS_DIR/template_animation.mp4" | grep -oE '[0-9]+' | head -n 1)

            if [ $SIZE -eq 0 ]; then
                RED "[$(date "+%T")] Aborting: could not download sample mp4"
                exit 1
            fi
        fi

        curl -L -s $IMAGE >"$ASSETS_DIR/template_image.$EXT"
        SIZE=$(wc -c "$ASSETS_DIR/template_image.$EXT" | grep -oE '[0-9]+' | head -n 1)

        if [ $SIZE -eq 0 ]; then
            RED "[$(date "+%T")] Aborting: could not download sample image"
            exit 1
        fi

        # initialises the assets - this will be multiple copies of the same
        # image/json pair with a new index
        INDEX="image"
        for ((i = 0; i < $ITEMS; i++)); do
            if [ ! "$TEST_IMAGE" = "Y" ]; then
                INDEX=$i
            fi
            NAME=$(($i + 1))
            MEDIA_NAME="$INDEX.$EXT"
            MEDIA_TYPE="image/$EXT"
            ANIMATION_URL=","
            cp "$ASSETS_DIR/template_image.$EXT" "$ASSETS_DIR/$i.$EXT"
            if [ "$ANIMATION" = 1 ]; then
                cp "$ASSETS_DIR/template_animation.mp4" "$ASSETS_DIR/$i.mp4"
                ANIMATION_URL=",\n\t\"animation_url\": \"$i.mp4\","
            fi
            printf "$METADATA" $NAME $NAME $MEDIA_NAME $ANIMATION_URL $MEDIA_NAME $MEDIA_TYPE >"$ASSETS_DIR/$i.json"
        done
        rm "$ASSETS_DIR/template_image.$EXT"
        # quietly removes the animation template (it might not exist)
        rm -f "$ASSETS_DIR/template_animation.mp4"
    fi

    if [ "$MANUAL_CACHE" == "Y" ]; then
        echo -n "{\"program\":{\"candyMachine\":\"\", \"candyMachineCreator\":\"\"}, \"items\":{" >> $CACHE_FILE
        
        for ((i = 0; i < $ITEMS; i++)); do
            if [ "$i" -gt "0" ]; then
                echo -n "," >> $CACHE_FILE
            fi
            NAME=$(($i + 1))
            METADATA_HASH=`sha256sum "$ASSETS_DIR/$i.json" | cut -d ' ' -f 1`
            echo -n "\"$i\":{\"name\":\"[$TIMESTAMP] Test #$NAME\",\"media_hash\":\"$MEDIA_HASH\",\"media_link\":\"$PNG\",\"metadata_hash\":\"$METADATA_HASH\",\"metadata_link\":\"$METADATA_URL\",\"onChain\":false}" >> $CACHE_FILE
        done

        echo -n "}}" >> $CACHE_FILE
    fi
fi

# Candy Machine configuration

CONFIG_FILE="config.json"

cat >$CONFIG_FILE <<-EOM
{
    "price": 0.1,
    "number": $ITEMS,
    "symbol": "TEST",
    "sellerFeeBasisPoints": 500,
    "gatekeeper": null,
    "solTreasuryAccount": "$(solana address)",
    "splTokenAccount": null,
    "splToken": null,
    "goLiveDate": "$(date "+%Y-%m-%dT%T%z" | sed "s@^.\{22\}@&:@")",
    "endSettings": null,
    "whitelistMintSettings": null,
    "hiddenSettings": null,
    "uploadMethod": "${STORAGE}",
    "ipfsInfuraProjectId": "${INFURA_ID}",
    "ipfsInfuraSecret": "${INFURA_SECRET}",
    "awsS3Bucket": "${AWS_BUCKET}",
    "retainAuthority": false,
    "isMutable": true,
    "creators": [
    {
      "address": "$(solana address)",
      "share": 100
    }
  ]
}
EOM

#-----------------------------------------------------------------------------#
# AUXILIARY FUNCTIONS                                                         #
#-----------------------------------------------------------------------------#

# edit cache file for reupload
function change_cache {
    cat $CACHE_FILE | jq -c ".items.\"0\".onChain=false|.items.\"0\".name=\"Changed #0\"|del(.items.\""$LAST_INDEX"\")" \
        >$CACHE_FILE.tmp && mv $CACHE_FILE.tmp $CACHE_FILE
    if [[ $(cat $CACHE_FILE | grep "Changed #0") ]]; then
        GRN "Success: cache file changed"
    else 
        RED "Failure: cache file was not changed"
    fi
}

# run the upload command
function upload {
    $SUGAR_BIN upload -c ${CONFIG_FILE} --keypair $WALLET_KEY --cache $CACHE_FILE -r $RPC $ASSETS_DIR
    EXIT_CODE=$?
    if [ ! $EXIT_CODE -eq 0 ]; then
        MAG "<<<"
        RED "[$(date "+%T")] Aborting: upload failed"
        exit 1
    fi
}

# run the deploy command
function deploy {
    $SUGAR_BIN deploy -c ${CONFIG_FILE} --keypair $WALLET_KEY --cache $CACHE_FILE -r $RPC
    EXIT_CODE=$?
    if [ ! $EXIT_CODE -eq 0 ]; then
        MAG "<<<"
        RED "[$(date "+%T")] Aborting: deploy failed"
        exit 1
    fi
}

# run the verify upload command
function verify {
    $SUGAR_BIN verify --keypair $WALLET_KEY --cache $CACHE_FILE -r $RPC
    EXIT_CODE=$?
    if [ ! $EXIT_CODE -eq 0 ]; then
        MAG "<<<"
        RED "[$(date "+%T")] Aborting: verify failed"
        exit 1
    fi
}

#-----------------------------------------------------------------------------#
# COMMAND EXECUTION                                                           #
#-----------------------------------------------------------------------------#

if [ "${CHANGE}" = "Y" ] && [ "$(command -v jq)" = "" ]; then
    echo "[$(date "+%T")] $(RED "Required 'jq' command could not be found, skipping reupload test")"
    CHANGE="n"
fi

echo "[$(date "+%T")] Deploying Candy Machine with $ITEMS items"
echo "[$(date "+%T")] Environment: ${ENV_URL}"
echo "[$(date "+%T")] RPC URL: ${RPC}"
echo "[$(date "+%T")] Testing started using ${STORAGE} storage"
echo ""
CYN "1. Validating JSON metadata files"
echo ""
MAG ">>>"
$SUGAR_BIN validate $ASSETS_DIR
EXIT_CODE=$?
MAG "<<<"

if [ ! $EXIT_CODE -eq 0 ]; then
    RED "[$(date "+%T")] Aborting: validation failed"
    exit 1
fi

echo ""
CYN "2. Uploading assets"
echo ""
MAG ">>>"
upload
MAG "<<<"
echo ""

echo ""
CYN "3. Deploying Candy Machine"
echo ""
MAG ">>>"
deploy
MAG "<<<"
echo ""

echo ""
CYN "4. Verifying deployment"
echo ""
MAG ">>>"
verify
MAG "<<<"
echo ""

echo ""
if [ "${CHANGE}" = "Y" ]; then
    CYN "5. Editing cache and testing re-deploy"
    echo ""
    MAG ">>>"
    change_cache
    deploy
    verify
    MAG "<<<"
else
    CYN "5. Editing cache and testing re-deploy (Skipped)"
fi

echo ""
CYN "6. Minting"
echo ""
MAG ">>>"
$SUGAR_BIN mint --keypair $WALLET_KEY --cache $CACHE_FILE -r $RPC -n $MULTIPLE
EXIT_CODE=$?
MAG "<<<"

if [ ! $EXIT_CODE -eq 0 ]; then
    RED "[$(date "+%T")] Aborting: mint failed"
    exit 1
fi

if [ "${CLOSE}" = "Y" ]; then
    CANDY_MACHINE_ID=`cat $CACHE_FILE | sed -n -e 's/^\(.*\)\(\"candyMachine\":\"\)\([a-zA-Z0-9]*\)\(.*\)$/\3/p'`
    echo ""
    CYN "7. Withdrawing Candy Machine funds and clean up"
    echo ""
    MAG ">>>"
    $SUGAR_BIN withdraw --keypair $WALLET_KEY -r $RPC $CANDY_MACHINE_ID
    EXIT_CODE=$?
    MAG "<<<"
    
    if [ ! $EXIT_CODE -eq 0 ]; then
        RED "[$(date "+%T")] Aborting: withdraw failed"
        exit 1
    fi

    clean_up
fi

# save to delete the resume checkpoint
rm -rf $RESUME_FILE 2>/dev/null

echo ""
echo "[$(date "+%T")] Test completed"