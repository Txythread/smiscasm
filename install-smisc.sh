#!/bin/bash

echo "Pulling github repos ..."
git clone https://github.com/Txythread/smiscasm
git clone https://github.com/Txythread/smiscvm
git clone https://github.com/Txythread/smisc-connect

wait $!


cargo --version 1>/dev/null

if [ $? -ne 0 ]; then
	echo "Couldn't ivoke cargo. Is rust installed correctly?!" 1>&2
	exit 1
fi


rustc --version 1>/dev/null

if [ $? -ne 0 ]; then
	echo "Couldn't invoke rustc. Is rust installed correctly?!" 1>&2
	exit 1
fi


openssl 1>/dev/null

if [ $? -ne 0 ]; then
	echo "Couldn't invoke openssl. Please install openssl to proceed" 1>&2
	exit 1
fi




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
