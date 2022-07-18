#!/usr/bin/env bash
# check for dependencies
if ! [ -x "$(command -v psql)" ]; then
    echo "Error: psql is not installed"
    exit 1
fi

if ![ -x "$(command -v sqlx)" ]; then
    echo "Error: sqlx is not installed"
    echo "Use:"
    echo "    cargo install sqlx"
    echo "to install sqlx"
fi

set -x
set -eo pipefail

# Check if a custom user has been set, otherwise default to "postgres"
DB_USER=${POSTGRES_USER:=postgres}
# Check if a custom password has been set, otherwise default to "password"
DB_PASSWORD=${POSTGRES_PASSWORD:=password}
# Check if a custom database name has been set, otherwise default to "newsletter"
DB_NAME=${POSTGRES_DB:=newsletter}
# Check if a custom database port has been set, otherwise default to "5432"
DB_PORT=${POSTGRES_PORT:=5432}

# Launch postgres using Docker
if [[ -z "${SKIP_DOCKER}" ]]; then
    docker run \
        --name newsletter-postgres \
        --rm \
        -e POSTGRES_USER="$DB_USER" \
        -e POSTGRES_PASSWORD="$DB_PASSWORD" \
        -e POSTGRES_DB="$DB_NAME" \
        -p "$DB_PORT":5432 \
        -d postgres \
        postgres -N 1000
fi

# ping the database until its ready
export PGPASSWORD="$DB_PASSWORD"
until psql -h localhost -p "$DB_PORT" -U "$DB_USER" -c '\q' 2>/dev/null; do
    echo >&2 "Postgres is unavailable - sleeping"
    sleep 1
done

echo >&2 "Postgres is up and running on port $DB_PORT"

export DATABASE_URL="postgres://$DB_USER:$DB_PASSWORD@localhost:$DB_PORT/$DB_NAME"
sqlx database create
sqlx migrate run
