cargo build 1>/dev/null 2>/dev/null

if [ $? -ne 0 ]; then
	echo "Build failed!" | lolcat 1>&2
	exit 1
fi


cargo test 1>/dev/null 2>/dev/null

if [ $? -ne 0 ]; then
	echo "Tests failed!" | lolcat 1>&2
	exit 2
fi

sudo mv target/debug/smiscasm /usr/local/bin
