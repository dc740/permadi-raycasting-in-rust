#!/usr/bin/env bash
                                                        
echo Cleaning up previous builds...
rm demo_server/web_app/vrao.wasm_bg.wasm demo_server/web_app/vrao.js
echo Copying js files
mkdir -p demo_server/web_app/js
rm -f demo_server/web_app/js/*.js
cp ./js/*.js  demo_server/web_app/js/

echo Copying image files
mkdir -p demo_server/web_app/images
cp ./images/*.ff  demo_server/web_app/images/
cp ./resources.json  demo_server/web_app/

echo Compiling web application
cd main_app
wasm-pack build --target web --out-name vrao.wasm -- --features web --no-default-features || exit 1

echo Optimizing the compiled wasm and moving the output to the server
# call wasm-opt manually if you need to workaround https://github.com/rustwasm/wasm-pack/issues/974
# wasm-opt pkg/vrao.wasm -o ../demo_server/web_app/vrao.wasm_bg.wasm -O4 -all -ffm --enable-simd || exit 1
cp pkg/vrao.wasm ../demo_server/web_app/vrao.wasm_bg.wasm

cp pkg/vrao.js ../demo_server/web_app/

cd -
echo
echo %%%%%%%% COMPILATION DONE. OPENING DEV SERVER: %%%%%%%
python3 demo_server/server.py
