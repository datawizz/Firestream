/**
 * DATABASE TRIGGERS
 * Automatically handle user profile creation and updates
 */

/**
 * This trigger automatically creates a user entry when a new user signs up via Supabase Auth.
 * It copies the full_name and avatar_url from the auth metadata.
 */
create function public.handle_new_user()
returns trigger as $$
begin
  insert into public.users (id, full_name, avatar_url)
  values (new.id, new.raw_user_meta_data->>'full_name', new.raw_user_meta_data->>'avatar_url');
  return new;
end;
$$ language plpgsql security definer;

create trigger on_auth_user_created
  after insert on auth.users
  for each row execute procedure public.handle_new_user();
