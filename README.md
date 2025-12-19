# UTCA

image:https://img.shields.io/github/v/release/ippras/utca-source?label=Version&sort=semver[Version]
image:https://img.shields.io/github/license/ippras/utca-source?label=License[License, link=license]
image:https://img.shields.io/github/actions/workflow/status/ippras/utca-source/rust.yml?logo=github&label=Rust[Rust, link=https://github.com/ippras/utca-source/actions/workflows/rust.yml]
image:https://img.shields.io/github/actions/workflow/status/ippras/utca-source/pages.yml?logo=github&label=Github Pages[Github Pages, link=https://github.com/ippras/utca-source/actions/workflows/pages.yml]

Ultimate Triacylglycerol Calculation Application

[fips.ru](https://fips.ru/registers-doc-view/fips_servlet?DB=EVM&DocNumber=2024691787)

== Documentation

|===
|link:doc/en-US.adoc[English 🇺🇸] | link:doc/ru-RU.adoc[Русский 🇷🇺]
|===

== Version

* link:https://ippras.github.io/utca[title=Latest Release, window=_blank]
* link:https://ippras.github.io/utca/{version}[title=Release Version, window=_blank]

== Build

=== Web

[source,shell]
trunk build --release --filehash=false --public-url=https://ippras.github.io/utca

== Run

=== Native

[source,shell]
cargo run

=== Web (http://127.0.0.1:8080)

[source,shell]
trunk serve --release --filehash=false

==== Requirements

[source,shell]
rustup target add wasm32-unknown-unknown

==== Errors

integrity:

* `chrome://settings/clearBrowserData`
* `Дополнительные`
* `Изображения и другие файлы, сохраненные в кеше`
* `Удалить данные`

* Check dist index.html urls

== See Also

* link:https://byrdwell.com/Triacylglycerols/TAGbyMass1.htm[byrdwell.com]
* link:https://physics.nist.gov/cgi-bin/Compositions/stand_alone.pl[nist.gov, title=Atomic Weights and Isotopic Compositions for All Elements]

== Dedication

== GIT

git branch -d <branch_name>
git push origin -d <branch_name>

## Fix

### Local

`rustup install nightly-2025-10-29`

### Workflows

From

```yml
uses: dtolnay/rust-toolchain@nightly
with:
    target: wasm32-unknown-unknown
```

To

```yml
uses: dtolnay/rust-toolchain@master
with:
    target: wasm32-unknown-unknown
    toolchain: nightly-2025-10-28
```

## TODO
