# Upstream Notes

Reference upstream repositories:

- https://github.com/Syzzle07/SingularityMM
- https://github.com/Syzzle07/NMS-Mod-Manager

## Maintenance Policy

- PulsarMM is independently maintained.
- Upstream repositories are reference-only and are not treated as merge targets.
- Upstream changes are ported manually when useful, typically via targeted diffs/cherry-picks.

## Porting Workflow

1. Compare file contents against upstream snapshots.
2. Port isolated improvements with explicit commit messages.
3. Re-test Linux/SteamOS/Steam Deck behavior before release.
