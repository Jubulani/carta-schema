#!/usr/bin/env bash

set -o nounset
set -o errexit
set -o pipefail

wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
tar xzf master.tar.gz
cd kcov-master
mkdir build
cd build
cmake ..
make
make install DESTDIR=../../kcov-build
cd ../..
rm -rf kcov-master
FILENAME=$(find . -wholename "./native/target/debug/carta_schema-*")
./kcov-build/usr/local/bin/kcov --exclude-pattern=/.cargo,/usr/lib --verify "native/target/cov/" "$FILENAME"
bash <(curl -s https://codecov.io/bash)
echo "Uploaded code coverage"
