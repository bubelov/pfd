#!/bin/bash

case $1 in
  apply)
    query="
      CREATE TABLE exchange_rate (
        base TEXT,
	quote TEXT,
        rate REAL
      )
    "
    sqlite3 pfd.db "$query"
    sqlite3 pfd.db "INSERT INTO exchange_rate(base, quote, rate) VALUES ('USD', 'BTC', 35000)"
  ;;
  rollback)
    rm pfd.db
  ;;
  *)
    echo "Unknown command: $1"
    exit 1
  ;;
esac
