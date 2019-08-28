use s3::bucket::Bucket;
use s3::credentials::Credentials;
use s3::region::Region;
use std::fs::File;
use std::io::Read;
use std::{env, fs};
use walkdir::{DirEntry, WalkDir};

fn main() {
    let bucket_name = "brows3r-test";
    let region = "ap-southeast-2";
    let test_directory = "/Users/Shavz/rust-watcher-test/";
    let aws_region: Region = region.parse().unwrap();
    let credentials: Credentials = Credentials::new(None, None, None, None);
    let test_bucket: Bucket = Bucket::new(bucket_name, aws_region, credentials).unwrap();
    // iterate in the directory
    loop {
        for entry in WalkDir::new(test_directory)
            .into_iter()
            .filter_map(|e| e.ok())
            {
                let md = entry.metadata().unwrap();
                if md.is_file() && !entry.file_name().to_str().unwrap().starts_with(".") {
                    println!("{}", entry.path().display());
                    let data_file_result = File::open(entry.path());
                    let was_ok = data_file_result.is_ok();
                    println!(" was file okay?");
                    println!("{}", was_ok);
                    if was_ok {
                        println!("okay pushing file");
                        let mut data_file = data_file_result.unwrap();
                        let mut data_buffer = Vec::new();
                        data_file.read_to_end(&mut data_buffer);
                        test_bucket.put_object(
                            entry.path().file_name().unwrap().to_str().unwrap(),
                            data_buffer.as_ref(),
                            "text/plain",
                        );
                        // delete file when done
                        fs::remove_file(entry.path());
                    }
                }
            }
    }
}
