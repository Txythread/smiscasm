# Make a production build and store it in the /usr/local/bin

cargo --version 1>/dev/null

if [ $? -ne 0 ]; then
	echo "Couldn't invoke cargo. Is rust installed correctly?!" 1>&2
	exit 1
fi



cargo build --release 1>/dev/null 2>/dev/null

if [ $? -ne 0 ]; then
	echo "Build failed!" 1>&2
	exit 1
fi


cargo test 1>/dev/null 2>/dev/null

if [ $? -ne 0 ]; then
	echo "Tests failed!" 1>&2
	exit 2
fi

sudo --user=root mv target/release/smiscasm /usr/local/bin
