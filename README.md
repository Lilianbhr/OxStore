# OxStore
OxStore is a thread-safe key-value database server written in Rust. Features TCP networking, a custom text protocol, Write-Ahead Log (WAL) persistence, and ergonomic error handling with `thiserror`.

## Protocol and Parsing Module (`protocol.rs`)
This module defines the communication interface between clients and the server, handling both incoming requests and outgoing responses.

### 1. Command Parsing
The `Command` enum represents the supported operations:
 - `Set { key, value }`: Inserts or updates a key-value pair.
 - `Get { key }`: Retrieves the value associated with a key.
 - `Del { key }`: Removes a key from the database.

The `parse` method takes a raw input string, splits it by spaces, and validates the syntax:
 - It checks if the command name (`set`, `get`, `del`) is valid.
 - It verifies that the correct number of arguments is provided.
 - If an error occurs, it returns a custom `CommandError` derived via `thiserror`, distinguishing between `InvalidCommandName` and `InvalidArguments`.

### 2. Server Answers
The `Answer` enum formats the server responses before sending them back over the TCP stream:
 - `Ok`: Returns a success message confirming the operation.
 - `Found { value }`: Returns the requested value.
 - `NotFound`: Informs the client that the specified key does not exist.

### 3. Unit Testing
A comprehensive test suite is integrated at the bottom of the module to validate correct parsing for valid commands, proper rejection of invalid arguments or unknown commands, and correct response formatting.
