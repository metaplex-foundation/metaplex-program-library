#!/usr/bin/env bash

set -e
set -x

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cd "$SCRIPT_DIR"

cd native
wasm-pack build --target web

# fixup duplicate init manually...
GLUE_FILE="pkg/private_metadata_js.js"

# in-place sed. extra empty argument to indicate we don't care about creating a
# backup. delete it on linux or with gnu sed
sed -i '' 's/async function init/async function wasm_init/g' "$GLUE_FILE"
sed -i '' 's/export default init/export default wasm_init/g' "$GLUE_FILE"
# this one is a bit dubious
sed -i '' 's/init.__wbindgen_wasm_module/wasm_init.__wbindgen_wasm_module/g' "$GLUE_FILE"

cd ..

WEB_UTILS_DIR="web/src/utils/privateMetadata"
rm -r $WEB_UTILS_DIR
mv native/pkg $WEB_UTILS_DIR
