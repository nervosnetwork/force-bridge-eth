# CKB dev-chain

### 1. docker build
```shell script
cd docker/ckb
docker build -t ckb-dev:v0.36.0 . 
```
### 2. docker run
```shell script
docker run --rm -it -p 8114:8114 ckb-dev:v0.36.0
```

# CKB indexer

### 1. docker build
```shell script
cd docker/ckb-indexer
docker build -t ckb-indexer . 
```
### 2. docker run
```shell script
docker run --rm -it -p 8116:8116 ckb-indexer
```

# Geth private chain

### 1. docker build
```shell script
cd docker/geth
docker build -t geth-priv:v1.9.23 . 
```
### 2. docker run

The first time you run container, it will take 5~10 minutes for geth to be ready because of generating dag.

We use docker volume `geth-dag` to store dag data, so the geth will be ready very soon the next time you run container. 

```shell script
 docker run --rm -it --mount source=geth-dag,target=/root/.ethash -p 8545:8545 tockb/geth-priv:v1.9.23
```

# Docker compose

```shell script
cd docker
# start up
docker-compose up
# stop and remove containers
docker-compose down
```
