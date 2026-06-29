# Bake THIS example's Next.js app (./app) into the firestream-nextjs image.
#
# `vendoredApp.src` replaces the canonical Firestream app; `npmDepsHash` is the
# buildNpmPackage hash for ./app/package-lock.json. This app reuses the
# canonical lockfile, so the hash matches the one baked into the container
# module; if you change dependencies, rebuild once with lib.fakeHash and paste
# the reported `got:` hash here.
{ ... }:

{
  config.nextjs.vendoredApp = {
    src = ../app;
    npmDepsHash = "sha256-1BUzCAuAUshgvkL8Hxr3z9WYBhBh59nTO9XrImBYvZQ=";
  };
}
