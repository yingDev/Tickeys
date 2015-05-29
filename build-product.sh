#!/bin/bash

echo "compiling...";
cargo build --release;

if [ $? -eq 0 ] 
then
	cp target/release/Tickeys Tickeys.app/Contents/MacOS/;
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