# FORK.md

## Firestream: Open Source continuity for Bitnami

**Fork Date:** August 2, 2025
**Original Repository:** https://github.com/bitnami/containers
**Fork Commit Hash:** `[f305c2dc77c0a97a4fdc545905af40fb5c361003]`
**License:** Apache 2.0 (maintained)

---

## Why This Fork Exists

On August 28th, 2025, Bitnami will paywall open source software:

- Community users get only a reduced set of "hardened" images
- All versioned tags moved to an unmaintained legacy repository
- Production use requires paid "Bitnami Secure Images"
- Opaque security procedures replace transparent builds

**Firestream** is adopting the Bitnami legacy.

## Our Principles

### 🔥 Open Source Without Gatekeepers
No artificial restrictions. No tiered access. No corporate control.

### 🔥 Reproducible Nix Builds
Every image built from source. Read the Nix, understand the build. No black boxes.

### 🔥 Full Transparency
- Auditable build processes
- No opaque OCI locations
- No Docker Hub lock-in
- Everything builds from Debian

### 🔥 Community Owned
Never pay a corporation to access open source. The community controls its tools.

## What's Changing

1. **Build System:** Bitnami's system → Nix
2. **Distribution:** Docker Hub dependency → Any local OCI engine or repo
3. **Security:** Community-driven updates

## What Stays the Same

- Apache 2.0 license
- Helm chart compatibility
- Deployment compatibility
- Container compatibility

## Technical Details

Hard forked from `bitnami/containers` commit `[f305c2dc77c0a97a4fdc545905af40fb5c361003]` on August 2, 2025, in response to Bitnami's catalog changes.

Fork triggered by:
- "Secure Images" paywall
- Removal of public version tags
- Opaque OCI registry locations
- Docker Hub over-reliance
- Non-reproducible (just repeatable) builds

## Join Us

Help build truly open infrastructure:
- Contribute Nix expressions
- Maintain the images you use
- Shape the project's future

**Let this be the new normal:** Open, auditable, reproducible infrastructure the community can trust, verify, and control.

---

*Original announcement: [\[Bitnami Secure Images\]](https://github.com/bitnami/containers/issues/83267)*
*Contributing: See CONTRIBUTING.md*
