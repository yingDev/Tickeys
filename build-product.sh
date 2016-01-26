#!/bin/bash

echo "compiling...";

export OPENSSL_LIB_DIR=/usr/local/opt/openssl/lib
export OPENSSL_INCLUDE_DIR=/usr/local/opt/openssl/include
export OPENSSL_STATIC=true

cargo clean;
cargo build --release;


if [ $? -eq 0 ] 
then
	echo "copying files...";
	cp target/release/Tickeys Tickeys.app/Contents/MacOS/;
	rm -rf Tickeys.app/Contents/SharedSupport;
	cp -r SharedSupport Tickeys.app/Contents/;
	
	ver=`fgrep "version" -m 1 Cargo.toml | cut -d\" -f2`;
	echo "updating version string... $ver";
	plutil -replace CFBundleVersion -string "$ver" Tickeys.app/Contents/Info.plist;
else
	echo "error: cargo build";
	exit -1;
fi

if [ -z "$DEVELOPER_ID" ]
then
	echo "Please set your DEVELOPER_ID envar first!";
	exit -1;
fi
echo "codesigning with DEVELOPER_ID=$DEVELOPER_ID";

codesign --force --sign "Developer ID Application: $DEVELOPER_ID" Tickeys.app;

echo "completed.";