#!/bin/sh
# Garante que o banco secllm existe (útil se o volume foi criado sem POSTGRES_DB)
psql -v ON_ERROR_STOP=0 -d postgres -c "CREATE DATABASE secllm;" 2>/dev/null || true
