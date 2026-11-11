# Validation and releases

The package version lives in `Cargo.toml`. Publication to crates.io and the release tag are
automated by CI; the workflow never bumps versions.

## Local validation

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked --release
cargo run --locked --example audit > audit.json
cargo package --locked --list      # data/minecraft-v1.json must be listed
```

The release automation has its own regression tests (no token, no network; HTTP and cargo are
mocked):

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .github/tests/release-workflow.ps1
```

They check the publication gate for every event, branch and repository combination, the
`publish = false` switch, registry responses (existing and missing versions, errors), the commit
chosen for the tag and that an existing tag is left unchanged.

## Continuous integration

[`.github/workflows/ci.yml`](../.github/workflows/ci.yml) validates pushes, pull requests and
manual dispatches (job `check`: release tests, fmt, clippy, debug and release tests, audit,
package).

The `publish` job runs only for pushes to `main` in `Sol1de/mc-mappings`, after `check` passes.
Pull requests, other branches, forks and manual dispatches only validate. It is the only job with
`contents: write`. Publication jobs share the `crates-io-publish` queue (`queue: max`,
`cancel-in-progress: false`): one runs at a time, pending ones are not replaced.

1. `.github/scripts/check-published-version.ps1` reads the package version. With
   `publish = false` in `Cargo.toml` it stops there: nothing is published or tagged and the job
   succeeds. Otherwise it asks crates.io: an existing version (even yanked) is skipped, a missing
   one is published; registry or network errors fail the job.
2. `cargo publish --locked --registry crates-io` runs with the `CARGO_REGISTRY_TOKEN` secret,
   exposed to this step only; the step fails if the secret is missing.
3. `.github/scripts/create-release-tag.ps1` creates the lightweight tag `v<version>` on the
   published commit with the built-in `GITHUB_TOKEN`. If the version was already published but
   its tag is missing, the commit is read from `.cargo_vcs_info.json` in the published archive, so
   a later commit keeping the same version is never tagged. Existing tags are never moved or
   forced.

To retry, rerun the failed jobs of the original push run: an uploaded version is skipped and a
missing tag is recovered. A failed publication creates no tag.

## Licence, credentials and switch

The crate is MIT-licensed (`LICENSE`). The runtime names bundled in `data/` are published as-is,
with the notice in the [README](../README.md#licence) (sources and their terms per configuration
in `data/minecraft-v1.provenance.json`).

The workflow uses a crates.io API token stored as the repository Actions secret
`CARGO_REGISTRY_TOKEN`. The first publication needs the `publish-new` scope; afterwards replace the
token with one restricted to `publish-update` for `mc-mappings`, and renew it before it expires.
Never commit or log the token. Tags are created with the built-in `GITHUB_TOKEN`: no tag ruleset
may block GitHub Actions from creating `v*` tags. Protect `main` with required checks to control
who can release.

To pause publication, add `publish = false` to `[package]` in `Cargo.toml`: the publish job then
succeeds without publishing or tagging. Remove it to resume.

## Preparing the next release

1. Bump `package.version` in `Cargo.toml` and refresh `Cargo.lock` (`cargo check`). Published
   versions cannot be overwritten.
2. Update the documentation and, for new tables, follow [coverage changes](README.md#coverage-changes-and-regression-checks).
3. Run the local validation above and `cargo publish --dry-run --locked`.
4. Merge to `main`.
