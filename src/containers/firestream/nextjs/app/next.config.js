/**
 * Firestream Next.js example — production config.
 *
 * `output: "standalone"` makes `next build` trace a minimal, self-contained
 * server into `.next/standalone` (server.js + a pruned node_modules). The
 * Firestream container module copies that tree out of the build and runs it
 * with `node server.js`, so the runtime image carries no npm/install step.
 *
 * @type {import('next').NextConfig}
 */
module.exports = {
  output: "standalone",
  // The standalone server honours HOSTNAME/PORT from the environment; the
  // chart sets PORT=3000 and HOSTNAME=0.0.0.0.
  poweredByHeader: false,
};
