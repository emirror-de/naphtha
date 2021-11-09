<!--
SPDX-FileCopyrightText: 2021 Lewin Probst, M.Sc. <info@emirror.de>

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# naphtha

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-informational?style=flat-square)](COPYRIGHT.md)

[![Crates.io](https://img.shields.io/crates/v/naphtha.svg)](https://crates.io/crates/naphtha) [![docs.rs](https://img.shields.io/docsrs/naphtha?style=flat-square)](https://docs.rs/naphtha) **naphtha**

[![Crates.io](https://img.shields.io/crates/v/naphtha-proc-macro.svg)](https://crates.io/crates/naphtha-proc-macro) **naphtha-proc-macro**

Please checkout the [documentation page](https://docs.rs/naphtha) for information and examples (also see examples folder in [naphtha](./naphtha/examples))

If you have questions, want to contribute or have any other type of request, your invited to create an issue or visit the [openprobst.dev](https://openprobst.dev) discord server.

[![](https://img.shields.io/discord/855726181142495242?color=154683&label=discord&style=flat-square)](https://discord.gg/nx7YtsjEbT)

## Roadmap

- [x] Connect to database using a wrapper, defining the base for interchangeable Databases
- [x] Implement support for `diesel::SQLiteConnection`
- [x] Create traits to enable database interaction before and after the execution of the `insert`, `update` or `remove` transactions
- [x] Implement `query_by_{property}` for querying models from the database using an exact match of the given property
- [x] Thread safe sharing of the database connection
- [x] Integrate `barrel` crate for writing migrations in Rust, available at runtime
- [x] Implement support for `diesel::MySqlConnection`
- [ ] Implement support for `diesel::PgConnection`
- [ ] More databases!!!

## Contributing

Unless explicitly stated, any contribution intentionally submitted
for inclusion in this project, as defined in the Apache-2.0 license, shall be
dual licensed as below, without any additional terms or conditions.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT)

at your option.
