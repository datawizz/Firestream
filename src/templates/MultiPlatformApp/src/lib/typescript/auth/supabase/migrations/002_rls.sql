/**
 * ROW LEVEL SECURITY (RLS) POLICIES
 * Enable RLS and create policies for secure data access
 */

-- Enable RLS on all tables
alter table users enable row level security;
alter table customers enable row level security;
alter table products enable row level security;
alter table prices enable row level security;
alter table subscriptions enable row level security;

-- USERS: Users can view and update their own data
create policy "Can view own user data." on users for select using (auth.uid() = id);
create policy "Can update own user data." on users for update using (auth.uid() = id);

-- CUSTOMERS: Private table - no public access
-- No policies as this is a private table that the user must not have access to.

-- PRODUCTS: Public read-only access
create policy "Allow public read-only access." on products for select using (true);

-- PRICES: Public read-only access
create policy "Allow public read-only access." on prices for select using (true);

-- SUBSCRIPTIONS: Users can only view their own subscriptions
create policy "Can only view own subs data." on subscriptions for select using (auth.uid() = user_id);
