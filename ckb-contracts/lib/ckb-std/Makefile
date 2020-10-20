TARGET := riscv64imac-unknown-none-elf
DOCKER_IMAGE := jjy0/ckb-capsule-recipe-rust:2020-9-28
CC := riscv64-unknown-elf-gcc

default: integration-in-docker

fix-permission-in-docker:
	chown -R $$OWNER target; chown -R $$OWNER $$HOME/.cargo/git; chown -R $$OWNER $$HOME/.cargo/registry;

integration-in-docker: test-shared-lib
	docker run --rm -eOWNER=`id -u`:`id -g` -v `pwd`:/code -v ${HOME}/.cargo/git:/root/.cargo/git -v ${HOME}/.cargo/registry:/root/.cargo/registry -w/code ${DOCKER_IMAGE} bash -c 'make integration; CODE=$$?; make fix-permission-in-docker; exit $$CODE'

publish-crate:
	docker run --rm -eOWNER=`id -u`:`id -g` -v `pwd`:/code -v ${HOME}/.cargo/git:/root/.cargo/git -v ${HOME}/.cargo/registry:/root/.cargo/registry -v ${HOME}/.cargo/credentials:/root/.cargo/credentials -w/code ${DOCKER_IMAGE} bash -c 'cargo publish --target ${TARGET}; make fix-permission-in-docker'

publish: publish-crate

clean:
	docker run --rm -eOWNER=`id -u`:`id -g` -v `pwd`:/code -v ${HOME}/.cargo/git:/root/.cargo/git -v ${HOME}/.cargo/registry:/root/.cargo/registry -v ${HOME}/.cargo/credentials:/root/.cargo/credentials -w/code ${DOCKER_IMAGE} bash -c 'cargo clean; make -C test clean'

test-shared-lib:
	make -C test/shared-lib all-via-docker

integration: check test

test:
	make -C test test

check:
	cargo check --target ${TARGET} --examples

install-tools:
	cargo install --git https://github.com/xxuejie/ckb-binary-patcher.git

.PHONY: test check

