---
title: Contributing
sidebar_position: 15
---

The development of Miralis is done through pull requests against the `main` branch.
We strive to maintain a clean linear history for the `main` branch, we rebase all PRs before merging and expect PRs to be rebased against the latest `main` branch.

An explicit goal is to ensure that all commits in the `main` branch pass the test suite of that commit, in other words `just test` must always succeed.
To enforce this the CI runs the tests against each new commit when submitting a PR, if you see a failure in the CI check for the details to find out which commit caused the issue.
Of course writing code requires iteration, a good rule of thumb is to write a first version while committing along the way, and to rework those commits in a second time using tools such as `git rebase --interactive` or [jj](https://steveklabnik.github.io/jujutsu-tutorial/).

## Design

This section describes the design principles behind Miralis, and serve as guidelines when taking design decisions for the project as a whole.

**Correctness over completeness**:

Miralis strives to be correct first and foremost.
In particular, we always sacrifice completeness in favor of correctness when necessary.
This means that we rather Miralis to panic and report an error when encountering an unexpected system state (e.g. unsupported value in a configuration register), than to silently continue in an unsupported and unstable systems state.
When implementing a new feature it is encouraged to voluntarily _not_ support all possible edge cases and to report an error when encountering them (e.g. panicking with an assert).

**Ease of testing**:

Miralis should be easy to test and debug.
In particular, it should be easy to:
- Write unit tests in the same Rust source file.
  This include tests of low-level system code manipulating systems registers.
- Write integration tests to run in QEMU or another simulator.
  In particular, it should be easy to write a custom test firmware, or to run an existing firmware as part of the test suite.
- To run on real hardware.

We invest in the necessary tooling and documentation necessary to make that happen.

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

**Rust code**:

We always insert a blank line between two functions:

```rs
fn foo() {
    bar();
}

fn bar() { // See how there is a blank line before
    baz();
}
```

The only exception is within trait definitions for function _signatures_:

```rs
trait MyTrait {
    // This is OK
    fn foo();
    fn bar();

    // But this is **not** OK
    fn baz() {
        foo();
    }
    fn fuzz() { // See how there is no blank line above? This is **not** OK
        baz();
    }
}
```

**Markdown**:

We write markdown documentation with exactly one sentence per line, like this:

```md
We always have only one sentence per line.
This makes it easy to review changes as the diff shows exactly which sentences changed.
```

In particular we do **not** wrap sentences across multiple line in `.md` files (but we do in Rust comments!):

```md
We **never** wrap a sentence
across multiple lines.
```
