#!/usr/bin/env bash
export CODECOV_TOKEN="9049b41d-9a3b-4610-94e2-ddaddff3ad11"
lib_dirs=("lib/eth-bridge-lockscript-lib" "lib/eth-light-client-lockscript-lib" "lib/eth-recipient-typescript-lib")
for dir in "${lib_dirs[@]}"
do
	if [ -d $dir ]; then
		cd $dir && cargo tarpaulin --exclude-files ../../types/* --exclude-files ../../eth-spv-lib/* --exclude-files ../../tests/* -o html -o xml --output-dir ../../target/tarpaulin/$dir && bash <(curl -s https://codecov.io/bash) -f ../../target/tarpaulin/$dir/cobertura.xml && cd ../../
	fi
done
