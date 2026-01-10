import { source } from '@/lib/source';
import { notFound } from 'next/navigation';
import { DocsPage, DocsBody } from 'fumadocs-ui/page';
import defaultMdxComponents from 'fumadocs-ui/mdx';
import { APIPage } from '@/components/api-page';

export default async function Page(props: {
  params: Promise<{ slug?: string[] }>;
}) {
  const params = await props.params;
  const page = source.getPage(params.slug);
  if (!page) notFound();

  // Handle OpenAPI pages
  if (page.data.type === 'openapi') {
    return (
      <DocsPage toc={page.data.toc} full>
        <DocsBody>
          <h1>{page.data.title}</h1>
          <APIPage {...page.data.getAPIPageProps()} />
        </DocsBody>
      </DocsPage>
    );
  }

  // Handle MDX pages
  const MDX = page.data.body;
  return (
    <DocsPage toc={page.data.toc}>
      <DocsBody>
        <MDX components={defaultMdxComponents} />
      </DocsBody>
    </DocsPage>
  );
}

export function generateStaticParams() {
  return source.generateParams();
}

export async function generateMetadata(props: {
  params: Promise<{ slug?: string[] }>;
}) {
  const params = await props.params;
  const page = source.getPage(params.slug);
  if (!page) notFound();

  return {
    title: page.data.title,
    description: page.data.description,
  };
}
