# Contributing

## Setup Development Prerequisites

- [SQLite](https://www.sqlite.org) 3.39.2
- [Rust](https://www.rust-lang.org) 1.62.1-aarch64-apple-darwin
- [rust-bindgen](https://github.com/rust-lang/rust-bindgen) 0.60.1

## Fork on GitHub

Before you do anything else, login on [GitHub](https://github.com/) and [fork](https://help.github.com/articles/fork-a-repo/) this repository.

## Clone Your Fork Locally

Install [Git](https://git-scm.com/) and clone your forked repository locally.

```shell
$ git clone https://github.com/<your-account>/libgsqlite
```

## Play with Your Fork

Create a new feature branch and play with it.

```shell
$ git switch -c add-new-feature
```

The project uses [Semantic Versioning 2.0.0](http://semver.org/), but you don't have to update `Cargo.toml` as I will maintain release.

### Note: How to Update SQLite Binding

```shell
$ bindgen --default-macro-constant-type signed sqlite3ext.h -o sqlite3ext.rs
```

## Test Your Fork

```shell
$ cargo test -- --skip test_extension
```

If you have (1) set up Google Cloud, and (2) created the sample spreadsheet as described in the [README](README.md), you can run full test.

```shell
$ export LIBGSQLITE_GOOGLE_CLIENT_ID=...
$ export LIBGSQLITE_GOOGLE_CLIENT_SECRET=...
$ export LIBGSQLITE_GOOGLE_CLIENT_TEST_ID=https://docs.google.com/spreadsheets/d/...
$ export LIBGSQLITE_GOOGLE_CLIENT_TEST_SHEET=Sheet1
$ export LIBGSQLITE_GOOGLE_CLIENT_TEST_RANGE=A2:D7
$ cargo test
```

## Open a Pull Request

1. Commit your changes locally, [rebase onto upstream/master](https://github.com/blog/2243-rebase-and-merge-pull-requests), then push the changes to GitHub
   ```sh
   $ git push origin add-new-feature
   ```
2. Go to your fork on GitHub, switch to your feature branch, then click "Compare and pull request" button for review.

# References

- [Run-Time Loadable Extensions](https://www.sqlite.org/loadext.html)
- [The Virtual Table Mechanism Of SQLite](https://sqlite.org/vtab.html)