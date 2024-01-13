[![CI](https://github.com/gngpp/fcsrv/actions/workflows/ci.yml/badge.svg)](https://github.com/gngpp/fcsrv/actions/workflows/ci.yml)
[![CI](https://github.com/gngpp/fcsrv/actions/workflows/release.yml/badge.svg)](https://github.com/gngpp/fcsrv/actions/workflows/release.yml)
 <a target="_blank" href="https://github.com/gngpp/fcsrv/blob/main/LICENSE">
  <img src="https://img.shields.io/badge/license-MIT-blue.svg"/>
 </a>

# fcsrv

Funcaptcha solver server

> Model uses [funcaptcha-challenger](https://github.com/MagicalMadoka/funcaptcha-challenger)

### Model

Currently supports the following models:

- `3d_rollball_animals`
- `3d_rollball_objects`

### Usage

```shell
# Run
fcsrv run 

# Start Daemon (Run in the background), must use sudo
fcsrv start

# Restart Daemon, must use sudo
fcsrv restart

# Stop Daemon, must use sudo
fcsrv stop

# Show Daemon log
fcsrv log

# Show Daemon status
fcsrv status

# Online Update
fcsrv update
```

### Command Manual

#### Description

- `--debug`, Debug mode
- `--bind`, Http service listening address, default 0.0.0.0:8000
- `--tls-cert`, TLS certificate file
- `--tls-key`, TLS private key file
- `--api-key`, API key
- `--model-dir`, Funcaptcha model directory
- `--num-threads`, Number of threads (ONNX Runtime session), default 1

```shell
$ fcsrv -h
Funcaptcha solver server

Usage: fcsrv
       fcsrv <COMMAND>

Commands:
  run      Run server
  start    Start server daemon
  restart  Restart server daemon
  stop     Stop server daemon
  status   Show the server daemon process
  log      Show the server daemon log
  update   Update the application
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

$ fcsrv run -h
Run server

Usage: fcsrv run [OPTIONS]

Options:
  -d, --debug                      Debug mode
  -b, --bind <BIND>                Bind address [default: 0.0.0.0:8000]
      --tls-cert <TLS_CERT>        TLS certificate file
      --tls-key <TLS_KEY>          TLS private key file
  -A, --api-key <API_KEY>          API key
      --model-dir <MODEL_DIR>      Funcaptcha model directory
      --num-threads <NUM_THREADS>  Number of threads (ONNX Runtime session) [default: 1]
  -h, --help                       Print help
```

### Example

```shell
curl --location 'http://127.0.0.1:8000/task' \
--header 'Content-Type: application/json' \
--data '{
    "type": "3d_rollball_animals",
    "image": "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD/2wCEAAkGBxMTEhUTExMWFhUXGBgYGBgYGBgYGBgYGBgYGBgYGBgYHSggGBolHRgXITEhJSkrLi4uGB8zODMsNygtLisBCgoKDg0OGxAQGy0"
}'
```

### Compile

- Linux compile, Ubuntu machine for example:

```shell
git clone https://github.com/gngpp/fcsrv.git && cd fcsrv
cargo build --release
```

### Contributing

If you would like to submit your contribution, please open a [Pull Request](https://github.com/gngpp/fcsrv/pulls).

### Getting help

Your question might already be answered on the [issues](https://github.com/gngpp/fcsrv/issues)

### License

**fcsrv** © [gngpp](https://github.com/gngpp), Released under the [MIT](./LICENSE) License.
