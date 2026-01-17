import { loader, multiple } from 'fumadocs-core/source';
import { toFumadocsSource } from 'fumadocs-mdx/runtime/server';
import { openapiPlugin, openapiSource } from 'fumadocs-openapi/server';
import { docs, meta } from '@source';
import { openapi } from './openapi';

// Combined MDX + OpenAPI documentation source
export const source = loader(
  multiple({
    docs: toFumadocsSource(docs, meta),
    openapi: await openapiSource(openapi, {
      baseDir: 'api',
    }),
  }),
  {
    baseUrl: '/docs',
    plugins: [openapiPlugin()],
  },
);
