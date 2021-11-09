# Changelog

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