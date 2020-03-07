set -e

rm -rf target/deploy
pushd ui
wasm-pack build --target web
rollup ./main.js --format iife --file ./pkg/bundle.js
popd
mkdir -p target/deploy/pkg
cp ui/static/* target/deploy
cp ui/pkg/bundle.js ui/pkg/tody_chat_ui_bg.wasm target/deploy/pkg
pushd target/deploy
tar -cvzf ../ui.tar.gz *
popd
