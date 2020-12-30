# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2020-12-30
### Added
- `Parser::add_action_parser` adding the ability to register `ActionParserFn` for custom `Actions`.

### Changed
- Exposed internal `COMMA_SEP_RE` and `QUOTED_STR_RE` helper regexes for use in custom `ActionParserFn`'s.


## [0.3.0] - 2020-12-14
### Added
- `apply_from_slice` transform function.

### Changed
- Updated to latest dependencies.
- Made linter suggested improvements.

### Fixed
- Added `Send + Sync` bounds to the `Action` trait allowing usage across threads.

## [0.2.0] - 2020-05-25
### Changed
- Converted to use `thiserror`.
- Reorganized code and exports.

## [0.1.1] - 2020-01-08
### Added
- Repository to Cargo.toml

## [0.1.0] - 2020-01-08
### Added
- Initial Release

[Unreleased]: https://github.com/rust-playground/proteus/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/rust-playground/proteus/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/rust-playground/proteus/compare/da422a5dd82c9cca612c864a7d9905992bce8281...v0.3.0
[0.2.0]: https://github.com/rust-playground/proteus/compare/e6563929efc6cefab9a7fc086a0b129f4690b94f...da422a5dd82c9cca612c864a7d9905992bce8281
[0.1.1]: https://github.com/rust-playground/proteus/compare/606709bc2d10236b8bb59da7034c98a6f4fc1f3f...e6563929efc6cefab9a7fc086a0b129f4690b94f
[0.1.0]: https://github.com/rust-playground/proteus/commit/606709bc2d10236b8bb59da7034c98a6f4fc1f3f