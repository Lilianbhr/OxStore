# OxStore
OxStore is a thread-safe key-value database server written in Rust. Features TCP networking, a custom text protocol, Write-Ahead Log (WAL) persistence, and ergonomic error handling with `thiserror`.

## Protocol and Parsing Module (`protocol.rs`)
This module defines the communication interface between clients and the server, handling both incoming requests and outgoing responses.

### 1. Command Parsing
The `Command` enum represents the supported operations:
 * `Set { key, value }`: Inserts or updates a key-value pair.
 * `Get { key }`: Retrieves the value associated with a key.
 * `Del { key }`: Removes a key from the database.

The `parse` method takes a raw input string, splits it by spaces, and validates the syntax:
 * It checks if the command name (`set`, `get`, `del`) is valid.
 * It verifies that the correct number of arguments is provided.
 * If an error occurs, it returns a custom `CommandError` derived via `thiserror`, distinguishing between `InvalidCommandName` and `InvalidArguments`.

### 2. Server Answers
The `Answer` enum formats the server responses before sending them back over the TCP stream:
 * `Ok`: Returns a success message confirming the operation.
 * `Found { value }`: Returns the requested value.
 * `NotFound`: Informs the client that the specified key does not exist.

### 3. Unit Testing
A comprehensive test suite is integrated at the bottom of the module to validate correct parsing for valid commands, proper rejection of invalid arguments or unknown commands, and correct response formatting.


## Storage and Persistence Module (`storage.rs`)

This module manages execution logging, error tracking, data persistence through a Write-Ahead Log (WAL), and in-memory state recovery upon startup.

### 1. Execution Logging (`LogManager`)
The `LogManager` handles activity tracking, error reporting, and debugging output:
* It records successful or warning server operations into a dedicated log file, timestamping each entry using local time with millisecond precision and assigning levels (`[SUCCESS]` or `[WARN]`).
* It features a dedicated `log_error` method to capture invalid or malformed commands under an `[ERROR]` level, recording the raw input string alongside the specific parsing error.

### 2. Write-Ahead Log (`WalManager`)
The `WalManager` ensures data durability and crash safety:
* It opens the WAL file in read/write mode and ensures cursor positioning at the end of the file prior to any write operation.
* It filters incoming commands, appending only state-mutating operations (`Set` and `Del`) while ignoring read-only queries (`Get`).

### 3. State Recovery (`build_live_memory`)
To restore the database state when the server boots up:
* The `build_live_memory` method seeks back to the beginning of the WAL file.
* It reads and parses the historical commands sequentially line by line.
* It replays each operation into an in-memory `HashMap`, inserting or updating key-value pairs for `Set` commands and removing keys for `Del` commands, perfectly reconstructing the previous database state.
