DROP TABLE IF EXISTS `eth_tx_relayer`;

CREATE TABLE `eth_tx_relayer` (
  `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
  `block_number` bigint(11) unsigned NOT NULL,
  `lock_tx_hash` varchar(64) NOT NULL DEFAULT '',
  `status` varchar(40) NOT NULL DEFAULT 'pending',
  `err_msg` varchar(256) DEFAULT NULL,
  `lock_tx_proof` varchar(8192) DEFAULT NULL,
  `create_time` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  `update_time` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `lock_tx_hash` (`lock_tx_hash`),
  KEY `block_number` (`block_number`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
