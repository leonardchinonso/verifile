### Verifile

### Description
This project allows a user to upload a set of files to server using a client. It also allows a user download a file with a given index from the server and verify its authenticity using Merkle Proofs

### How To Run
To run this project, you need to install `rust` and `cargo`. You can set it up [here](https://www.rust-lang.org/tools/install).

Once you have rust installed, clone the repository and navigate to the top level directory. You would need to add the files you need to upload to the server. You can upload the files anywhere in the current working directory. Alternatively, there are sample files in the `files` directory you can use.

Next, you would need to run the server and the clients on two different terminal processes.

To start the server...
First change the working directory to the server directory
```shell
$ cd server
```

Then start the server
```shell
$ cargo run
```

On a new terminal/interactive environment, send the files using the client. It takes two arguments, an optional `--file | -f` with the relative path of the files separated by commas and a `--action | -a` that can either be `send` or `download-N`, where `N` is the index of the file to download.

To send the files...
```shell
$ cargo run -- -f files/cv.txt,files/food.json,files/recipe.html,files/schools.csv -a send
```

After sending, you can download an arbitrary file at an index...
```shell
$ cargo run -- -a download-2
```
This downloads and verifies the file at index `2` - `files/recipe.html`.
