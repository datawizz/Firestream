/**
 * REALTIME SUBSCRIPTIONS
 * Only allow realtime listening on public tables.
 * Products and prices are public data that can be subscribed to for real-time updates.
 * Subscriptions table is kept private for security.
 */
drop publication if exists supabase_realtime;
create publication supabase_realtime for table products, prices;
