/**
 * SEED DATA
 * Optional test data for development
 * This creates example products and prices for local testing
 */

-- Insert example products
insert into products (id, active, name, description, image, metadata)
values
  (
    'prod_basic',
    true,
    'Basic',
    'Perfect product for small businesses.',
    null,
    '{"price_card_variant": "basic", "generated_images": 10, "image_editor": "basic", "support_level": "email"}'::jsonb
  ),
  (
    'prod_pro',
    true,
    'Pro',
    'Perfect product for growing or large businesses.',
    null,
    '{"price_card_variant": "pro", "generated_images": 100, "image_editor": "pro", "support_level": "live"}'::jsonb
  ),
  (
    'prod_enterprise',
    true,
    'Enterprise',
    'Perfect product for enterprises.',
    null,
    '{"price_card_variant": "enterprise", "image_editor": "pro", "support_level": "live"}'::jsonb
  )
on conflict (id) do nothing;

-- Insert example prices
insert into prices (id, product_id, active, description, unit_amount, currency, type, interval, interval_count, trial_period_days, metadata)
values
  -- Basic monthly
  (
    'price_basic_month',
    'prod_basic',
    true,
    'Basic plan - Monthly',
    500,
    'usd',
    'recurring',
    'month',
    1,
    null,
    '{}'::jsonb
  ),
  -- Basic yearly
  (
    'price_basic_year',
    'prod_basic',
    true,
    'Basic plan - Yearly',
    5000,
    'usd',
    'recurring',
    'year',
    1,
    null,
    '{}'::jsonb
  ),
  -- Pro monthly
  (
    'price_pro_month',
    'prod_pro',
    true,
    'Pro plan - Monthly',
    1000,
    'usd',
    'recurring',
    'month',
    1,
    null,
    '{}'::jsonb
  ),
  -- Pro yearly
  (
    'price_pro_year',
    'prod_pro',
    true,
    'Pro plan - Yearly',
    10000,
    'usd',
    'recurring',
    'year',
    1,
    null,
    '{}'::jsonb
  )
on conflict (id) do nothing;
