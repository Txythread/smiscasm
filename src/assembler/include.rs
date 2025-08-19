use std::error;
use std::fs::*;
use std::io::Write;
use std::process::exit;
use std::string::ToString;
use colorize::AnsiColor;
use reqwest::get;
use url::Url;
use crate::expand_path;

const KNOWN_PUBLIC_LIBS: [(&str, &str, &str); 1] = [
    //  NAME                                 DOWNLOAD ADDRESS                                FILE NAME
    ("bscmath", "https://raw.githubusercontent.com/Txythread/smisc-bscmath/main/bscmath.s", "bscmath.s")
];

const PUBLIC_LIBS_DIR: &str = "pub-libs/";

/// Recursively goes throw all "!include"s and includes them recursively.
pub async fn perform_inclusions(code: String) -> String {
    let mut result: Vec<String> = vec![];

    let mut recursive_needed = false;

    for line in code.lines() {
        // Check if it's a normal line or an include statement
        if !line.starts_with("!include") {
            // Normal line, therefore push it.
            result.push(line.to_string());
            continue;
        }

        // Strip the !include to arrive at the file name
        let inclusion_argument = line.strip_prefix("!include").unwrap().trim().to_string();


        // Look if the file exists.
        if let Some(file) = expand_path(inclusion_argument.as_str()) {
            if let Some(file_contents) = read_to_string(file).ok() {
                result.push(file_contents);
                recursive_needed = true;
                continue;
            }
        }

        // Couldn't get file
        // Retry with URL
        // But check if it's already been downloaded first.
        let file_name = PUBLIC_LIBS_DIR.to_string() + inclusion_argument.split('/').last().unwrap();

        // Check if it exists already
        if let Some(file) = expand_path(file_name.as_str()) {
            if let Some(file_contents) = read_to_string(file).ok() {
                recursive_needed = true;
                result.push(file_contents);
                continue;
            }
        }

        // Doesn't exist
        // Make sure pub-libs exists
        if create_dir_all(PUBLIC_LIBS_DIR).is_err() {
            let error = "Couldn't create public libraries directory to store public libraries in.".red();
            eprintln!("{}", error);
            exit(104)
        }

        // Try to download the URL
        if let Some(include_source_code) = attempt_download(inclusion_argument.clone(), file_name.clone(), inclusion_argument.clone()).await {
            result.push(include_source_code);
            recursive_needed = true;
            continue;
        }

        // Couldn't download as a URL

        // Look if it's a well-known public library
        let package_info = KNOWN_PUBLIC_LIBS.iter().find(|&x| x.0 == inclusion_argument);

        if let Some(package_info) = package_info {
            // It is a well-known public lib
            let url = package_info.1.to_string();
            let file_name = PUBLIC_LIBS_DIR.to_string().clone() + package_info.2;

            // Look if it has been downloaded already.
            if let Some(file_contents) = read_to_string(file_name.clone()).ok() {
                result.push(file_contents);
                recursive_needed = true;
                continue;
            }

            // Doesn't exist -> Download it
            if let Some(include_source_code) = attempt_download(url, file_name, inclusion_argument.clone()).await {
                result.push(include_source_code);
                recursive_needed = true;
                continue;
            }
        }


        // No include option worked -> throw error
        let error = format!("Couldn't find dependency {}.", inclusion_argument).red();
        eprintln!("{}", error);
        exit(398)
    }

    let mut code = result.join("\n");

    if recursive_needed {
        code = Box::pin(perform_inclusions(code)).await;
    }

    code
}

fn print_failed(){
    let msg = "failed".red();
    println!("{}", msg);
}

async fn download(url: String, name: String) -> Result<(), Box<dyn error::Error>> {
    let response = get(url).await?;
    let content = response.bytes().await?;

    let mut downloaded_file = File::create(name)?;
    downloaded_file.write_all(&content)?;

    Ok(())
}

async fn attempt_download(line: String, file_name: String, download_name: String) -> Option<String> {
    if Url::parse(line.as_str()).is_ok() {
        let msg = format!("Trying to download: {} as {} ... ", download_name, file_name);
        print!("{}", msg);
        if download(line.clone(), file_name.to_string()).await.is_ok(){
            if let Some(d_file) = expand_path(file_name.as_str()) {
                if let Some(file_contents) = read_to_string(d_file).ok() {
                    let msg = "ok".green();
                    println!("{}", msg);

                    return Some(file_contents);
                } else {
                    print_failed();
                    let error = "Download of dependency failed, are you in the same dir as the file to assemble?".red();
                    eprintln!("{}", error);
                    exit(298)
                }
            } else {
                print_failed();
                let error = "Download of dependency failed, are you in the same dir as the file to assemble? Are you connected to the internet?".red();
                eprintln!("{}", error);
                exit(198)
            }
        }

        // Download failed
        print_failed();
        let error = format!("Download of dependency {} failed.\nAre you in the same dir as the file to assemble?\nAre you connected to the internet?", line).red();
        eprintln!("{}", error);
        exit(398)
    }

    None
}