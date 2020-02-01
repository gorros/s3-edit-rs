extern crate rusoto_core;

use docopt::Docopt;
use std::fs::create_dir_all;
use std::path::Path;
use std::process::Command;

use futures::{Future, Stream};
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, PutObjectOutput, PutObjectRequest, S3Client, S3};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::io::Read;

const USAGE: &'static str = "
Edit S3 files

Usage:
    s3-edit-rs edit <s3-path>
    s3-edit-rs (-h | --help)

Options:
    -h --help   Show this screen.
";

// enum S3EditRSError {
//     SDKError(RusotoError),
//     S3Error(PutObjectError),
//     IoError(io::Error)
// }

// impl From<io::Error> for S3EditRSError {
//     fn from(error: io::Error) -> Self {
//         S3EditRSError::IoError(error)
//     }
// }

// impl From<PutObjectError> for S3EditRSError {
//     fn from(error: PutObjectError) -> Self {
//         S3EditRSError::S3Error(error)
//     }
// }

// impl From<RusotoError> for S3EditRSError {
//     fn from(error: RusotoError) -> Self {
//         S3EditRSError::SDKError(error)
//     }
// }

fn upload_file_to_s3(
    file_path: &Path,
    bucket: String,
    key: String,
) -> Result<PutObjectOutput, Box<dyn Error>> {
    let mut file = File::open(file_path)?;
    let mut file_data: Vec<u8> = vec![];
    file.read_to_end(&mut file_data)
        .expect("Failed to read file");

    let client = S3Client::new(Region::UsEast1);
    let mut request = PutObjectRequest::default();
    request.body = Some(file_data.into());
    request.bucket = bucket;
    request.key = key;

    Ok(client.put_object(request).sync()?)
}

fn main() {
    let args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.parse())
        .unwrap_or_else(|e| e.exit());
    // println!("File {}", args.get_str("<s3-path>"));

    let s: Vec<&str> = args
        .get_str("<s3-path>")
        .trim_start_matches("s3://")
        .split("/")
        .collect();
    let bucket = s[0].to_string();
    let key = s[1..].join("/");
    // println!("{} {}", bucket, key);
    let s3_client = S3Client::new(Region::default());
    let get_req = GetObjectRequest {
        bucket: bucket.clone(),
        key: key.clone(),
        ..Default::default()
    };

    let file_name = s.last().unwrap().to_owned();
    let tmp_dir_path = Path::new("/tmp/.s3-edit-rs");
    create_dir_all(tmp_dir_path).expect("failed to create directory");
    let tmp_file_path = tmp_dir_path.join(Path::new(file_name));

    let result = s3_client.get_object(get_req).sync().expect("error!");
    let stream = result.body.unwrap();
    let body = stream.concat2().wait().unwrap();
    let mut file = File::create(&tmp_file_path).expect("create failed");
    file.write_all(&body).expect("failed to write body");

    let editor = "vim";
    Command::new("sh")
        .arg("-c")
        .arg(format!("{} {}", editor, tmp_file_path.to_str().unwrap()))
        .status()
        .expect("failed to open file");

    // // let hello = output.stdout;
    // println!("{}", String::from_utf8(hello).unwrap());
    // println!("{}", tmp_dir_path.to_str().unwrap())
    upload_file_to_s3(&tmp_file_path, bucket.clone(), key.clone()).expect("Failed to edit file");
    // Ok(())
}
