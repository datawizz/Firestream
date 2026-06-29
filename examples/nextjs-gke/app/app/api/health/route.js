// Liveness/readiness endpoint for the container Healthcheck and the chart
// HTTP probes. Intentionally does NOT touch the database — it reports that the
// Next.js server process is up and serving, independent of DB availability.
export const dynamic = "force-dynamic";

export async function GET() {
  return Response.json({ status: "ok" });
}
