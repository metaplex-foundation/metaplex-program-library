#!/bin/bash
#
# Sugar CLI binary installation script
# ------------------------------------
#
# The purpose of this script is to automate the download and installation
# of Sugar CLI binary.
#
# The script does a (simple) platform detection, downloads the binary
# for the detected platform and copies it to a folder in the PATH variable.
# 
# Currently the supported platforms are macOS, Linux, or another Unix-like OS
# running variations of the sh Unix shells.
#

RED() { echo $'\e[1;31m'$1$'\e[0m'; }
GRN() { echo $'\e[1;32m'$1$'\e[0m'; }
CYN() { echo $'\e[1;36m'$1$'\e[0m'; }

abort_on_error() {
    if [ ! $1 -eq 0 ]; then
        RED "Aborting: operation failed"
        exit 1
    fi
}

CYN  "🍬 Sugar CLI binary installation script"
echo "---------------------------------------"
echo ""

OS_FLAVOUR="$(uname -s)"
PROCESSOR="$(uname -m)"

# we need to check whether we are running on an ARM
# architecture or not

case "$PROCESSOR" in
    arm* | aarch* | ppc* )
        if [ "$OS_FLAVOUR" != Darwin ]; then
            echo "Binary for $PROCESSOR architecture is not currently supported using this installer. Please follow the instructions at:"
            echo "  => $(CYN https://github.com/metaplex-foundation/sugar)"
            echo ""
            echo "for details on alternate installation methods."
            exit 1
        fi
        ;;

    *)
        # good to go
        ;;
esac

# determine the latest release

echo "Checking available releases..."
echo ""

TAGS=`curl -L --silent https://api.github.com/repos/metaplex-foundation/sugar/releases | grep tag_name`

TAGS_V1=""
TAGS_V2=""

for TAG in $TAGS
do
  if [[ "$TAG" != *tag_name* ]]; then
    # v1 release
    if [[ "$TAG" == *v1* ]]; then
      if [[ "$TAG" > "\"v1" ]] && [ -z "$TAG_V1" ]; then
        TAG_V1=`echo $TAG | sed "s/\"//g;s/,//"`
      fi
    fi
    # v2 release
    if [[ "$TAG" == *v2* ]]; then
      if [[ "$TAG" > "\"v2" ]] && [ -z "$TAG_V2" ]; then
        TAG_V2=`echo $TAG | sed "s/\"//g;s/,//"`
      fi
    fi
  fi
done

echo "🧰 $(CYN "Available releases:")"
echo ""
echo "Recommended:"
echo "$(CYN "1. $TAG_V2") (Metaplex Candy Machine V3)"
echo ""
echo "Legacy:"
echo "2. $TAG_V1 (Metaplex Candy Machine V2)"
echo ""
echo -n "$(CYN "Select a release [1-2]") (default 1): "
read Version

case "$Version" in
    2)
        RELEASE=$TAG_V1
    ;;
    *)
        RELEASE=$TAG_V2
    ;;
esac
 
BIN="sugar"
VERSION="ubuntu-latest"

if [ "$OS_FLAVOUR" = Darwin ]; then
    case "$PROCESSOR" in
        arm* )
            VERSION="macos-m1-latest"
            ;;
        *)
            VERSION="macos-intel-latest"
            ;;
    esac
fi

DIST="$VERSION"

# creates a temporary directory to save the distribution file
SOURCE="$(mktemp -d)"

echo ""
echo "🖥  $(CYN "Downloading binary")"

# downloads the distribution file
REMOTE="https://github.com/metaplex-foundation/sugar/releases/download/$RELEASE/"
curl -L --progress-bar $REMOTE$BIN"-"$DIST --output "$SOURCE/$DIST"
abort_on_error $?

SIZE=$(wc -c "$SOURCE/$DIST" | grep -oE "[0-9]+" | head -n 1)

if [ $SIZE -eq 0 ]; then
    RED "Aborting: could not download Sugar distribution"
    exit 1
fi

# makes sure the binary will be executable
chmod u+x "$SOURCE/$DIST"
abort_on_error $?

echo ""
echo "📤 $(CYN "Moving binary into place")"
echo ""

if [ ! "$(command -v $BIN)" = "" ]; then
    # binary already found on system, ask if we should
    # replace it
    EXISTING="$(which $BIN)"

    echo "Sugar binary was found at:"
    echo "  => $(CYN $EXISTING)"
    echo ""
    echo -n "$(CYN "Replace it? [Y/n]") (default 'n'): "
    read REPLACE

    if [ "$REPLACE" = Y ]; then
        echo ""
        echo "'$BIN' binary will be moved to '$(dirname "$EXISTING")'."

        mv "$SOURCE/$DIST" "$EXISTING"
        abort_on_error $?
    else
        # nothing else to do, replacement was cancelled
        RED "Aborting: replacement cancelled"
        exit 1
    fi
else
    # determines a suitable directory for the binary - preference:
    # 1) ~/.cargo/bin if exists
    # 2) ~/bin otherwise
    TARGET="$HOME/.cargo/bin"

    if [ ! -d "$TARGET" ]; then
        TARGET="$HOME/bin"

        if [ ! -d "$TARGET" ]; then
            mkdir $TARGET
        fi
    fi

    echo "'$BIN' binary will be moved to '$TARGET'."

    mv "$SOURCE/$DIST" "$TARGET/$BIN"
    abort_on_error $?

    if [ "$(command -v $BIN)" = "" ]; then
        ENV_FILE="$HOME/.$(basename $SHELL)rc"

        if [ -f "$ENV_FILE" ]; then
            echo "  => adding '$TARGET' to 'PATH' variable in '$ENV_FILE'"
            echo "export PATH=\"$HOME/bin:\$PATH\"" >> "$ENV_FILE"
        else
            echo "  => adding '$TARGET' to 'PATH' variable to execute 'sugar' from any directory."
            echo "     - file '$(CYN $ENV_FILE)' was not found"
            echo "" 
            echo -n "$(CYN "Would you like to create '$ENV_FILE'? [Y/n]") (default 'n'): "
            read CREATE

            if [ "$CREATE" = Y ]; then
                echo "  => adding '$TARGET' to 'PATH' variable in '$ENV_FILE'"
                echo "export PATH=\"$HOME/bin:\$PATH\"" >> "$ENV_FILE"
            else
                echo ""
                echo "     $(RED "[File creation cancelled]")"
                echo ""
                echo "     - to manually add '$TARGET' to 'PATH' you will need to:"
                echo ""
                echo "       1. create a file named '$(basename $ENV_FILE)' in your directory '$(dirname $ENV_FILE)'"
                echo "       2. add the following line to the file:"
                echo ""
                echo "           export PATH=\"$HOME/bin:\$PATH\""
            fi
        fi
    fi
fi

echo ""
# sanity check
if [ "$(command -v $BIN)" = "" ]; then
    # installation was completed, but sugar is not in the PATH
    echo "✅ $(GRN "Installation complete:") restart your shell to update 'PATH' variable or type '$TARGET/$BIN' to start using it."
else
    # success
    echo "✅ $(GRN "Installation successful:") type '$BIN' to start using it."
fi