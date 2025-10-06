#!/bin/bash

# Make sure all required commands are installed.
which cargo 1>/dev/null

if [ $? -ne 0 ]; then
	echo "Couldn't invoke cargo. Is rust installed correctly?!" 1>&2
	exit 1
fi


which rustc 1>/dev/null

if [ $? -ne 0 ]; then
	echo "Couldn't invoke rustc. Is rust installed correctly?!" 1>&2
	exit 1
fi


which openssl 1>/dev/null

if [ $? -ne 0 ]; then
	echo "Couldn't invoke openssl. Please install openssl to proceed" 1>&2
	exit 1
fi

which sudo 1>/dev/null

if [ $? -ne 0 ]; then
	echo "Couldn't invoke sudo. Please install sudo to proceed" 1>&2
	exit 1
fi


which cc 1>/dev/null

if [ $? -ne 0 ]; then
	echo "Couldn't invoke (g)cc. Please install cc to proceed" 1>&2
	exit 1
fi


which pkg-config 1>/dev/null

if [ $? -ne 0 ]; then
	echo "Couldn't invoke pkg-config. Please install pkg-config to proceed" 1>&2
	exit 1
fi

which git 1>/dev/null

if [ $? -ne 0 ]; then
	echo "Couldn't invoke git. Please install git to proceed" 1>&2
	exit 1
fi


# Look if the github repos have been installed already
SKIP_DOWNLOAD=0 # Skip at default

cd smiscasm 1>/dev/null 2>/dev/null

if [ $? -ne 0 ]; then
  SKIP_DOWNLOAD=1 # Don't skip

else
  cd ..
fi


cd smiscvm 1>/dev/null 2>/dev/null

if [ $? -ne 0 ]; then
  SKIP_DOWNLOAD=1 # Don't skip

else
  cd ..
fi

cd smisc-connect 1>/dev/null 2>/dev/null

if [ $? -ne 0 ]; then
  SKIP_DOWNLOAD=1 # Don't skip

else
  cd ..
fi



if [ $SKIP_DOWNLOAD -ne 0 ]; then

  # Install the github repos
  echo "Pulling github repos ..."
  git clone https://github.com/Txythread/smiscasm
  git clone https://github.com/Txythread/smiscvm
  git clone https://github.com/Txythread/smisc-connect

  wait $!


else
  # Remove the leftovers if requested.
  printf "Do you want to download updates for previously installed repos? [y/N] "
  read -r response </dev/tty
  case "$response" in
      ([yY][eE][sS]|[yY])
          cd smiscasm
          git pull
	  wait $!

	  cd ..
          cd smiscvm
          git pull
	  wait $!

          cd ..
	  cd smisc-connect
          git pull
	  wait $!
	  cd ..
          ;;
      (*)
          ;;
  esac
fi

echo "Executing build scripts ..."
echo "Note: This might take a while."
echo "You may be asked to enter your password."
echo "This is for moving the binaries."
echo "If you don't want this, execute the install scripts in the subdirectories yourself"
cd smiscasm
./production.sh

wait $!


# Check if smiscasm compiled successfuly
which smiscasm 1>/dev/null

if [ $? -ne 0 ]; then
	echo "Couldn't compile smiscasm. Please try to execute the production.sh script in the smiscasm directory & fix the issue. Sorry." 1>&2
	exit 1
fi

echo "smiscasm compiled"


# Compile smiscvm and smisc-connect at once
cd ../smiscvm
./production.sh
cd ../smisc-connect
./build.sh
cd ..

wait $!

# Check if smiscvm compiled successfuly
which smiscvm 1>/dev/null

if [ $? -ne 0 ]; then
	echo "Couldn't compile smiscvm. Please try to execute the production.sh script in the smiscvm directory & fix the issue. Sorry. Btw smiscasm compiled just fine." 1>&2
	exit 1
fi

echo "smiscvm compiled"

# Check if smiscvm compiled successfuly
which smisc-connect 1>/dev/null

if [ $? -ne 0 ]; then
	echo "Couldn't compile smisc-connect. Please try to execute the production.sh script in the smisc-connect directory & fix the issue. Sorry. Btw smiscasm & smiscvm compiled just fine." 1>&2
	exit 1
fi


echo "smisc-connect compiled"

# Remove the leftovers if requested.
printf "Do you want to remove the installation folders (n if you want to modify the program)? [y/N] "
read -r response </dev/tty
case "$response" in
    ([yY][eE][sS]|[yY]) 
        sudo rm -r smiscasm
        sudo rm -r smiscvm
        sudo rm -r smisc-connect
        ;;
    (*) 
        ;;
esac

wait $!
echo "Done!"
