CREATE TABLE `eth_to_ckb` (
	`id` INT ( 11 ) UNSIGNED NOT NULL AUTO_INCREMENT,
	`eth_lock_tx_hash` VARCHAR ( 64 ) NOT NULL DEFAULT '',
	`status` VARCHAR ( 40 ) NOT NULL DEFAULT 'pending',
	`token_addr` VARCHAR ( 40 ) DEFAULT NULL,
	`sender_addr` VARCHAR ( 40 ) DEFAULT NULL,
	`locked_amount` VARCHAR ( 40 ) DEFAULT NULL,
	`bridge_fee` VARCHAR ( 64 ) DEFAULT NULL,
	`ckb_recipient_lockscript` VARCHAR ( 20 ) DEFAULT NULL,
	`sudt_extra_data` VARCHAR ( 20 ) DEFAULT NULL,
	`ckb_tx_hash` VARCHAR ( 64 ) DEFAULT NULL,
	`err_msg` VARCHAR ( 2048 ) DEFAULT NULL,
	PRIMARY KEY ( `id` )
) ENGINE = INNODB DEFAULT CHARSET = utf8;

CREATE TABLE `ckb_to_eth` (
	`id` INT ( 11 ) UNSIGNED NOT NULL AUTO_INCREMENT,
	`ckb_burn_tx_hash` VARCHAR ( 64 ) UNIQUE NOT NULL,
	`status` VARCHAR ( 40 ) NOT NULL DEFAULT 'pending',
	`recipient_addr` VARCHAR ( 40 ) DEFAULT NULL,
	`token_addr` VARCHAR ( 40 ) DEFAULT NULL,
	`lock_contract_addr` VARCHAR ( 40 ) DEFAULT NULL,
	`bridge_lock_hash` VARCHAR ( 40 ) DEFAULT NULL,
	`token_amount` VARCHAR ( 40 ) DEFAULT NULL,
	`fee` VARCHAR ( 40 ) DEFAULT NULL,
	`eth_tx_hash` VARCHAR ( 64 ) DEFAULT NULL,
	`err_msg` VARCHAR ( 2048 ) DEFAULT NULL,
PRIMARY KEY ( `id` )
) ENGINE = INNODB DEFAULT CHARSET = utf8;
