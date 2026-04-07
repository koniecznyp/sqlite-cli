# SQLite db reader cli

This is my first project in `Rust` learning project focused on implementing a simple SQLite database file reader. The primary goal is to deepen understanding and play with the Rust language through hands-on experience with binary file parsing, data structures, and command-line interface development. The application reads SQLite database file and support simple queries and basic dot commands like `.tables` for displaying available tables or `.dbinfo` to see some metadata about db itself. (see more [sqlite cli](https://sqlite.org/cli.html) and [sqlite3 file format](https://sqlite.org/fileformat.html)).

## SQLite3 File Structure

SQLite databases are stored as single files divided into fixed-size pages (4096 bytes). Each page contains headers and data cells. The very first page (Page 1) is unique because it contains the 100-byte File Header. Below is a visual representation of the key components:

```
File
├─ Database Header (100 bytes, only on Page 1)
│  ├─ Magic: "SQLite format 3" (bytes 0-15)
│  ├─ Page Size (bytes 16-17) → typically 4096
│  └─ ... other metadata ...
│
├─ Page 1 (4096 bytes by default)
│  ├─ Page Header
│  ├─ Cell Pointers (N × 2 bytes)
│  │   ... free space
│  └─ Cell Content / Payload
│
├─ Page 2
│  └─ (same structure as Page 1, no DB header)
│
└─ Page N
   └─ (same structure as Page 1 or 2)
```

## Page structure
A SQLite database is not a continuous stream of data; it is divided into equal-sized blocks called Pages. Every disk I/O operation happens at the page level to ensure efficiency.
```
+-----------------------------------------------------------+
| Page Header (8 or 12 bytes)                               |
| (Flags, offset to first freeblock, cell count, etc.)      |
+-----------------------------------------------------------+
| Cell Pointer Array (Offsets)                              |
| [Offset 1] [Offset 2] [Offset 3] ... growth direction --> |
+-----------------------------------------------------------+
| Unallocated Space (Free Space)                            |
|                                                           |
|                      <--- ... --->                        |
|                                                           |
+-----------------------------------------------------------+
| Cell Payload Area (Data & Keys)                           |
|                <-- growth direction ... [Cell 3 Content]  |
|                        [Cell 2 Content] [Cell 1 Content]  |
+-----------------------------------------------------------+
```

## Cell structure
Cell is a variable-length structure stored within a B-Tree page. Because SQLite stores rows of different sizes, cells are not uniform 
```
|----------------------- CELL STRUCTURE (Payload) ------------------------|
|  Payload Size  |  RowID (Key)  |  Record Header  |     Column Data      |
|   (Varint*)    |   (Varint*)   |  (Serial Types) |  (Strings, Blobs...) |
|-------------------------------------------------------------------------|
      |               |                 |                  |
      |               |                 |                  +--> The actual values
      |               |                 +---------------------> Defines types/lengths
      |               +---------------------------------------> The row key id
      +-------------------------------------------------------> Total byte size
```

*A Varint (Variable-length Integer) is a space-saving encoding used by SQLite to store 64-bit integers using as few bytes as possible. Instead of always using 8 bytes for a small number like 10, SQLite uses between 1 and 9 bytes based on the value's magnitude.

## Multiple pages scanning logic

This diagram shows how the scanner processes a table for example 1000 users. The Root page acts as a map, directing the scanner to specific pages. In this examples we assume that single page can hold 400 user rows. (see also very useful tool for browsing database internals https://sqlite-internal.pages.dev/)

```
[ Page: table interior (root) type 0x05]
|
|-- [ Cell 1 ] key 400, ptr to page 2 --> users with id < 400 are on page 2
|-- [ Cell 2 ] key 800, ptr to page 5 --> users with id < 800 are on page 5
|-- [ Header ] ptr page 10 --> Right-most pointer - rest of users are stored on page 10
|
v
[ the scanning process ]
|
|   1. Reading page 2 (table leaf, type 0x0D)
|      +----------------------------+
|      | Users with IDs: 1 to 400   | --> [fetching records]
|      |  record 1                  |
|      |  record 2                  |
|      |  ...                       |
|      |  record 400                |
|      +----------------------------+
|
|   2. Reading page 5 (table leaf, type 0x0D)
|      +----------------------------+
|      | Users with IDs: 401 to 800 | --> [fetching records]
|      |  ...                       |
|      +----------------------------+
|
|   3. Reading page 10 (table leaf, type 0x0D)
|      +----------------------------+
|      | Users with IDs: 801 to 1000| --> [fetching records]
|      |  ...                       |
|      +----------------------------+
|
v
[ scan complete: 1000 rows ]
```

## Filtering

Currently, select query support a single where condition for numbers (int) and text (text) with support for appropriate operators. The following table describes the supported operators:

| Operator | Code (Enum) | SQL Syntax | Implementation Support |
| :--- | :--- | :--- | :--- |
| **Equal** | `Eq` | `=` | Supported for both Integers and Strings |
| **Not Equal** | `Neq` | `!=` | Supported for both Integers and Strings |
| **Less Than** | `Lt` | `<` | Supported for Integers only |
| **Less or Equal** | `Lte` | `<=` | Supported for Integers only |
| **Greater Than** | `Gt` | `>` | Supported for Integers only |
| **Greater or Equal** | `Gte` | `>=` | Supported for Integers only |

> **Note:** Attempting to use inequality operators (`Lt`, `Lte`, `Gt`, `Gte`) on string values will result in a error.

## How to Run

1. Navigate to the project directory: `cd sqlite-cli`
2. Build the project: `cargo build`
3. Run the application: `cargo run`
4. Optional: `cargo test`

## Usage Example

```bash
cargo run
db> .dbinfo
database page size:     4096
database page count:    5
software version:       3051000
... other metadata
db> .tables
numbers   books   cars
db> select * from books
1|lord of the rings
2|hobbit
db> select * from books where title = 'hobbit'
2|hobbit
db> select * from users where id = 325
325|User_325|user325@example.com
db> select * from users where id >= 998
998|User_998|user998@example.com
999|User_999|user999@example.com
1000|User_1000|user1000@example.com
db> .exit
```

## Code Structure

- `main.rs`: Application entry point and command interpreter
- `database.rs`: Core database structure and file handling
- `page.rs`: Page parsing and management logic
- `page_reader.rs`: Low-level page reading utilities
- `tokenizer.rs`: parse input string into list of tokens
- `parser.rs`: validates statement based on tokens 
- `scanner.rs`: Data scanning and record parsing
- `executor.rs`: read records based on given query plan

## Todo
- ✅ improve scanner and fetching rows (`scanner::scan()`)
- ✅ parse basic sql query -> select statement
- ✅ fix page header size 8/12 bytes (now its hardcoded for simplicity)
- fix warnings
- add some other tests

## Future ideas
- ✅ handle multipaging
- ✅ introduce simple filter predicate(s) using `where`
- improve select statement to specify direct columns instead of only `select *`
- introduce other types of dot commands (see https://sqlite.org/cli.html)