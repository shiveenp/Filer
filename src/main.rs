use s3::bucket::Bucket;
use s3::credentials::Credentials;
use s3::region::Region;
use std::fs::File;
use std::io::{Read, Write};
use std::{env, fs};
use walkdir::{DirEntry, WalkDir};

enum SyncType {
    Upload,
    Download,
    Sync,
}

fn main() {
    let bucket_name = "brows3r-test";
    let region = "ap-southeast-2";
    let test_directory = "/Users/Shavz/rust-watcher-test/";
    let delete_flag = true;
    let aws_region: Region = region.parse().unwrap();
    let credentials: Credentials = Credentials::new(None, None, None, None);
    let test_bucket: Bucket = Bucket::new(bucket_name, aws_region, credentials).unwrap();

    let mode = SyncType::Sync;

    run_s3_sync(test_directory, delete_flag, &test_bucket, mode)
}

fn run_s3_sync(test_directory: &str, delete_flag: bool, test_bucket: &Bucket, mode: SyncType) {
    loop {
        match mode {
            SyncType::Upload => {
                run_upload(test_directory, delete_flag, &test_bucket)
            }

            SyncType::Download => {
                run_download(test_directory, test_bucket)
            }

            SyncType::Sync => {
                run_upload(test_directory, false, &test_bucket);
                run_download(test_directory, test_bucket);
            }
        }
    }
}

fn run_upload(test_directory: &str, delete_flag: bool, test_bucket: &Bucket) {
    for entry in WalkDir::new(test_directory)
        .into_iter()
        .filter_map(|e| e.ok())
        {
            let md = entry.metadata().unwrap();
            if md.is_file() && !entry.file_name().to_str().unwrap().starts_with(".") {
                let data_file_result = File::open(entry.path());
                let was_ok = data_file_result.is_ok();
                if was_ok {
                    let mut data_file = data_file_result.unwrap();
                    let mut data_buffer = Vec::new();
                    let upload_file_path = entry.path().file_name().unwrap().to_str().unwrap();
                    let (data, check_if_files_exists) = test_bucket.get_object(upload_file_path).unwrap();
                    if check_if_files_exists != 200 {
                        data_file.read_to_end(&mut data_buffer);
                        test_bucket.put_object(
                            upload_file_path,
                            data_buffer.as_ref(),
                            "text/plain",
                        );
                        // delete file when done if the delete flag is set
                        if delete_flag {
                            fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
}

fn run_download(test_directory: &str, test_bucket: &Bucket) {
    // gets the list of files from s3 and scans the dir to see which files aren't present and downloads them
    let mut current_dir_files_list: Vec<String> = Vec::new();
    for entry in WalkDir::new(test_directory)
        .into_iter()
        .filter_map(|e| e.ok())
        {
            current_dir_files_list.push(entry.path().file_name().unwrap().to_str().unwrap().to_owned())
        }

    let s3_files_list = test_bucket.list("", Some("")).unwrap();
    for (list, code) in s3_files_list {
        assert_eq!(200, code);
        for content in &list.contents {
            if !current_dir_files_list.contains(&content.key) {
                let (data, code) = test_bucket.get_object(content.key.as_ref()).unwrap();
                let mut buffer = File::create(test_directory.to_string() + content.key.as_ref()).unwrap();
                buffer.write(data.as_ref());
            }
        }
    }
}
