# Filer 🗄

Filer is a rust service to allow seamless document syncing with a variety of cloud systems. The application is written in [Rust](https://www.rust-lang.org/). Filer is designed to be easy to use, fault tolerant, fast and efficient at using system resources.

## How it works

Filer is a low overhead service, meant to be run in the background once as a daemon. Filer comes with a gui which the user can use to launch multiple instances of Filer for one or multiple directories.

Filer supports three main operation types:

- Upload: In this mode, filer will simply upload all new directory data to the provided cloud storage service.
- Download: In this mode filer will simply keep downloading all new data added in a new cloud service.
- Sync: In the mode the users can select a source directory, which can be either the cloud service or the local directory. Once source is specified, filer will try to keep the synced directory in exact state as the source directory.

## Usage

To use and build filer on your local for now, clone this repository and run (ensuring [rustup](https://rustup.rs/) is installed prior to running):

```shell script
cargo build
```

Once build, run the created rust executable to start the syncing process to a specified s3 bucket.

