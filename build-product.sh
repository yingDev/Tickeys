#!/bin/bash

echo "compiling...";
cargo build --release;
cargo build --release; 


if [ $? -eq 0 ] 
then
	echo "copying files...";
	cp target/release/Tickeys Tickeys.app/Contents/MacOS/;
	rm -rf Tickeys.app/Contents/Resources
	cp -r Resources Tickeys.app/Contents/
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