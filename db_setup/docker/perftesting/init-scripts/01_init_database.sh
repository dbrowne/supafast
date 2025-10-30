#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    -- Create application user
    CREATE USER perf_user WITH PASSWORD '${PERF_USER_PASSWORD:-perf_password}';

    -- Create database
    CREATE DATABASE sec_master OWNER perf_user;

    -- Grant permissions
    GRANT ALL PRIVILEGES ON DATABASE perftest TO perf_user;

    -- Connect to sec_master database
    \c perftest


    -- Grant extension usage
    GRANT USAGE ON SCHEMA public TO perf_user;
    GRANT CREATE ON SCHEMA public TO perf_user;
EOSQL
