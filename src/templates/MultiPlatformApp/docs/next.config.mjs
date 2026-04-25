import nextra from 'nextra'

const withNextra = nextra({
  // Content served at root since this is a standalone docs site
})

export default withNextra({
  reactStrictMode: true
})
