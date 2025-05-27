# Media Timeline

Media timeline for the fediverse

Show a media timeline for hashtags of interest.
This is not a replacement client for the fediverse.

## Usage

You can run the container image [`ghcr.io/milk/media-timeline:latest`](https://github.com/MiLk/media-timeline/pkgs/container/media-timeline).

```sh
podman run \
  --name media-timeline \
  --read-only \
  --publish 1337:1337 \
  --env LISTEN_ADDR=0.0.0.0 \
  --env RUST_LOG=debug \
  --volume ./data:/app/data:rw \
  ghcr.io/milk/media-timeline:latest
```

## Configuration

You can set multiple environment variables to configure the application:
- `LISTEN_ADDR` to change the address the http server is listening to (e.g: `LISTEN_ADDR=0.0.0.0`).

## Building from source

```cargo build --release```

### Cross compilation



```sh
# For macos
make setup/macos
# For fedora
make setup/fedora
```

### Build

```sh
make build/aarch64-linux
```

## Architecture overview

~~Inspired by~~ Stolen from [microsoft/cookiecutter-rust-actix-clean-architecture](https://github.com/microsoft/cookiecutter-rust-actix-clean-architecture/blob/main/docs/onion-architecture-article.md#architecture-overview).

The onion architecture is a layered architecture based on the onion model.
Where each layer in the onion model is used to define the different layers of an application.

For this rust implementation 4 layers are used.
* api (app) module: The outermost layer that contains the controllers and the endpoints definition, serialization and deserialization of the data, validation and error handling.
* infrastructure: Layer that typically includes database connections, external APIs calls, logging and configuration management.
* services: Layer that contains the application's services, which encapsulate the core business logic and provide a higher-level abstraction for the application to interact with the domain entities.
* domain: The innermost layer that contains the core business logic and entities of the application.


Folder structure:
```
.
├── src
│   ├── api
│   │   ├── controllers
│   │   │   └── ...  # controllers for the api
│   │   └── dto # Data transfer objects
│   │       └── ... # Individual DTOs
│   ├── infrastructure
│   │   ├── services
│   │   │   └── ...  # Services that use third party libraries or services (e.g. email service)
│   │   ├── databases
│   │   │   └── ...  # Database adapaters and initialization
│   │   ├── repositories
│   │   │   └── ...  # Repositories for interacting with the databases
│   │   └── models
│   │       └── ...  # Database models
│   ├── domain
│   │   ├── mod.rs
│   │   ├── constants.rs
│   │   ├── errors.rs
│   │   ├── models
│   │   │   └── ...  # Business logic models traits or structs
│   │   ├── services
│   │   │   └── ...  # Service traits
│   │   └── repositories
│   │       └── ...  # Repository traits
│   ├── services
│   │   └── ...  # Concrete service implementation for interacting with the domain (business logic)
│   ├── container.rs
│   ├── create_app.rs # app factory
│   ├── lib.rs
│   └── main.rs
```
