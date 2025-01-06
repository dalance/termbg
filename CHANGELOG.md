# Change Log

## [Unreleased](https://github.com/dalance/termbg/compare/v0.6.2...Unreleased) - ReleaseDate

## [v0.6.2](https://github.com/dalance/termbg/compare/v0.6.1...v0.6.2) - 2025-01-06

## [v0.6.1](https://github.com/dalance/termbg/compare/v0.6.0...v0.6.1) - 2024-11-15

## [v0.6.0](https://github.com/dalance/termbg/compare/v0.5.2...v0.6.0) - 2024-10-21

* [Fixed] Stdin is blocked [#29](https://github.com/dalance/termbg/pull/29)

## [v0.5.2](https://github.com/dalance/termbg/compare/v0.5.1...v0.5.2) - 2024-10-11

## [v0.5.1](https://github.com/dalance/termbg/compare/v0.5.0...v0.5.1) - 2024-09-13

## [v0.5.0](https://github.com/dalance/termbg/compare/v0.4.4...v0.5.0) - 2024-03-06

* [Breaking Changed] feat: upgrade crossterm to 0.27.0 [#22](https://github.com/dalance/termbg/pull/22)
* [Changed] Remove special case for VSCode [#21](https://github.com/dalance/termbg/pull/21)

## [v0.4.4](https://github.com/dalance/termbg/compare/v0.4.3...v0.4.4) - 2023-12-06

* [Fixed] Do not set the terminal to raw mode if it isn't actually a terminal [#19](https://github.com/dalance/termbg/pull/19)

## [v0.4.3](https://github.com/dalance/termbg/compare/v0.4.2...v0.4.3) - 2023-03-02

## [v0.4.2](https://github.com/dalance/termbg/compare/v0.4.1...v0.4.2) - 2023-03-02

## [v0.4.1](https://github.com/dalance/termbg/compare/v0.4.0...v0.4.1) - 2022-05-25

* [Added] emacs detection

## [v0.4.0](https://github.com/dalance/termbg/compare/v0.3.0...v0.4.0) - 2022-01-25

* [Breaking Changed] drop Terminal::RxvtCompatible
* [Added] latency measurement function

## [v0.3.0](https://github.com/dalance/termbg/compare/v0.2.4...v0.3.0) - 2021-05-28

* [Fixed] unexpected stdin lock [#5](https://github.com/dalance/termbg/issues/5)

## [v0.2.4](https://github.com/dalance/termbg/compare/v0.2.3...v0.2.4) - 2021-05-20

* [Fixed] Panic at response check thread

## [v0.2.3](https://github.com/dalance/termbg/compare/v0.2.2...v0.2.3) - 2021-05-19

* [Fixed] Hung at VSCode

## [v0.2.2](https://github.com/dalance/termbg/compare/v0.2.1...v0.2.2) - 2021-05-18

## [v0.2.1](https://github.com/dalance/termbg/compare/v0.2.0...v0.2.1) - 2021-05-18

* [Fixed] Bytes leak [#4](https://github.com/dalance/termbg/issues/4)

## [v0.2.0](https://github.com/dalance/termbg/compare/v0.1.0...v0.2.0) - 2020-11-10

* [Changed] Return type from `Option` to `Result`
* [Changed] Remove rxvt detection ( rxvt is XtermCompatible )
* [Added] Windows Terminal detection
* [Added] Windows console support
