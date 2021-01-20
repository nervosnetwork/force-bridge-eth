CREATE TABLE IF NOT EXISTS eth_to_ckb
(
    `id` INT ( 11 ) UNSIGNED NOT NULL AUTO_INCREMENT,
--     id                       INTEGER PRIMARY KEY NOT NULL,
    eth_lock_tx_hash         VARCHAR(255) NOT NULL UNIQUE,
    status                   VARCHAR(255)             NOT NULL DEFAULT 'pending',
    token_addr               VARCHAR(255)                      DEFAULT NULL,
    sender_addr              VARCHAR(255)                      DEFAULT NULL,
    locked_amount            VARCHAR(255)                      DEFAULT NULL,
    bridge_fee               VARCHAR(255)                      DEFAULT NULL,
    ckb_recipient_lockscript VARCHAR(4096)                      DEFAULT NULL,
    sudt_extra_data          VARCHAR(4096)                      DEFAULT NULL,
    ckb_tx_hash              VARCHAR(255)                      DEFAULT NULL,
    err_msg                  VARCHAR(10240)                      DEFAULT NULL,

    `create_time` TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
    `update_time` TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY ( `id` )
) ENGINE = INNODB DEFAULT CHARSET = utf8;


CREATE TABLE IF NOT EXISTS ckb_to_eth
(
--     id                 INTEGER PRIMARY KEY NOT NULL,
    `id` INT ( 11 ) UNSIGNED NOT NULL AUTO_INCREMENT,
    ckb_burn_tx_hash   VARCHAR(255) NOT NULL UNIQUE      ,
    status             VARCHAR(255)             NOT NULL DEFAULT 'pending',
    recipient_addr     VARCHAR(255)                      DEFAULT NULL,
    token_addr         VARCHAR(255)                      DEFAULT NULL,
    lock_contract_addr VARCHAR(255)                      DEFAULT NULL,
    bridge_lock_hash   VARCHAR(255)                      DEFAULT NULL,
    token_amount       VARCHAR(255)                      DEFAULT NULL,
    fee                VARCHAR(255)                      DEFAULT NULL,
    eth_tx_hash        VARCHAR(255)                      DEFAULT NULL,
    err_msg            VARCHAR(10240)                      DEFAULT NULL,
        `create_time` TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
    `update_time` TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY ( `id` )
) ENGINE = INNODB DEFAULT CHARSET = utf8;
