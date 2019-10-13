#[macro_use]
extern crate clap;
#[macro_use]
extern crate human_panic;

use clap::App;
use s3::bucket::Bucket;
use s3::credentials::Credentials;
use s3::region::Region;
use std::fs::File;
use std::io::{Read, Write};
use std::{env, fs, thread, time};
use walkdir::{DirEntry, WalkDir};

enum SyncServices {
    S3,
}

fn main() {
    setup_panic!();
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let current_working_dir = env::current_dir().unwrap().display().to_string();
    let bucket_name = matches.value_of("bucket").unwrap_or("");
    let region = matches.value_of("awsregion").unwrap_or("");
    let mode = matches.value_of("synctype").unwrap_or("");
    let sync_dir = matches.value_of("syncdir").unwrap_or(current_working_dir.as_ref());
    let syncsource = matches.value_of("syncsource").unwrap_or("");
    let region = "ap-southeast-2";
    let delete_flag = true;
    let aws_region: Region = region.parse().unwrap();
    let credentials: Credentials = Credentials::new(None, None, None, None);
    let test_bucket: Bucket = Bucket::new(bucket_name, aws_region, credentials).unwrap();
    let source = "directory";
    run_s3_sync(delete_flag, &test_bucket, mode, source, sync_dir)
}

fn validate_user_input(sync_type: &str) {
    let allowed_sync_types: Vec<&str> = vec!["upload", "download", "replicate"];
    if !allowed_sync_types.contains(&sync_type) {
        panic!("Unable to recognise given synctype, only following values are accepted: upload, download, replicate")
    }
}

fn run_s3_sync(delete_flag: bool, test_bucket: &Bucket, mode: &str, source: &str, sync_dir: &str) {
    let sleep_time = time::Duration::from_secs(1000);
    loop {
        match mode.to_lowercase().as_str() {
            "upload" => run_upload(delete_flag, &test_bucket, sync_dir),

            "download" => run_download(test_bucket, sync_dir),

            "replicate" => run_sync(test_bucket, sync_dir, source),
            _ => {
                panic!("Unable to recognise given sync type");
            }
        }
        //        thread::sleep(sleep_time);
    }
}

fn run_upload(delete_flag: bool, test_bucket: &Bucket, sync_dir: &str) {
    for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        let md = entry.metadata().unwrap();
        if md.is_file() && !entry.file_name().to_str().unwrap().starts_with(sync_dir) {
            let data_file_result = File::open(entry.path());
            let was_ok = data_file_result.is_ok();
            if was_ok {
                let mut data_file = data_file_result.unwrap();
                let mut data_buffer = Vec::new();
                let upload_file_path = entry.path().file_name().unwrap().to_str().unwrap();
                println!("checking if {} exists", upload_file_path);
                let file_does_not_exist = test_bucket.get_object(upload_file_path).is_err();
                if file_does_not_exist {
                    println!("file not found, uploading {}", upload_file_path);
                    data_file.read_to_end(&mut data_buffer);
                    test_bucket.put_object(upload_file_path, data_buffer.as_ref(), "text/plain");

                    // delete file when done if the delete flag is set
                    if delete_flag {
                        fs::remove_file(entry.path());
                    }
                }
            }
        }
    }
}

fn run_download(test_bucket: &Bucket, sync_dir: &str) {
    // gets the list of files from s3 and scans the dir to see which files aren't present and downloads them
    let mut current_dir_files_list: Vec<String> = Vec::new();
    for entry in WalkDir::new(sync_dir).into_iter().filter_map(|e| e.ok()) {
        current_dir_files_list.push(
            entry
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
        )
    }

    let s3_files_list = test_bucket.list("", Some("")).unwrap();
    for (list, code) in s3_files_list {
        assert_eq!(200, code);
        for content in &list.contents {
            println!("checking for downloads");
            println!("checking if {} exists", content.key);
            if !current_dir_files_list.contains(&content.key) {
                println!("downloading");
                let (data, code) = test_bucket.get_object(content.key.as_ref()).unwrap();
                println!("code was {}", code);
                let new_file_path = format!("{}/{}", sync_dir, content.key.as_str());
                let mut buffer = File::create(new_file_path).unwrap();
                let file_write_result = buffer.write(data.as_ref());
                println!("file write result is {}", file_write_result.is_ok())
            }
        }
    }
}

fn run_sync(test_bucket: &Bucket, sync_dir: &str, sync_source: &str) {
    match sync_source {
        "directory" => {
            // fist upload everything existing
            run_upload(false, test_bucket, sync_dir);

            // then delete everything that is in s3 but not in the directory
            let mut current_dir_files_list: Vec<String> = Vec::new();
            for entry in WalkDir::new(sync_dir).into_iter().filter_map(|e| e.ok()) {
                let md = entry.metadata().unwrap();
                if md.is_file() && !entry.file_name().to_str().unwrap().starts_with(".") {
                    current_dir_files_list.push(
                        entry
                            .path()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned(),
                    )
                }
            }

            let s3_files_list = test_bucket.list("", Some("")).unwrap();

            for (list, code) in s3_files_list {
                for content in &list.contents {
                    if !current_dir_files_list.contains(&content.key) {
                        let (_, delete_op_code) = test_bucket.delete_object(&content.key).unwrap();
                        assert_eq!(200, code);
                    }
                }
            }
        }

        "s3" => {
            let mut current_dir_files_list: Vec<String> = Vec::new();
            for entry in WalkDir::new(sync_dir).into_iter().filter_map(|e| e.ok()) {
                let md = entry.metadata().unwrap();
                if md.is_file() && !entry.file_name().to_str().unwrap().starts_with(".") {
                    current_dir_files_list.push(
                        entry
                            .path()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned(),
                    )
                }
            }

            let s3_files_list = test_bucket.list("", Some("")).unwrap();

            // download
            run_download(test_bucket, sync_dir);

            // remove
            for (list, code) in s3_files_list {
                let file_keys = &list
                    .contents
                    .into_iter()
                    .map(|x| x.key.to_string())
                    .collect::<Vec<String>>();
                println!("file keys are {:?}", file_keys);
                for current_dir_file in &current_dir_files_list {
                    println!(
                        "checking current dir file: {} not inside file list",
                        current_dir_file
                    );
                    if !file_keys.contains(&current_dir_file) {
                        println!("removing file {}", current_dir_file);
                        fs::remove_file(format!("{}", current_dir_file));
                    }
                }
            }
        }
        _ => {}
    }
}
