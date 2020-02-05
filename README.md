# s3-edit-rs
The original idea for this project comes from  [s3-edit](https://github.com/tsub/s3-edit) written in Go by [tsub](https://github.com/tsub). I started this project for educational purposes, and it is my first project written in Rust.

## Intallation
### Install with cargo
```
$ cargo install --git https://github.com/gorros/s3-edit-rs
```

## Requirements

- AWS credentials
- Upload files to S3 in advance

## Usage
Upload the file to S3 in advance.
```
$ echo "The quick brown fox jumps over the lazy dog" > test_keyboard.txt
$ aws s3 cp test_keyboard.txt s3://mybucket/test_keyboard.txt
```
To directly edit a file on S3, use edit subcommand.

```
$ s3-edit-rd edit s3://mybucket/test_keyboard.txt
```