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
  `ckb_spv_proof` varchar(2048) DEFAULT NULL,
  `ckb_block_number` int(11) unsigned DEFAULT NULL,
  `ckb_raw_tx` varchar(4096) DEFAULT NULL,
  `eth_block_number` int(11) unsigned DEFAULT '0',
  PRIMARY KEY (`id`),
  KEY `ckb_burn_tx_hash` (`ckb_burn_tx_hash`),
  KEY `eth_tx_hash` (`eth_tx_hash`),
  KEY `block_number` (`ckb_block_number`),
  KEY `recipient_addr` (`recipient_addr`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;



# Dump of table ckb_unconfirmed_block
# ------------------------------------------------------------

DROP TABLE IF EXISTS `ckb_unconfirmed_block`;

CREATE TABLE `ckb_unconfirmed_block` (
  `id` int(11) unsigned NOT NULL,
  `number` int(11) unsigned DEFAULT NULL,
  `hash` varchar(64) DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;



# Dump of table cross_chain_height_info
# ------------------------------------------------------------

DROP TABLE IF EXISTS `cross_chain_height_info`;

CREATE TABLE `cross_chain_height_info` (
  `id` tinyint(1) unsigned NOT NULL AUTO_INCREMENT,
  `height` int(11) unsigned NOT NULL DEFAULT '0',
  `client_height` int(11) unsigned NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

LOCK TABLES `cross_chain_height_info` WRITE;
/*!40000 ALTER TABLE `cross_chain_height_info` DISABLE KEYS */;

INSERT INTO `cross_chain_height_info` (`id`, `height`, `client_height`)
VALUES
	(1,0,0),
	(2,0,0);

/*!40000 ALTER TABLE `cross_chain_height_info` ENABLE KEYS */;
UNLOCK TABLES;


# Dump of table eth_to_ckb
# ------------------------------------------------------------

DROP TABLE IF EXISTS `eth_to_ckb`;

CREATE TABLE `eth_to_ckb` (
  `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
  `eth_lock_tx_hash` varchar(64) NOT NULL DEFAULT '',
  `status` varchar(40) NOT NULL DEFAULT '',
  `token_addr` varchar(40) DEFAULT '',
  `sender_addr` varchar(40) DEFAULT NULL,
  `locked_amount` varchar(20) DEFAULT NULL,
  `bridge_fee` varchar(20) DEFAULT NULL,
  `ckb_recipient_lockscript` varchar(512) DEFAULT NULL,
  `sudt_extra_data` varchar(256) DEFAULT NULL,
  `ckb_tx_hash` varchar(64) DEFAULT NULL,
  `eth_spv_proof` varchar(15360) DEFAULT NULL,
  `eth_block_number` int(11) unsigned DEFAULT NULL,
  `replay_resist_outpoint` varchar(128) DEFAULT NULL,
  `ckb_block_number` int(11) unsigned DEFAULT '0',
  PRIMARY KEY (`id`),
  KEY `eth_lock_tx_hash` (`eth_lock_tx_hash`),
  KEY `replay_resist_outpoint` (`replay_resist_outpoint`),
  KEY `block_number` (`eth_block_number`),
  KEY `sender_addr` (`sender_addr`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;



# Dump of table eth_unconfirmed_block
# ------------------------------------------------------------

DROP TABLE IF EXISTS `eth_unconfirmed_block`;

CREATE TABLE `eth_unconfirmed_block` (
  `id` int(11) unsigned NOT NULL,
  `number` int(10) unsigned DEFAULT NULL,
  `hash` varchar(64) DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
