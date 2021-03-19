# sign-server
Multi-signature server for force-bridge-eth.

## Build this project
```
git clone https://github.com/LeonLi000/force-bridge-eth.git
cd sign-server
cargo build
```

## Usage

``` shell
➜  sign-server ✗ ./sign-server run --help 
  
sign-server-run 

USAGE:
    sign-server run [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --cell-script <cell-script>
            [default:
            590000001000000030000000310000001313a0eaa571a9168e44ceba1a0d0a328840d9de43aab2388af7c860b57c9a0c01240000005edca2d744b6eaa347de7ff0edcd2e6e88ab8f2836bcbd0df0940026956e5f8107000000]

        --ckb-indexer-url <ckb-indexer-url>    [default: http://127.0.0.1:8116]
        --ckb-key-path <ckb-key-path>          [default: conf/ckb_key]
        --ckb-rpc-url <ckb-rpc-url>            [default: http://127.0.0.1:8114]
        --config-path <config-path>            [default: conf/config.toml]
        --eth-key-path <eth-key-path>          [default: conf/eth_key]
        --eth-rpc-url <eth-rpc-url>            [default: http://127.0.0.1:8545]
        --listen-url <listen-url>              [default: 0.0.0.0:3031]
```
### Example: 

```
./sign-server run --config-path conf/config.toml --listen-url 0.0.0.0:3031 --ckb-key-path conf/ckb_key --eth-key-path conf/eth_key --cell-script 590000001000000030000000310000001313a0eaa571a9168e44ceba1a0d0a328840d9de43aab2388af7c860b57c9a0c01240000005edca2d744b6eaa347de7ff0edcd2e6e88ab8f2836bcbd0df0940026956e5f8107000000
```
