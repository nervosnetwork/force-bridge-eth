DROP TABLE IF EXISTS `ckb_to_eth`;

CREATE TABLE `ckb_to_eth` (
  `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
  `ckb_burn_tx_hash` varchar(64) NOT NULL DEFAULT '',
  `status` varchar(40) NOT NULL DEFAULT 'pending',
  `recipient_addr` varchar(40) DEFAULT NULL,
  `token_addr` varchar(40) DEFAULT NULL,
  `lock_contract_addr` varchar(40) DEFAULT NULL,
  `bridge_lock_hash` varchar(64) DEFAULT NULL,
  `token_amount` varchar(20) DEFAULT NULL,
  `fee` varchar(20) DEFAULT NULL,
  `eth_tx_hash` varchar(64) DEFAULT NULL,
  `err_msg` varchar(256) DEFAULT NULL,
  `ckb_spv_proof` varbinary(2048) DEFAULT NULL,
  `block_number` int(11) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `ckb_burn_tx_hash` (`ckb_burn_tx_hash`),
  KEY `eth_tx_hash` (`eth_tx_hash`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;


DROP TABLE IF EXISTS `eth_to_ckb`;

CREATE TABLE `eth_to_ckb` (
  `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
  `eth_lock_tx_hash` varchar(64) NOT NULL DEFAULT '',
  `status` varchar(40) NOT NULL DEFAULT '',
  `token_addr` varchar(40) DEFAULT '',
  `sender_addr` varchar(40) DEFAULT NULL,
  `locked_amount` varchar(20) DEFAULT NULL,
  `bridge_fee` varchar(20) DEFAULT NULL,
  `ckb_recipient_lockscript` varchar(256) DEFAULT NULL,
  `sudt_extra_data` varchar(256) DEFAULT NULL,
  `ckb_tx_hash` varchar(64) DEFAULT NULL,
  `err_msg` varchar(256) DEFAULT NULL,
  `eth_spv_proof` varchar(8192) DEFAULT NULL,
  `block_number` int(11) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `eth_lock_tx_hash` (`eth_lock_tx_hash`),
  KEY `ckb_tx_hash` (`ckb_tx_hash`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
