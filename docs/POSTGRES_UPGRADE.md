# PostgreSQL Upgrade (15 -> 17)

Your production volume currently holds data initialized under PostgreSQL 15. Attempting to start a 17.x container directly against that volume causes the fatal incompatibility error you observed:

```
FATAL:  database files are incompatible with server
DETAIL:  The data directory was initialized by PostgreSQL version 15, which is not compatible with this version 17.x.
```

## Safe Upgrade Options

### 1. Logical Dump & Restore (Simplest)

1. Run existing v15 container (ensure healthy).
2. Take a logical backup:
   ```bash
   pg_dump -Fc -h <host> -U <user> -d <db> -f backup.dump
   ```
3. Stop v15 container (keep volume as backup until done).
4. Start a temporary fresh v17 container with a NEW, EMPTY volume:
   ```bash
   docker run --name pg17_temp -e POSTGRES_USER=<user> -e POSTGRES_PASSWORD=<pass> -e POSTGRES_DB=<db> -v pg17_data:/var/lib/postgresql/data -p 5544:5432 -d postgres:17-alpine
   ```
5. Restore:
   ```bash
   pg_restore -c -h localhost -p 5544 -U <user> -d <db> backup.dump
   ```
6. Point your compose file to use the new `pg17_data` volume and image `postgres:17-alpine`.
7. Remove old v15 container/volume only after verifying application.

Pros: Easiest. Cons: Requires downtime during dump/restore.

### 2. In-Place `pg_upgrade` (Fast for large DBs)

1. Create two separate data directories / volumes:
   - `pg15_data` (current)
   - `pg17_data` (new, empty)
2. Run both images and copy binaries or use the `postgres` image + `pg_upgrade` utility inside a helper container mounting both volumes.
3. Execute `pg_upgrade` referencing old and new data dirs.
4. Run `analyze_new_cluster.sh` produced by upgrade.
5. Swap volumes / update compose to point to upgraded dir.

Pros: Much faster for big DBs. Cons: More steps.

### 3. Logical Replication (Minimal Downtime)

1. Stand up new v17 instance.
2. Enable publication on v15: `CREATE PUBLICATION pub_all FOR ALL TABLES;`
3. Create matching schema on v17 (run migrations if needed).
4. Create subscription on v17: `CREATE SUBSCRIPTION sub_all CONNECTION 'host=... user=... password=... dbname=...' PUBLICATION pub_all;`
5. After sync finishes, cut over application to v17.

Pros: Near zero downtime. Cons: More complexity.

## Choosing a Method

- Small DB (< a few GB) → dump & restore.
- Large DB & maintenance window acceptable → pg_upgrade.
- Need live cutover → logical replication.

## Compose Adjustments

Until upgrade completes, keep production compose on:

```
image: postgres:15-alpine
```

After successful upgrade, change to `postgres:17-alpine` and (important) point to the new upgraded volume (not the old one) or reuse the upgraded directory if you performed pg_upgrade.

## Verification Checklist

- Container logs: no FATAL version mismatch.
- `SELECT version();` returns expected 17.x.
- Migration status matches previous environment.
- App can connect & pass health endpoint.
- Recent writes visible.

## Rollback Plan

Keep the untouched v15 volume until post-upgrade validation passes. If issues, revert compose to `postgres:15-alpine` with old volume.

## Common Pitfalls

- Reusing the same volume with a higher major version without upgrade → version mismatch error.
- Forgetting to re-run `ANALYZE` after pg_upgrade → poor performance.
- Not backing up before upgrade → irreversible data loss risk.

## Quick Dump & Restore Script (example)

```bash
#!/usr/bin/env bash
set -euo pipefail
DB_HOST=localhost
DB_PORT=5433
DB_USER=appuser
DB_NAME=app_db
BACKUP=backup_$(date +%Y%m%d%H%M%S).dump

echo "Dumping v15 database..."
PGPASSWORD=app_pass pg_dump -Fc -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$BACKUP"

echo "Starting fresh v17 container..."
docker run --rm -d --name pg17_upgrade -e POSTGRES_USER=$DB_USER -e POSTGRES_PASSWORD=app_pass -e POSTGRES_DB=$DB_NAME -p 5544:5432 postgres:17-alpine

until PGPASSWORD=app_pass psql -h localhost -p 5544 -U $DB_USER -d $DB_NAME -c 'SELECT 1' >/dev/null 2>&1; do
  sleep 2
  echo "Waiting for pg17..."
done

echo "Restoring into v17..."
PGPASSWORD=app_pass pg_restore -c -h localhost -p 5544 -U $DB_USER -d $DB_NAME "$BACKUP"

echo "Verify versions:"
PGPASSWORD=app_pass psql -h localhost -p 5544 -U $DB_USER -d $DB_NAME -c 'SELECT version();'
```

---

Need help performing the upgrade? Ask and specify which path you prefer.
