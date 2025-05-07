#!/usr/bin/env bash
#
# Initialize the database

set -e

db_dir=$(dirname $0)
db_file="db.db3"

if [ "$1" = test ]; then
	db_file="test.db3"
fi

sqlite3 -echo "$db_dir/$db_file" < "$db_dir/db.sql"

if [ "$1" = debug ]; then
	sqlite3 -echo "$db_dir/$db_file" < "$db_dir/debug.sql"
fi
