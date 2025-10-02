use std::error;
use std::fs::*;
use std::io::Write;
use std::string::ToString;
use colorize::AnsiColor;
use reqwest::get;
use url::Url;
use crate::expand_path;
use crate::config::*;
use crate::util::code_error::{display_code_error, ErrorNotificationKind};
use crate::util::exit::{exit, ExitCode};
use crate::util::line_mapping::{ LineMap, LineInfo };


/// Recursively goes throw all "!include"s and includes them recursively.
pub async fn perform_inclusions(code: String) -> (String, LineMap) {
    let mut result: Vec<String> = vec![];
    let mut line_map = LineMap::new();


    let mut current_line_number = 1u32;
    for line in code.lines() {
        // Check if it's a normal line or an include statement
        if !line.starts_with("!include") {
            // Normal line, therefore push it.
            result.push(line.to_string());

            // Add this to the line map
            line_map.add_line(LineInfo::new_no_info(line.to_string(), current_line_number));
            current_line_number += 1;

            continue;
        }

        // Strip the !include to arrive at the file name
        let inclusion_argument = line.strip_prefix("!include").unwrap().trim().to_string();


        // Look if the file exists.
        if let Some(file) = expand_path(inclusion_argument.as_str()) {
            if let Some(file_contents) = read_to_string(file).ok() {
                let file_contents = Box::pin(perform_inclusions(file_contents)).await.0;
                result.push(file_contents.clone());

                // Update line mapping
                for x in file_contents.split('\n'){
                    line_map.add_line(LineInfo::new_no_info(x.to_string(), current_line_number));
                }

                current_line_number += 1;

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
                let file_contents = Box::pin(perform_inclusions(file_contents)).await.0;
                result.push(file_contents.clone());

                // Update line mapping
                for x in file_contents.split('\n'){
                    line_map.add_line(LineInfo::new_no_info(x.to_string(), current_line_number));
                }

                current_line_number += 1;

                continue;
            }
        }

        // Doesn't exist
        // Make sure pub-libs exists
        if create_dir_all(PUBLIC_LIBS_DIR).is_err() {
            exit("Couldn't create public libraries directory to store public libraries in.".to_string(), ExitCode::ReadWriteError);
        }

        // Try to download the URL
        if let Some(include_source_code) = attempt_download(inclusion_argument.clone(), file_name.clone(), inclusion_argument.clone()).await {
            let file_contents = Box::pin(perform_inclusions(include_source_code)).await.0;
            result.push(file_contents.clone());

            // Update line mapping
            for x in file_contents.split('\n'){
                line_map.add_line(LineInfo::new_no_info(x.to_string(), current_line_number));
            }

            current_line_number += 1;

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
                let file_contents = Box::pin(perform_inclusions(file_contents)).await.0;
                result.push(file_contents.clone());

                // Update line mapping
                for x in file_contents.split('\n'){
                    line_map.add_line(LineInfo::new_no_info(x.to_string(), current_line_number));
                }

                current_line_number += 1;

                continue;
            }

            // Doesn't exist -> Download it
            if let Some(include_source_code) = attempt_download(url, file_name, inclusion_argument.clone()).await {
                let file_contents = Box::pin(perform_inclusions(include_source_code)).await.0;
                result.push(file_contents.clone());

                // Update line mapping
                for x in file_contents.split('\n'){
                    line_map.add_line(LineInfo::new_no_info(x.to_string(), current_line_number));
                }

                current_line_number += 1;

                continue;
            }
        }


        // No include option worked -> throw error
        let mut code: Vec<String> = vec![];

        // Create newlines except for the last (current) line

        for _ in 0..current_line_number{
            code.push(String::new());
        }

        code.push(line.to_string());

        let column = 9; // The length of the !include statement
        display_code_error(ErrorNotificationKind::Error, current_line_number as i32, Some(column), Some(inclusion_argument.len() as u32), "Library not found".to_string(), "There was no file found with that name, no URL with that name could be reached and there's no such well-known public library.".to_string(), code);
        line_map.errors_count += 1;
        line_map.stop_after_step = true;
    }

    let code = result.join("\n");


    line_map.exit_if_needed();

    (code, line_map)
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
                    exit("Download of dependency failed, are you in the same dir as the file to assemble?".to_string(), ExitCode::Other);
                }
            } else {
                print_failed();
                exit("Download of dependency failed, are you in the same dir as the file to assemble? Are you connected to the internet?".to_string(), ExitCode::Other);
            }
        }

        // Download failed
        print_failed();
        exit(format!("Download of dependency {} failed.\nAre you in the same dir as the file to assemble?\nAre you connected to the internet?", line).to_string(), ExitCode::Other);
    }

    None
}