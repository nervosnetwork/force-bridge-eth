#!/usr/bin/env bash
lib_dirs=("lib/eth-bridge-lockscript-lib" "lib/eth-light-client-lockscript-lib" "lib/eth-recipient-typescript-lib")
for dir in "${lib_dirs[@]}"
do
	if [ -d $dir ]; then
		cd $dir && cargo tarpaulin --exclude-files ../../types/* --exclude-files ../../eth-spv-lib/* --exclude-files ../../tests/* -o html --output-dir ../../target/tarpaulin/$dir && cd ../../
	fi
done
