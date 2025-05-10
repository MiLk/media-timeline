# Media Timeline

Media timeline for the fediverse

Show a media timeline for hashtags of interest.
This is not a replacement client for the fediverse.


## Cross compile to Raspberry Pi

### Requirements

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

### Deploy

```sh
export TARGET_HOST=
scp dist/build.tar.gz ${TARGET_HOST}:'~/media-timeline/'
ssh ${TARGET_HOST} 'cd ~/media-timeline && tar xvf build.tar.gz && rm build.tar.gz'
```

### Running

```sh
LISTEN_ADDR=0.0.0.0 ./media-timeline
```
