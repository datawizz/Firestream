# Scripts Directory

Build and automation scripts for the multi-platform app template.

## Scripts

### proto-generate.sh
Generates TypeScript types from Protocol Buffer definitions.

**Usage**:
```bash
# From project root
make proto-generate

# Or directly
bash scripts/proto-generate.sh
```

**Requirements**:
- Protocol Buffer definitions in `src/lib/proto/`
- proto-types package installed
- protoc with @bufbuild/protoc-gen-es

### Permissions

If you encounter permission errors, make scripts executable:

```bash
chmod +x scripts/*.sh
```
