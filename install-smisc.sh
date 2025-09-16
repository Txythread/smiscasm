#!/bin/bash

echo "Pulling github repos ..."
git clone https://github.com/Txythread/smiscasm
git clone https://github.com/Txythread/smiscvm
git clone https://github.com/Txythread/smisc-connect

wait $!

echo "Executing build scripts ..."
echo "Note: This might take a while."
echo "You may be asked to enter your password."
echo "This is for moving the binaries."
echo "If you don't want this, execute the install scripts in the subdirectories yourself"
cd smiscasm
./production.sh

wait $!

cd ../smiscvm
./production.sh
cd ../smisc-connect
./build.sh
cd ..

wait $!

echo "Done. You can remove the newly downloaded smisc directories if you want to."