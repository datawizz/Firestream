# Supabase Database Migrations

This directory contains database migrations for the authentication and subscription management system.

## Migration Files

Migrations are numbered sequentially and should be applied in order:

1. **001_init.sql** - Initial schema setup
   - Creates custom types (pricing_type, pricing_plan_interval, subscription_status)
   - Creates tables (users, customers, products, prices, subscriptions)

2. **002_rls.sql** - Row Level Security policies
   - Enables RLS on all tables
   - Sets up security policies for data access control

3. **003_triggers.sql** - Database triggers
   - Auto-creates user profiles on signup
   - Syncs auth metadata to user table

4. **004_realtime.sql** - Realtime subscriptions
   - Configures realtime publication for products and prices

## Seed Data

The `seed.sql` file contains optional test data for development:
- Example products (Basic, Pro, Enterprise)
- Example prices (monthly and yearly plans)

## Usage

### Apply Migrations

From the project root:

```bash
pnpm db:migrate
```

### Generate TypeScript Types

After applying migrations, generate TypeScript types:

```bash
pnpm db:generate-types
```

This will create `src/types/database.ts` with type-safe database types.

### Seed Development Data

To load test data:

```bash
pnpm db:seed
```

## Stripe Integration

Stripe products and prices are synced to the database via webhooks in production. For local development:

1. Use the `stripe-fixtures.json` file to create test products in Stripe
2. Run the Stripe CLI to forward webhooks to your local environment
3. Products will automatically sync to your Supabase database

See `../stripe-fixtures.json` for fixture definitions.

## Schema Overview

### Users Table
- Stores user profiles linked to Supabase Auth
- Includes billing address and payment method
- RLS: Users can only view/update their own data

### Customers Table
- Maps users to Stripe customer IDs
- Private table - no direct user access
- Managed by server-side code only

### Products Table
- Synced from Stripe via webhooks
- Public read-only access
- Includes metadata for feature configuration

### Prices Table
- Pricing information for products
- Supports one-time and recurring billing
- Public read-only access

### Subscriptions Table
- User subscription records
- Synced from Stripe via webhooks
- RLS: Users can only view their own subscriptions

## Development Workflow

1. Make changes to migration files (always create new files, never edit existing ones)
2. Apply migrations: `pnpm db:migrate`
3. Generate types: `pnpm db:generate-types`
4. Update application code to use new types
5. Test with seed data: `pnpm db:seed`

## Production Deployment

Migrations are automatically applied when deploying to Supabase. Ensure:
- All migration files are committed to version control
- Migrations are tested in staging environment first
- Breaking changes are handled with proper versioning

## Troubleshooting

### Migration Errors

If a migration fails:
1. Check the Supabase dashboard for error details
2. Verify the migration file syntax
3. Ensure migrations are applied in order
4. Check for conflicts with existing schema

### Type Generation Errors

If type generation fails:
1. Ensure Supabase CLI is installed: `pnpm add -D supabase`
2. Verify your Supabase project is configured
3. Check that migrations have been applied successfully

### RLS Policy Issues

If users can't access data:
1. Verify RLS policies are correctly defined
2. Check that `auth.uid()` matches the user ID
3. Test policies in the Supabase SQL editor
