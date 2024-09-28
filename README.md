[![progress-banner](https://backend.codecrafters.io/progress/sqlite/47c57dd9-82b7-4da4-9ddb-62ad5c21fa1d)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own SQLite" Challenge](https://codecrafters.io/challenges/sqlite).

In this challenge, you'll build a barebones SQLite implementation that supports
basic SQL queries like `SELECT`. Along the way we'll learn about
[SQLite's file format](https://www.sqlite.org/fileformat.html), how indexed data
is
[stored in B-trees](https://jvns.ca/blog/2014/10/02/how-does-sqlite-work-part-2-btrees/)
and more.

**Note**: You can try the challenge on [codecrafters.io](https://codecrafters.io).

# Sample Databases

To make it easy to test queries locally, we've added a sample database in the
root of this repository: `sample.db`.

This contains two tables: `apples` & `oranges`. You can use this to test your
implementation for the first 6 stages.

You can explore this database by running queries against it like this:

```sh
$ sqlite3 sample.db "select id, name from apples"
1|Granny Smith
2|Fuji
3|Honeycrisp
4|Golden Delicious
```

There are two other databases that you can use:

1. `superheroes.db`:
    - This is a small version of the test database used in the table-scan stage.
    - It contains one table: `superheroes`.
    - It is ~1MB in size.
1. `companies.db`:
    - This is a small version of the test database used in the index-scan stage.
    - It contains one table: `companies`, and one index: `idx_companies_country`
    - It is ~7MB in size.

These aren't included in the repository because they're large in size. You can
download them by running this script:

```sh
./download_sample_databases.sh
```

If the script doesn't work for some reason, you can download the databases
directly from
[codecrafters-io/sample-sqlite-databases](https://github.com/codecrafters-io/sample-sqlite-databases).

## Installation and run

To run the application first clone the repo and run the application using `./run.sh` script

### Examples

#### Get DB Info

```shell
$ ./run.sh sample.db .dbinfo

database page size:  4096
write format:        1
read format:         1
reserved bytes:      64
file change counter: 5
database page count: 0
freelist page count: 0
schema cookie:       2
schema format:       4
default cache size:  0
autovacuum top root: 0
incremental vacuum:  0
text encoding:       1 (utf-8)
user version:        0
application id:      0
software version:    3034000
number of tables:    3

```

#### Get Tables

```shell
$ ./run.sh sample.db .tables

apples sqlite_sequence oranges 
```

#### Query Data

```shell
$ ./run.sh sample.db "select * from apples"

1|Granny Smith|Light Green
2|Fuji|Red
3|Honeycrisp|Blush Red
4|Golden Delicious|Yellow
```

```shell
$ ./run.sh sample.db "select * from apples where color != 'Yellow' limit 2"

1|Granny Smith|Light Green
2|Fuji|Red
```