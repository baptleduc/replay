# Contributing to Replay

Thanks for wanting to contribute!

This document explains how to propose changes (bug fixes, new features, documentation, tests) in a way that improve code quality.


## Before you start

1. Read the `README.md` and any project documentation.
2. Search existing issues and pull requests, your bug or idea may already be discussed.
3. For large changes (major features, big refactors) open an issue or discussion first to agree on the approach.


## Bug reports and feature requests

Both bug reports and feature request, big or small, are welcome.

Before submitting a feature request, please check if an open issue already exists. If this is not the case, submit a feature request. Describe your use case, why you need this feature and why this feature is important for `replay`.

Before filing a bug report, please check if an open issue already exists. If this is not the case, submit a new bug report. If you're not sure if something is a bug or not, feel free to file a bug report anyway.


## Contribution workflow (Pull Requests)

GitHub Pull Requests (PRs) are the main way to contribute. We use the **fork-and-pull model**: you push changes to your personal fork, then open a PR into this repository.

### Before opening a PR
- Search existing and closed PRs maybe someone already worked on the same idea. If so, you can help by reviewing, testing, or reviving it.
- Keep PRs focused: smaller PRs are easier to review and get merged faster. If your contribution spans multiple concerns, split it into several PRs.

### How to open a PR
1. **Fork** the repository.
2. **Create a branch** with a clear name, e.g. `fix/bug-short-desc` or `feat/add-xyz`.
3. Make your changes and include tests where appropriate.
4. Format code with `cargo fmt`; use the pre-commit hook to automate this.
5. Run `cargo clippy` and `cargo test` locally to fix issues before opening a PR.
6. **Commit** using the required conventions.
   - When addressing review feedback, prefer `git commit --fixup <commit>` so reviewers see incremental changes.
   - Don’t squash or rewrite history after review — wait until a maintainer asks you to.
7. **Open a Pull Request** against the main branch (usually `main` or `master`) and include:
   - Purpose of the change.
   - Implementation details and rationale.
   - Possible risks, regressions, and semantic-versioning considerations.
   - How to test the change locally.
8. Leave “Allow edits from maintainers” enabled — this helps maintainers finalize your PR if small adjustments are needed.

### During review
- Respond quickly to comments to avoid stalled PRs.
- If your PR hasn’t received attention in a while, you may politely ping a maintainer or mention it in Discussions.


## PR checklist

* [ ] I have read the `README` and searched existing issues/PRs.
* [ ] My PR targets the correct branch.
* [ ] I wrote or updated tests where necessary.
* [ ] I added rustdoc comments for public items if needed.
* [ ] `cargo fmt` and `cargo test` pass locally.
* [ ] `cargo clippy` passes locally (no warnings).
* [ ] Commit messages follow the required format.
* [ ] I described the purpose of the PR and how to test it.


## Code style

* Format code with `rustfmt` (`cargo fmt`) before committing. The pre-commit hook will do this automatically if enabled.
* We treat warnings as errors in CI. Fix warnings locally before opening a PR.
* Prefer small, focused PRs and avoid adding unnecessary dependencies.

## Working with git
### Git hooks (recommended)

We provide a pre-configured set of Git hooks shipped in the repository under `.githooks/` to help contributors keep the codebase consistent. To enable them locally run:

```sh
# enable the repository-provided hooks
git config core.hooksPath .githooks
```

**What the hooks do (recommended defaults)**

* `pre-commit`: automatically runs `cargo fmt --all` and stages formatted files so commits are properly formatted.



> Note: hooks are optional for contributors but strongly recommended. Don’t worry if you forget — CI will catch it for you.”

### Commit conventions

This repository enforces a simple conventional-commit-like subject format in CI. The accepted types are:

```
feat, fix, build, chore, doc, style, refactor, test, perf
```

**Subject format**:

```
<type>(optional-scope)?: <short description>
```

Examples:

* `feat(auth): add token refresh`
* `fix: handle invalid input`
* `docs: improve README examples`


### Commit management after review

When addressing review feedback, please use **fixup commits** instead of rewriting history yourself:

```sh
git commit --fixup=<commit-hash>
```

Otherwise the history of review changes is lost and for large PRs, it makes it difficult for the reviewer to follow them. It might also happen that you introduce regression and won't be able to recover them from previous commits.

Once reviewers approve your changes, follow these steps:

```sh
git fetch origin
git rebase -i --autosquash origin/main
```

Then push your branch with:
```sh
git push --force-with-lease
```

Finally, use a **squash merge** in github to cleanly integrate your commits.
## Tests & quality
Replay has a test suite that you can run with `cargo test`. Ideally, we'd like pull requests to include tests where they make sense. For example, when fixing a bug, add a test that would have failed without the fix.

After you've made your change, make sure the tests pass in your development environment.


## Continuous Integration (CI)

This project uses a GitHub Actions workflow (rust-ci) that runs on pushes to main/master and on pull requests. CI enforces commit message format, code formatting, lints (warnings are treated as errors), documentation checks and the test suite across configured platforms.

Please ensure checks (format, lints, tests, docs) pass locally before opening a PR — CI will still run and may catch platform-specific issues.





## Documentation & examples

Contributions to docs and examples are highly appreciated:

* Update README.md for changes that affect the user experience (UX).
* Add examples under `examples/` when appropriate.

Rustdoc comments (///) are strongly encouraged for public APIs.


## Thank you

Thanks for contributing — code, issues, docs, and tests make the project better for everyone!
