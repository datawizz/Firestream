// @ts-nocheck
import { default as __fd_glob_22 } from "../content/docs/guides/meta.json?collection=meta"
import { default as __fd_glob_21 } from "../content/docs/getting-started/meta.json?collection=meta"
import { default as __fd_glob_20 } from "../content/docs/development/meta.json?collection=meta"
import { default as __fd_glob_19 } from "../content/docs/architecture/meta.json?collection=meta"
import { default as __fd_glob_18 } from "../content/docs/meta.json?collection=meta"
import * as __fd_glob_17 from "../content/docs/guides/k3d-cluster.mdx?collection=docs"
import * as __fd_glob_16 from "../content/docs/guides/index.mdx?collection=docs"
import * as __fd_glob_15 from "../content/docs/guides/devcontainer.mdx?collection=docs"
import * as __fd_glob_14 from "../content/docs/guides/deployment.mdx?collection=docs"
import * as __fd_glob_13 from "../content/docs/getting-started/quickstart.mdx?collection=docs"
import * as __fd_glob_12 from "../content/docs/getting-started/installation.mdx?collection=docs"
import * as __fd_glob_11 from "../content/docs/getting-started/index.mdx?collection=docs"
import * as __fd_glob_10 from "../content/docs/development/testing.mdx?collection=docs"
import * as __fd_glob_9 from "../content/docs/development/index.mdx?collection=docs"
import * as __fd_glob_8 from "../content/docs/development/contributing.mdx?collection=docs"
import * as __fd_glob_7 from "../content/docs/architecture/rust-workspace.mdx?collection=docs"
import * as __fd_glob_6 from "../content/docs/architecture/python-etl.mdx?collection=docs"
import * as __fd_glob_5 from "../content/docs/architecture/overview.mdx?collection=docs"
import * as __fd_glob_4 from "../content/docs/architecture/infrastructure.mdx?collection=docs"
import * as __fd_glob_3 from "../content/docs/architecture/index.mdx?collection=docs"
import * as __fd_glob_2 from "../content/docs/odoo-nix-packaging-prd.md?collection=docs"
import * as __fd_glob_1 from "../content/docs/index.mdx?collection=docs"
import * as __fd_glob_0 from "../content/docs/PRD-firestream-vib.md?collection=docs"
import { server } from 'fumadocs-mdx/runtime/server';
import type * as Config from '../source.config';

const create = server<typeof Config, import("fumadocs-mdx/runtime/types").InternalTypeConfig & {
  DocData: {
  }
}>({"doc":{"passthroughs":["extractedReferences"]}});

export const docs = await create.doc("docs", "content/docs", {"PRD-firestream-vib.md": __fd_glob_0, "index.mdx": __fd_glob_1, "odoo-nix-packaging-prd.md": __fd_glob_2, "architecture/index.mdx": __fd_glob_3, "architecture/infrastructure.mdx": __fd_glob_4, "architecture/overview.mdx": __fd_glob_5, "architecture/python-etl.mdx": __fd_glob_6, "architecture/rust-workspace.mdx": __fd_glob_7, "development/contributing.mdx": __fd_glob_8, "development/index.mdx": __fd_glob_9, "development/testing.mdx": __fd_glob_10, "getting-started/index.mdx": __fd_glob_11, "getting-started/installation.mdx": __fd_glob_12, "getting-started/quickstart.mdx": __fd_glob_13, "guides/deployment.mdx": __fd_glob_14, "guides/devcontainer.mdx": __fd_glob_15, "guides/index.mdx": __fd_glob_16, "guides/k3d-cluster.mdx": __fd_glob_17, });

export const meta = await create.meta("meta", "content/docs", {"meta.json": __fd_glob_18, "architecture/meta.json": __fd_glob_19, "development/meta.json": __fd_glob_20, "getting-started/meta.json": __fd_glob_21, "guides/meta.json": __fd_glob_22, });