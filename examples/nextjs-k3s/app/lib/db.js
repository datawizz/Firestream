// Shared PostgreSQL connection pool for the Firestream Next.js example.
//
// The connection is assembled from libpq-style environment variables
// (PGHOST/PGPORT/PGUSER/PGPASSWORD/PGDATABASE) injected by the Helm chart from
// the bundled firestream-postgresql subchart. A fully-formed DATABASE_URL, if
// present, wins. `pg` is pure-JS, so it survives Next.js standalone tracing
// without native addons.
import pg from "pg";

const { Pool } = pg;

export function databaseUrl() {
  const url = process.env.DATABASE_URL;
  if (url && url.length > 0) {
    return url;
  }
  const host = process.env.PGHOST || "127.0.0.1";
  const port = process.env.PGPORT || "5432";
  const user = process.env.PGUSER || "postgres";
  const password = process.env.PGPASSWORD || "";
  const database = process.env.PGDATABASE || "postgres";
  const auth = password
    ? `${encodeURIComponent(user)}:${encodeURIComponent(password)}`
    : encodeURIComponent(user);
  return `postgresql://${auth}@${host}:${port}/${database}`;
}

let pool;

export function getPool() {
  if (!pool) {
    pool = new Pool({ connectionString: databaseUrl() });
  }
  return pool;
}
