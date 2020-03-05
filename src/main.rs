extern crate rusoto_core;

use docopt::Docopt;
use std::fs::create_dir_all;
use std::path::Path;
use std::process::Command;

use futures::{Future, Stream};
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, PutObjectOutput, PutObjectRequest, S3Client, S3};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::Write;

const USAGE: &'static str = "
Edit S3 files

Usage:
    s3-edit-rs edit <s3-path>
    s3-edit-rs (-h | --help)

Options:
    -h --help   Show this screen.
";

fn donwload_file_from_s3(
    file_path: &Path,
    bucket: String,
    key: String,
) -> Result<(), Box<dyn Error>> {
    let s3_client = S3Client::new(Region::default());
    let get_req = GetObjectRequest {
        bucket: bucket.clone(),
        key: key.clone(),
        ..Default::default()
    };
    let result = s3_client.get_object(get_req).sync()?;
    let stream = result.body.unwrap();
    let body = stream.concat2().wait().unwrap();
    let mut file = File::create(&file_path)?;
    Ok(file.write_all(&body)?)
}

fn upload_file_to_s3(
    file_path: &Path,
    bucket: String,
    key: String,
) -> Result<PutObjectOutput, Box<dyn Error>> {
    let mut file = File::open(file_path)?;
    let mut file_data: Vec<u8> = vec![];
    file.read_to_end(&mut file_data)?;

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

    let s: Vec<&str> = args
        .get_str("<s3-path>")
        .trim_start_matches("s3://")
        .split("/")
        .collect();
    let bucket = s[0].to_string();
    let key = s[1..].join("/");

    let file_name = s.last().unwrap().to_owned();
    let tmp_dir_path = Path::new("/tmp/.s3-edit-rs");
    create_dir_all(tmp_dir_path).expect("failed to create directory");
    let tmp_file_path = tmp_dir_path.join(Path::new(file_name));

    donwload_file_from_s3(&tmp_file_path, bucket.clone(), key.clone()).expect("Download failed");

    let editor = match env::var_os("EDITOR") {
        Some(val) => val.into_string().unwrap_or(String::from("vi")),
        None => String::from("vi"),
    };
    Command::new("sh")
        .arg("-c")
        .arg(format!("{} {}", editor, tmp_file_path.to_str().unwrap()))
        .status()
        .expect("failed to open file");

    upload_file_to_s3(&tmp_file_path, bucket.clone(), key.clone()).expect("Failed to edit file");
}
