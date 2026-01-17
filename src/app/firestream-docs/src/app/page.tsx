import Link from 'next/link';
import { HomeLayout } from 'fumadocs-ui/layouts/home';
import { baseOptions } from './layout.config';

export default function HomePage() {
  return (
    <HomeLayout {...baseOptions}>
      <main className="flex min-h-screen flex-col items-center justify-center p-24">
        <div className="max-w-2xl text-center">
          <h1 className="text-4xl font-bold mb-4">Firestream Documentation</h1>
          <p className="text-lg text-fd-muted-foreground mb-8">
            Serverless data warehouse - create-react-app for data engineering.
            Local K3D development that mirrors production.
          </p>
          <div className="flex gap-4 justify-center">
            <Link
              href="/docs"
              className="inline-flex items-center justify-center rounded-md bg-fd-primary px-6 py-3 text-sm font-medium text-fd-primary-foreground shadow transition-colors hover:bg-fd-primary/90"
            >
              Get Started
            </Link>
            <Link
              href="https://github.com/Cogent-Creation-Co/Firestream"
              className="inline-flex items-center justify-center rounded-md border border-fd-border px-6 py-3 text-sm font-medium transition-colors hover:bg-fd-accent"
            >
              GitHub
            </Link>
          </div>
        </div>
      </main>
    </HomeLayout>
  );
}
