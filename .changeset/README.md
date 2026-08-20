# Changesets

Every pull request that changes the published package should include a changeset.

Run `pnpm changeset`, select the appropriate semantic-version bump, and describe the
user-facing change. Pull requests opened by the repository owner publish a uniquely
versioned beta package after all CI checks pass. When a pull request containing a
changeset is merged into `main`, CI consumes the changeset and publishes the stable
package.
