cd $(dirname $0)
set -e
CC='cc' CXX='c++' cargo run --release -- --itool -o $(pwd)/runtime < $(pwd)/runtime/itool.json
cmake -GNinja -S./runtime -B./build -DCMAKE_EXPORT_COMPILE_COMMANDS=YES 
ninja -C build