DB_PATH=/tmp/force.db
DATABASE_URL=sqlite:${DB_PATH}
SCHEMA_PATH := src/util/schemas
SCHEMA_DEST_PATH := src/util/generated

migrate:
	rm -f ${DB_PATH}
	sqlx db create && sqlx migrate run

schema:
	moleculec --language rust --schema-file ${SCHEMA_PATH}/ckb_tx_proof.mol > ${SCHEMA_DEST_PATH}/ckb_tx_proof.rs
