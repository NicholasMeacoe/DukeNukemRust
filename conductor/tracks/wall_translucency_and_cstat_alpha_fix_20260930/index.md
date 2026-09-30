# Track: Wall Translucency and Cstat Alpha Fix

- **Track ID**: `wall_translucency_and_cstat_alpha_fix_20260930`
- **Type**: Bugfix
- **Status**: Complete `[x]`
- **Spec**: [spec.md](./spec.md)
- **Plan**: [plan.md](./plan.md)

## Summary
Eliminates unwanted translucency on building facades, street walls, and sprites caused by conflating `cstat & 4` with translucency in `Wall` and `Sprite` methods.
