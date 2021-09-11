# Changelog

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