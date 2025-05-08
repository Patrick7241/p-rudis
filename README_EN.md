# p-rudis 🚀

> **Language Switch**: [中文](README.md) | [English](README_EN.md)

## Project Introduction 📚

`p-rudis` is a lightweight key-value database written in **Rust**, implementing the Redis **RESP protocol**. It aims to provide an efficient and reliable in-memory storage solution. The goal of the project is to create a simple server that can interact with clients using a Redis-like protocol, supporting basic key-value operations and the publish/subscribe pattern. Although `p-rudis` does not yet have all the features of Redis, it has already implemented many core commands and protocols, making it suitable for learning, experimentation, or small-scale applications. ⚡️

- **High Performance**: Built with Rust and the Tokio async runtime, capable of handling a large number of concurrent connections.
- **Easy to Extend**: Modular design, making it easy to add new features such as persistence storage or more Redis commands.
- **RESP Protocol Compatibility**: Can interact with any client following the RESP protocol, such as Redis clients.
- **Publish/Subscribe**: Supports clients subscribing to channels and pushing messages to subscribers when publishers publish messages.

## Features ✨

- **RESP Protocol Support**: Communication between the client and server follows the RESP protocol.
- **Core Redis Commands**: Implements common Redis commands like `SET`, `GET`, `DEL`, etc.
- **Publish/Subscribe**: Allows clients to subscribe to channels and receive pushed messages.
- **High Performance**: Based on the **Tokio** async runtime, capable of handling high-concurrency connections.
- **Modular Design**: Features are clearly separated, making it easy to extend and maintain.

## Installation and Run 🚀

1. **Clone the project**

   Choose the appropriate method to clone the project:

   ```bash
   git clone https://github.com/Patrick7241/p-rudis.git
   cd p-rudis
   ```

   Or, if you prefer using Gitee:

   ```bash
   git clone https://gitee.com/hvck/p-rudis.git
   cd p-rudis
   ```

2. **Build the project**

   Build the project using `cargo`:

   ```bash
   cargo build --release
   ```

3. **Run the project**

   Start the `p-rudis` server:

   ```bash
   cargo run
   ```

   By default, the server will start at `127.0.0.1:6379`, and you can connect and interact with it using a **Redis** client.

## Usage Examples 🛠

### 1. Start the server

Once `p-rudis` is running, you can interact with the server through any client that supports the RESP protocol, such as the **Redis CLI** or other tools by connecting to `127.0.0.1:6379`.

### 2. Common Commands

- **SET Command**: Set a key-value pair

  ```bash
  SET mykey "Hello, World!"
  ```

- **GET Command**: Get the value of a key

  ```bash
  GET mykey
  ```

- **DEL Command**: Delete a key

  ```bash
  DEL mykey
  ```

### 3. Publish/Subscribe

`p-rudis` supports the publish/subscribe pattern, allowing clients to subscribe to channels and receive messages published by other clients.

- **SUBSCRIBE Command**: Subscribe to a channel

  ```bash
  SUBSCRIBE mychannel
  ```

- **PUBLISH Command**: Publish a message to a channel

  ```bash
  PUBLISH mychannel "Hello, Subscribers!"
  ```

## Project Structure 🗂

- **`commands.rs`**: Defines all the commands supported by the database and their respective handling logic.
- **`connection.rs`**: Handles client connections, reading data, parsing commands, and sending responses.
- **`db.rs`**: Implements the database module, managing data storage and operations.
- **`dict.rs`**: Implements command loading and retrieval.
- **`frame.rs`**: Defines the various data types in the RESP protocol, such as `Simple`, `Error`, `Integer`, etc.
- **`parser.rs`**: Defines the RESP protocol parser, responsible for converting byte streams into RESP data types.
- **`server.rs`**: Manages server startup, listens for client connections, and processes them.
- **`shutdown.rs`**: Manages the graceful shutdown of the server, receiving and triggering shutdown signals.

## TODO 🚧

- Implement persistence storage.
- Add support for more Redis commands.
- Support server configuration via a configuration file.
- Improve error handling and logging functionality.
- Implement more advanced features.

## Contributing ❤️

We welcome PRs and issues to help improve `p-rudis`! If you find any bugs or have feature requests, feel free to raise them. Your contributions are highly appreciated! 😊

## License 📝

This project is licensed under the **MIT License**, for details please refer to the [LICENSE](LICENSE) file.
