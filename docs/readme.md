# Mirage Developer Documentation

This folder contains documentation that is primarily intended for people working on (rather than with) Mirage.
This documentation is intended to be living thing!
It will and should evolve along with the project.

## Contributing

The development of Mirage is done through pull requests against the `main` branch.
We strive to maintain a clean linear history for the `main` branch, we rebase all PRs before merging and expect PRs to be rebased against the latest `main` branch.

An explicit goal is to ensure that all commits in the `main` branch pass the test suite of that commit, in other words `just test` must always succeed.
To enforce this the CI run the tests against each new commit when submitting a PR, if you see a failure in the CI check for the details to find out which commit caused the issue.
Of course writing code requires iteration, a good rule of thumb is to write a first version while committing along the way, and to rework those commits in a second time using tools such as `git rebase --interactive` or [jj](https://steveklabnik.github.io/jujutsu-tutorial/).

## Code Style

This section describes the style we strive to enforce across the code base, and serves as a reference when arbitrary choices need to be made.

**Comments**:

Comment starts with a leading white space and a capital letter, like this:

```rs
// Our comment style
```

But **not** like this:

```rs
//We avoid this
// or this
//and this
```
