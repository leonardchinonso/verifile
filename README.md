### Verifile

### Description

This project allows a user to upload a set of files to server using a client. It also allows a user download a file with a given index from the server and verify its authenticity using Merkle Proofs

### How To Run

To run this project, you need to install `rust` and `cargo`. You can set it up [here](https://www.rust-lang.org/tools/install).

Once you have rust installed, clone the repository and navigate to the top level directory. You would need to add the files you need to upload to the server. You can upload the files anywhere in the current working directory. Alternatively, there are sample files in the `files` directory you can use.

Next, you would need to run the server and the clients on two different terminal processes and then make them interact with one another. Follow the below commands

1. Build the the client and server binaries.
```shell
$ cargo build
```

2. Start the server using the server binary.
```shell
$ cargo run --bin server
```

The client takes two arguments, an optional `--file | -f` with the relative path of the files separated by commas and a `--action | -a` that can either be `send` or `download-N`, where `N` is the index of the file to download.

3. Send the files from the client using the client binary.
```shell
$ cargo run --bin client -- -f files/cv.txt,files/food.json,files/recipe.html,files/schools.csv -a send
```
The server would receive and store the files if there are no issues.

4. Download the file by the index from the server using the server binary. Here we download the file at index 2.
```shell
$ cargo run --bin client -- -a download-2
```
The file should be downloaded if it is successful.


### Tests

To run tests, you would need to run it from the root directory.

```shell
$ cargo test
```
