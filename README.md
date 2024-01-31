# graphql-transformer

`graphql-transformer` is a simple Rust-based HTTP server based on hyper that transforms incoming POST requests into GET requests and relays them to a specified host.

## Installation

Before you can use `graphql-transformer`, you'll need to install Rust and set up your development environment. You can install Rust by following the instructions on the official website: [Rust Installation Guide](https://www.rust-lang.org/tools/install).

Once Rust is installed, you can clone the `graphql-transformer` repository and build the project:

```bash
git clone https://github.com/your-username/graphql-transformer.git
cd graphql-transformer
cargo build --release
```

## Usage

1. Set the `GRAPHQL_TRANSFORMER_BASE_URL` environment variable to the host where you want to relay the transformed GET requests. For example:

```bash
export GRAPHQL_TRANSFORMER_BASE_URL=https://api.example.com/graphql
```
2. Run the graphql-transformer server:

```
cargo run --release
```
The server will start and listen for incoming requests on `http://127.0.0.1:80`.

1. Send a POST request to `http://127.0.0.1:80` with your GraphQL query in the request body. The server will transform the request into a GET request and relay it to the specified host.

For example:

```bash
curl -X POST http://127.0.0.1:3000 -d '{"query": "your-graphql-query"}'
```

## TODO
- APQ support
- Docker image

## Contributing
We welcome contributions to graphql-transformer. If you find any issues or have suggestions for improvements, please open an issue or submit a pull request.

## Licence
This project is licensed under the MIT License.