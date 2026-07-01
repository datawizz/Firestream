import { getPool } from "../lib/db";

// Always render on each request so the page reflects the live database.
export const dynamic = "force-dynamic";

export default async function Home() {
  let row = null;
  let error = null;
  try {
    const result = await getPool().query(
      "select version() as version, now() as now",
    );
    row = result.rows[0];
  } catch (err) {
    error = err.message;
  }

  return (
    <main style={{ maxWidth: 720, margin: "4rem auto", padding: "0 1.5rem" }}>
      <h1>Firestream × Next.js — GKE</h1>
      <p>
        This Next.js app was built into a production container and Helm chart by
        the Firestream Nix modules, with PostgreSQL provided by the bundled
        <code> firestream-postgresql </code> subchart.
      </p>
      {error ? (
        <pre style={{ color: "crimson", whiteSpace: "pre-wrap" }}>
          Database error: {error}
        </pre>
      ) : (
        <section
          style={{
            background: "#f5f5f5",
            borderRadius: 8,
            padding: "1rem 1.25rem",
          }}
        >
          <p style={{ margin: "0 0 0.5rem" }}>
            <strong>PostgreSQL:</strong> {row.version}
          </p>
          <p style={{ margin: 0 }}>
            <strong>Server time:</strong> {String(row.now)}
          </p>
        </section>
      )}
    </main>
  );
}
