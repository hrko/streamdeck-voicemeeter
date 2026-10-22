# Dependency updates

Run `mise run deps:audit` to verify publication dates of currently pinned tools
and direct npm dependencies without changing dependency versions. The report is
saved to `docs/dependency-age-report.json`, including source URLs and failures.

Run `mise run deps:update` to update the project:

1. Re-resolve Node, pnpm and Rust selectors for Windows x64 using
   `mise lock --bump --minimum-release-age 7d`.
2. Verify locked Node, pnpm, Rust and the fixed mise bootstrap publication dates
   against official upstream endpoints. Unknown dates fail with a report.
3. Select stable npm versions within each current major, published at least seven
   days ago, and preserve exact package.json versions. The PI client stays pinned
   until its API is explicitly reviewed. Synchronize packageManager with mise.lock.
4. Install locked tools, resolve npm dependencies under the workspace's 10,080
   minute policy, run Cargo update and regenerate the TypeScript/font contracts.

Cargo **library** updates do not receive mise's waiting period. Review Cargo.lock
separately. Major updates, the Rust/Node/pnpm series, the PI client, the Rust SDK,
and the fixed mise bootstrap/action commits require explicit compatibility review.

After updates run `mise run check`, `mise run test`, and `mise run package`.
Review lockfiles, the age report, generated contracts, license exceptions and
behavior before committing. A failed update may leave partial changes for review;
the task does not reset or discard local changes. CI never invokes update tasks.

Do not broadly exempt packages from minimumReleaseAge. If an emergency requires
an exception, record the exact package version, reason and review in this file,
and remove any narrowly scoped exception after its purpose is fulfilled.

There are currently no release-age exceptions.
