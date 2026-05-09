# Contributing

Contributions are welcome. Please open an issue before starting significant work so we can discuss the approach first.

## Getting Started

```
git clone https://github.com/smythg4/rust-ynab
cd rust-ynab
cargo build
```

Run the test suite:

```
cargo test
```

## Guidelines

**Code style** — All code must be formatted with `rustfmt`. The CI pipeline will reject unformatted code. Run `cargo fmt` before pushing.

**Tests** — New endpoints and bug fixes must include tests. The existing tests in `src/ynab/*.rs` show the patterns to follow using [wiremock](https://github.com/LukeMathWalker/wiremock-rs).

**Doc comments** — Public types and functions require a doc comment. Private helpers and test functions do not.

**Commits** — Keep commits focused. One logical change per commit makes review and bisection easier.

**No generated code** — This library is hand-written to stay idiomatic. Do not use OpenAPI generators or similar tools.

## Adding an Endpoint

1. Add the method to the appropriate `src/ynab/*.rs` file, following the existing pattern for the HTTP verb
2. Add the corresponding unit test in the same file's `#[cfg(test)]` module
3. Add a row to the API Coverage table in `README.md`

## Reporting Bugs

Open a GitHub issue with:
- The method you called
- The response or error you received
- What you expected to happen

If the bug involves sensitive account data, describe the shape of the response rather than the values.

## Versioning

This project follows [Semantic Versioning](https://semver.org). Breaking changes to public types or method signatures require a major version bump.

## License

By contributing you agree that your contributions will be licensed under the same [MIT License](LICENSE) as the project.
