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
