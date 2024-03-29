# Changelog

## v0.6.0

### New features

* Added required `primary_key` parameter to the `#[model]` attribute. This enables the possibility of setting the primary key name.

### Changes

* Improved error handling. The `DatabaseModelModifier` now returns an `anyhow::Result` instead of a plain `bool`
* Improved internal structure
* proc-macro implementation now uses the anyhow re-exported crate from naphtha

### Bugfixes

* Fixed wrong import statements in proc macro crate
* Added missing diesel::Table use statement

## v0.5.0

### New features

* `diesel::Pg` connection is now implemented

### Changes

* Updated examples to be able to use feature flags for the database selection

## v0.4.1

### New features

* None

### Changes

* Updated `barrel` dependency to `0.7.0`
* Bugfix, the `barrel` implementation was specifically implemented for the example
* Removed the requirement of `log` crate as dependency when using this library

### Removals and deprecations

None

## v0.4.0

### New features

* Added support for `diesel::MysqlConnection` 
* Added support for `MySql` barrel backend
* Added `.tmuxp.yaml` file for better development
* Added `docker-compose.yml` file with `MySQL` database for easy testing

### Changes

* Internal macro definition changes due to conflicting `use` statements

### Removals and deprecations

None

## v0.3.1

### New features

* Added `DatabaseInsertHandler` and `DatabaseRemoveHandler` to have the possibility to extend the CRUD model.

### Changes

None

### Removals and deprecations

None

## v0.2.0

### New features

* Added `QueryByProperties` trait that enables to query the model by a specific property value. It only returns models that matches the exact value.
* Implemented `QueryByProperties` for `SQLite3`

### Changes

* The `custom` method has been updated. Calling it now requires an anonymous type as second generic parameter.
* The example for `SQLite3` has been updated.

### Removals and deprecations

None

## v0.1.0

### New features

* Added `barrel` integration for `SQLite3`

* Added `SQLite3` support, backed by `diesel`

* Added an example for `SqliteConnection`

* Added possibility for `custom` queries

* Added possibility to change the model before and after updating it in the database by implementing the `DatabaseUpdateHandler` trait.

### Changes

### Removals and deprecations

None
