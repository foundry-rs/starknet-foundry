# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `warp` cheatcode
- `roll` cheatcode
- `prank` cheatcode

### Changed

- `declare` return type to `starknet::ClassHash`, doesn't return a `Result`
- `PreparedContract` `class_hash` changed to `starknet::ClassHash`
- `deploy` return type to `starknet::ContractAddress`

### Fixed

- Using the same cairo file names as corelib files no longer fails test execution

## [0.1.1] - 2023-07-26

### Fixed

- `class_hash`es calculation
- Test collection

## [0.1.0] - 2023-07-19

### Added

- Initial release