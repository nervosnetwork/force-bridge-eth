CREATE TABLE IF NOT EXISTS eth_to_ckb
(
    id                       INTEGER PRIMARY KEY NOT NULL,
    eth_lock_tx_hash         VARCHAR UNIQUE NOT NULL,
    status                   VARCHAR        NOT NULL DEFAULT 'pending',
    token_addr               VARCHAR                 DEFAULT NULL,
    sender_addr              VARCHAR                 DEFAULT NULL,
    locked_amount            VARCHAR                 DEFAULT NULL,
    bridge_fee               VARCHAR                 DEFAULT NULL,
    ckb_recipient_lockscript VARCHAR                 DEFAULT NULL,
    sudt_extra_data          VARCHAR                 DEFAULT NULL,
    ckb_tx_hash              VARCHAR                 DEFAULT NULL,
    err_msg                  VARCHAR                 DEFAULT NULL
)
