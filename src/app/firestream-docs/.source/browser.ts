// @ts-nocheck
import { browser } from 'fumadocs-mdx/runtime/browser';
import type * as Config from '../source.config';

const create = browser<typeof Config, import("fumadocs-mdx/runtime/types").InternalTypeConfig & {
  DocData: {
  }
}>();
const browserCollections = {
  docs: create.doc("docs", {"PRD-firestream-vib.md": () => import("../content/docs/PRD-firestream-vib.md?collection=docs"), "index.mdx": () => import("../content/docs/index.mdx?collection=docs"), "odoo-nix-packaging-prd.md": () => import("../content/docs/odoo-nix-packaging-prd.md?collection=docs"), "architecture/index.mdx": () => import("../content/docs/architecture/index.mdx?collection=docs"), "architecture/infrastructure.mdx": () => import("../content/docs/architecture/infrastructure.mdx?collection=docs"), "architecture/overview.mdx": () => import("../content/docs/architecture/overview.mdx?collection=docs"), "architecture/python-etl.mdx": () => import("../content/docs/architecture/python-etl.mdx?collection=docs"), "architecture/rust-workspace.mdx": () => import("../content/docs/architecture/rust-workspace.mdx?collection=docs"), "development/contributing.mdx": () => import("../content/docs/development/contributing.mdx?collection=docs"), "development/index.mdx": () => import("../content/docs/development/index.mdx?collection=docs"), "development/testing.mdx": () => import("../content/docs/development/testing.mdx?collection=docs"), "getting-started/index.mdx": () => import("../content/docs/getting-started/index.mdx?collection=docs"), "getting-started/installation.mdx": () => import("../content/docs/getting-started/installation.mdx?collection=docs"), "getting-started/quickstart.mdx": () => import("../content/docs/getting-started/quickstart.mdx?collection=docs"), "guides/deployment.mdx": () => import("../content/docs/guides/deployment.mdx?collection=docs"), "guides/devcontainer.mdx": () => import("../content/docs/guides/devcontainer.mdx?collection=docs"), "guides/index.mdx": () => import("../content/docs/guides/index.mdx?collection=docs"), "guides/k3d-cluster.mdx": () => import("../content/docs/guides/k3d-cluster.mdx?collection=docs"), }),
};
export default browserCollections;