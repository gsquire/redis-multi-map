# redis-multi-map
[![Build Status](https://travis-ci.org/gsquire/redis-multi-map.svg?branch=master)](https://travis-ci.org/gsquire/redis-multi-map)

This crate provides a custom [Redis module](https://redis.io/topics/modules-intro) type using Rust's FFI support.
By leveraging the ability to make [custom types](https://redis.io/topics/modules-native-types) that Redis can utilize we can
use Rust to provide a new data structure. The `MultiMap` type is akin to Redis' existing hash type with the added
benefit of storing more than one value.

## Production Readiness
I would not consider this module to be production ready at the moment. There needs to be more
unit tests as well as benchmarking.

## Install
You must have Rust and LLVM installed in order to compile this project. The preferred way to install Rust is using the
[rustup](https://rustup.rs/) tool. If you are wary of shell script installs, you can download it through brew.

```sh
brew install llvm # And `brew install rust` if you don't already have it installed.

git clone https://github.com/gsquire/redis-multi-map
cd redis-multi-map
cargo build --release # The dynamic library will be under the target/release folder.
```

## Running Redis
You can load the module in a few ways.

### Directly from the command line.
```sh
redis-server --loadmodule /path/to/module.dylib # Or /path/to/module.so on Unix systems.
```

### Using the Redis configuration file
```sh
loadmodule /path/to/module.dylib
```

## API
The API is open to extending if other functionality is desired. The currently supported commands are as follows:

### Insert
```sh
MULTIMAP.INSERT KEY MAP_KEY MAP_VALUE...
```
This command returns a simple string of "OK".

### Values
```sh
MULTIMAP.VALUES KEY MAP_KEY
```
This command lists all values associated with `MAP_KEY`. It is an array of strings.

### Length
```sh
MULTIMAP.LEN KEY MAP_KEY
```
This command returns the length of values for `MAP_KEY` and `0` if it doesn't exist. This is an integer response.

### Delete
```sh
MULTIMAP.DEL KEY MAP_KEY
```
This command deletes `MAP_KEY` from `KEY`. It is an integer response of `0` if `MAP_KEY` did not exist and `1` if it did.

## Thanks
Thanks to [redis-cell](https://github.com/brandur/redis-cell) for providing some motivation and guidance in making another
module using Rust.
## License
This code is release under an MIT license and the Redis license stored under REDIS_LICENSE.
