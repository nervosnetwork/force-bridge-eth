DROP TABLE IF EXISTS `replay_resist_cells`;

CREATE TABLE `replay_resist_cells` (
  `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
  `outpoint` varchar(1024) NOT NULL,
  `token` varchar(40) NOT NULL,
  `status` varchar(40) NOT NULL DEFAULT 'available',
  `create_time` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  `update_time` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `token` (`token`),
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
