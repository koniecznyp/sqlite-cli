# SQLite Database Reader

This project is my Rust learning project focused on implementing a simple SQLite database file reader. The primary goal is to deepen understanding and play with the Rust language through hands-on experience with binary file parsing, data structures, and command-line interface development. The application reads SQLite database file and support basic dot command `.tables` for displaying available tables. (see more [sqlite cli](https://sqlite.org/cli.html) and [sqlite3 file format](https://sqlite.org/fileformat.html)).

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
│  ├─ Cell Pointers (N × 2 bytes) [-> grow direction ->] 
│  ├─ ... free space
│  └─ Cell Content / Payload [<- grow direction <-]
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
| [Offset 1] [Offset 2] [Offset 3] ...                      |
+-----------------------------------------------------------+
| Unallocated Space (Free Space)                            |
|                                                           |
|                      <--- ... --->                        |
|                                                           |
+-----------------------------------------------------------+
| Cell Payload Area (Data & Keys)                           |
|                                         [Cell 3 Content]  |
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
## How to Run

1. Ensure Rust is installed: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Clone this repository
3. Navigate to the project directory: `cd sqlite-cli`
4. Build the project: `cargo build`
5. Run the application: `cargo run`

## Usage Example

```bash
cargo run
db> .tables
numbers
books
cars
settings
db> .exit
```

## Code Structure

- `main.rs`: Application entry point and command interpreter
- `database.rs`: Core database structure and file handling
- `page.rs`: Page parsing and management logic
- `page_reader.rs`: Low-level page reading utilities
- `scanner.rs`: Data scanning and record parsing functionality

## Todo
- fix page header 8/12 bytes (now its hardcoded for simplicity)
- improve scanner and fetching rows (`scanner::scan()`)
- parse basic sql query -> select statement