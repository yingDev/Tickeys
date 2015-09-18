#!/bin/bash

echo "compiling...";
cargo build --release;


if [ $? -eq 0 ] 
then
	echo "copying files...";
	cp target/release/Tickeys Tickeys.app/Contents/MacOS/;
	rm -rf Tickeys.app/Contents/Resources;
	cp -r Resources Tickeys.app/Contents/;
	
	ver=`fgrep "version" -m 1 Cargo.toml | cut -d\" -f2`;
	echo "updating version string... $ver";
	plutil -replace CFBundleVersion -string "$ver" Tickeys.app/Contents/Info.plist;
else
	echo "error: cargo build";
	exit;
fi

if [ -z "$DEVELOPER_ID" ]
then
	echo "Please set your DEVELOPER_ID envar first!";
	exit;
fi
echo "codesigning with DEVELOPER_ID=$DEVELOPER_ID";

codesign --force --sign "Developer ID Application: $DEVELOPER_ID" Tickeys.app;

echo "completed.";