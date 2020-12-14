CREATE TABLE IF NOT EXISTS eth_to_ckb
(
    id                       INTEGER PRIMARY KEY NOT NULL,
    eth_lock_tx_hash         VARCHAR UNIQUE      NOT NULL,
    status                   VARCHAR             NOT NULL DEFAULT 'pending',
    token_addr               VARCHAR                      DEFAULT NULL,
    sender_addr              VARCHAR                      DEFAULT NULL,
    locked_amount            VARCHAR                      DEFAULT NULL,
    bridge_fee               VARCHAR                      DEFAULT NULL,
    ckb_recipient_lockscript VARCHAR                      DEFAULT NULL,
    sudt_extra_data          VARCHAR                      DEFAULT NULL,
    ckb_tx_hash              VARCHAR                      DEFAULT NULL,
    err_msg                  VARCHAR                      DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS ckb_to_eth
(
    id                 INTEGER PRIMARY KEY NOT NULL,
    ckb_burn_tx_hash   VARCHAR UNIQUE      NOT NULL,
    status             VARCHAR             NOT NULL DEFAULT 'pending',
    recipient_addr     VARCHAR                      DEFAULT NULL,
    token_addr         VARCHAR                      DEFAULT NULL,
    lock_contract_addr VARCHAR                      DEFAULT NULL,
    bridge_lock_hash   VARCHAR                      DEFAULT NULL,
    token_amount       VARCHAR                      DEFAULT NULL,
    fee                VARCHAR                      DEFAULT NULL,
    eth_tx_hash        VARCHAR                      DEFAULT NULL,
    err_msg            VARCHAR                      DEFAULT NULL
);
