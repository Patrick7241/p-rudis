# p-rudis 🚀

> **Language Switch**: [中文](README.md) | [English](README_EN.md)

## 项目介绍 📚

`p-rudis` 是一个用 **Rust** 编写的轻量级键值数据库，采用 Redis 的 **RESP 协议**，旨在提供高效、可靠的内存存储解决方案。项目的目标是实现一个简单的服务器，能够通过类似 Redis 的协议与客户端交互，支持基本的键值操作、发布/订阅模式等。虽然 `p-rudis` 目前还没有 Redis 的所有功能，但它已经实现了许多核心命令和协议，适合用作学习、实验或者小型应用的数据库。⚡️

- **高性能**：基于 Rust 和 Tokio 异步运行时，能够处理大量并发连接。
- **易于扩展**：模块化设计，便于扩展新功能，如持久化存储或更多 Redis 命令。
- **RESP 协议兼容**：支持与任何遵循 RESP 协议的客户端交互，例如 Redis 客户端。
- **发布/订阅功能**：支持客户端订阅频道，并在发布者发布消息时推送给订阅者。
- **RDB,AOF**：支持 RDB 和 AOF 文件格式，以实现数据的持久化。

## 特性 ✨

- **RESP 协议支持**：客户端与服务端之间通过 RESP 协议进行通信。
- **核心 Redis 命令**：如 `SET`、`GET`、`DEL` 等常见命令。
- **发布/订阅**：支持客户端订阅频道并接收消息推送。
- **高性能**：基于 **Tokio** 异步运行时，能够处理高并发连接。
- **模块化设计**：功能模块清晰，便于扩展和维护。
- **RDB,AOF**：支持 RDB 和 AOF 文件格式，以实现数据的持久化。

## 安装与运行 🚀

1. **克隆项目**

   选择合适的方式克隆项目：

   ```bash
   git clone https://github.com/Patrick7241/p-rudis.git
   cd p-rudis
   ```

   或者如果你更喜欢使用 Gitee：

   ```bash
   git clone https://gitee.com/hvck/p-rudis.git
   cd p-rudis
   ```

2. **构建项目**

   使用 `cargo` 进行构建：

   ```bash
   cargo build --release
   ```

3. **运行项目**

   启动 `p-rudis` 服务器：

   ```bash
   cargo run
   ```

   默认情况下，服务会启动在 `127.0.0.1:6379`，可以通过 **Redis** 客户端进行连接和操作。

## 使用示例 🛠

### 1. 启动服务

启动 `p-rudis` 后，你可以通过任何支持 RESP 协议的客户端与服务器交互。例如，使用 **Redis CLI** 或其他工具连接到 `127.0.0.1:6379`。

### 2. 常见命令

- **SET 命令**：设置键值对

  ```bash
  SET mykey "Hello, World!"
  ```

- **GET 命令**：获取键的值

  ```bash
  GET mykey
  ```

- **DEL 命令**：删除键

  ```bash
  DEL mykey
  ```

### 3. 发布/订阅

`p-rudis` 支持发布/订阅模式，允许客户端订阅频道并接收来自其他客户端发布的消息。

- **SUBSCRIBE 命令**：订阅频道

  ```bash
  SUBSCRIBE mychannel
  ```

- **PUBLISH 命令**：发布消息到频道

  ```bash
  PUBLISH mychannel "Hello, Subscribers!"
  ```

## 项目结构 🗂

- **`commands.rs`**：定义了数据库支持的所有命令和对应的处理逻辑。
- **`connection.rs`**：处理与客户端的连接，负责数据的读取、命令解析和响应发送。
- **`db.rs`**：实现数据库模块，管理数据存储和操作。
- **`dict.rs`**：实现命令的加载和获取。
- **`frame.rs`**：定义了 RESP 协议的不同数据类型，如 `Simple`、`Error`、`Integer` 等。
- **`parser.rs`**：定义了 RESP 协议的解析器，负责将字节流转换为 RESP 数据类型。
- **`server.rs`**：负责启动和管理服务器，监听客户端连接并处理它们。
- **`shutdown.rs`**：管理服务端的优雅关闭，接收和触发关闭信号。
- **`persistence`**：实现持久化存储，支持 RDB 和 AOF 文件格式。

## TODO 🚧

- 增加更多 Redis 命令支持。
- 支持通过配置文件启动，简化服务器的配置。
- 改进错误处理和日志功能。
- 支持更多高级特性。

## 贡献 ❤️

欢迎提交 PR 和 issue 来帮助改进 `p-rudis`！如果你发现任何 bug 或有功能需求，请随时提出。非常欢迎你的加入！

## License 📝

此项目采用 **MIT 许可证**，详情请查看 [LICENSE](LICENSE) 文件。

