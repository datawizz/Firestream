import { Footer, Layout, Navbar } from 'nextra-theme-docs'
import { Head } from 'nextra/components'
import { getPageMap } from 'nextra/page-map'
import 'nextra-theme-docs/style.css'

export const metadata = {
  title: {
    default: 'Multi-Platform App Template',
    template: '%s | Multi-Platform App'
  },
  description: 'Documentation for the Multi-Platform App Template - build web, desktop, and iOS apps with shared TypeScript code.',
  icons: {
    icon: '/favicon.svg'
  }
}

const navbar = (
  <Navbar
    logo={<strong>Multi-Platform App</strong>}
    projectLink="https://github.com/your-org/multi-platform-app"
  />
)

const footer = <Footer>MIT {new Date().getFullYear()} Multi-Platform App Template</Footer>

export default async function RootLayout({ children }) {
  return (
    <html lang="en" dir="ltr" suppressHydrationWarning>
      <Head />
      <body>
        <Layout
          navbar={navbar}
          footer={footer}
          pageMap={await getPageMap()}
          docsRepositoryBase="https://github.com/your-org/multi-platform-app/tree/main/docs"
          editLink="Edit this page on GitHub"
        >
          {children}
        </Layout>
      </body>
    </html>
  )
}
