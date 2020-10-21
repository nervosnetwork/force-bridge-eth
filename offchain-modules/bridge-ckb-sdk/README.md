# tockb-cli

## Quick Start

```bash
# usage 1: use it as sdk and write the rust script yourself.
$ cd sdk
$ cargo run --example demo

# usage 2: use it as a cli.
$ cd cli
$ cargo build
$ ../target/debug/tockb-cli -h
# a full demo to run through the process
$ make test-tockb-cli 

# usage 3: use it via jsonrpc HTTP request.
# You can serve a jsonrpc service for dapp users. 
# Users will get a unsigned transaction which They can sign and send themselves.
$ ../target/debug/tockb-cli server

# install httpie
$ http 127.0.0.1:3030 <<< '{"jsonrpc": "2.0", "method": "contract", "params": {"from_lockscript_addr": "ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37", "tx_fee": "0.1", "sub_cmd": {"DepositRequest": {"user_lockscript_addr": "ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37", "pledge": 10000, "kind": 1, "lot_size": 1}}}, "id": 1}'
HTTP/1.1 200 OK
Connection: keep-alive
Content-Length: 2151
Content-Type: application/json; charset=utf-8
Date: Sat, 26 Sep 2020 21:18:05 GMT
Keep-Alive: timeout=4
Proxy-Connection: keep-alive

{
    "id": 1,
    "jsonrpc": "2.0",
    "result": {
        "cell_deps": [
            {
                "dep_type": "code",
                "out_point": {
                    "index": "0x1",
                    "tx_hash": "0x46f4c6b5835425461e96af47c2b7a26894a1df6d525ef737659c418a48700f23"
                }
            },
            {
                "dep_type": "code",
                "out_point": {
                    "index": "0x0",
                    "tx_hash": "0x46f4c6b5835425461e96af47c2b7a26894a1df6d525ef737659c418a48700f23"
                }
            },
            {
                "dep_type": "dep_group",
                "out_point": {
                    "index": "0x0",
                    "tx_hash": "0xace5ea83c478bb866edf122ff862085789158f5cbff155b7bb5f13058555b708"
                }
            }
        ],
        "hash": "0xe2c5b84fe6e0c6c0ebb74165b882122deca72f035fd279c8330aaf3294d0ecdc",
        "header_deps": [],
        "inputs": [
            {
                "previous_output": {
                    "index": "0x1",
                    "tx_hash": "0x8291f9aafe38ebd77d472d6cab3f37c1b12507955748dcadd770b167964e269d"
                },
                "since": "0x0"
            },
            {
                "previous_output": {
                    "index": "0x1",
                    "tx_hash": "0xbb7ec1650987faf487774f8af908e577cc4860b827c3dd87716d207e21e660d3"
                },
                "since": "0x0"
            }
        ],
        "outputs": [
            {
                "capacity": "0xe8d4a51000",
                "lock": {
                    "args": "0x54d30309189c0c567a428ae5c9f01f9f5bd3d581c3e20fb798ae2a7392bee09b",
                    "code_hash": "0x0d665001e9c412712ceb28ea809639400af13fa53df65e36cb46a9cf3c4d4023",
                    "hash_type": "data"
                },
                "type": {
                    "args": "0x018291f9aafe38ebd77d472d6cab3f37c1b12507955748dcadd770b167964e269d01000000",
                    "code_hash": "0x7d92992b7c750c050d887ae9c007f17e53aa3acee42c8315b515c10a2e526c66",
                    "hash_type": "data"
                }
            },
            {
                "capacity": "0x1bbce86779e4c9a0",
                "lock": {
                    "args": "0xc8328aabcd9b9e8e64fbc566c4385c3bdeb219d7",
                    "code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
                    "hash_type": "type"
                },
                "type": null
            }
        ],
        "outputs_data": [
            "0x4e01000028000000290000002a0000007300000077000000ac000000b0000000e50000001a0100000101490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000c8328aabcd9b9e8e64fbc566c4385c3bdeb219d7000000003500000010000000300000003100000000000000000000000000000000000000000000000000000000000000000000000000000000000000003500000010000000300000003100000000000000000000000000000000000000000000000000000000000000000000000000000000350000001000000030000000310000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000c0000002c000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "0x"
        ],
        "version": "0x0",
        "witnesses": []
    }
}
```
