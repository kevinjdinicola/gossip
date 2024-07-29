#!/bin/zsh

CARGO_HOME=`dirname $(which cargo)`
BINDINGS_DIR=$PROJECT_DIR/bindings/
MOST_RECENTLY_GENERATED_LIBRARY=`find $CARGO_TARGET_DIR -name "$EXECUTABLE_PATH" -print0 | xargs -0 ls -lt | head -n 1 | awk '{print $9}'`
# the path from xcode has a bunch of crap on it
# that causes things to not build lol
#env
if [[ "$TARGET_BUILD_DIR" = *"Previews"* ]]; then
    echo "Xcode is running for previews."
else
    echo "Xcode is not running for previews."
    
    export PATH=$CARGO_HOME:/usr/bin/
    cargo run --manifest-path uniffi-bindgen/Cargo.toml --bin uniffi-bindgen generate --library $MOST_RECENTLY_GENERATED_LIBRARY --language swift --out-dir $BINDINGS_DIR
fi


