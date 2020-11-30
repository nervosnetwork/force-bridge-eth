DB_PATH=/tmp/force.db
DATABASE_URL=sqlite:${DB_PATH}

migrate:
	rm -f ${DB_PATH}
	sqlx db create && sqlx migrate run
