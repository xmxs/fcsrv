use std::path::PathBuf;

use serde_json::{json, Value};
use tokio::fs::read_dir;

#[tokio::main]
async fn main() {
    let mut dir = read_dir("images").await.unwrap();
    let mut dir_child = Vec::new();
    while let Some(entry) = dir.next_entry().await.unwrap() {
        let filename = entry.file_name().to_str().unwrap().to_string();
        dir_child.push((entry.path(), filename));
    }
    for (path, filename) in dir_child {
        group_example(path, &filename).await
    }
}

async fn group_example(path: PathBuf, typed: &str) {
    // 读取images文件夹所有.jpg文件名，返回Vec<String>
    let mut dir = read_dir(&path).await.unwrap();
    let mut files = Vec::new();
    while let Some(entry) = dir.next_entry().await.unwrap() {
        let filename = entry.file_name().to_str().unwrap().to_string();
        if filename.contains(".jpg") {
            files.push(filename);
        }
    }

    let client = reqwest::Client::new();
    for chunk in files.chunks(5) {
        let paths: Vec<PathBuf> = chunk.iter().map(|p| PathBuf::from(&path).join(p)).collect();
        for filepath in paths {
            let bytes = tokio::fs::read(&filepath).await.unwrap();
            #[allow(deprecated)]
            let image = base64::encode(bytes);
            let resp = client
                .post("http://127.0.0.1:8000/task")
                .json(&json!(
                    {
                        "type": typed,
                        "images": [image],
                    }
                ))
                .send()
                .await
                .unwrap();
            let json = resp.json::<Value>().await.unwrap();
            for ele in json.get("objects").unwrap().as_array().unwrap().iter() {
                let guess = ele.as_number().unwrap();
                let guess_file = filepath.clone().to_str().unwrap().replace(".jpg", ".txt");
                let ok_guess = tokio::fs::read_to_string(guess_file).await.unwrap();
                if guess.as_u64().unwrap() != ok_guess.parse::<u64>().unwrap() {
                    println!("{}: {} != {}", filepath.to_string_lossy(), guess, ok_guess);
                } else {
                    println!("{}: {} == {}", filepath.to_string_lossy(), guess, ok_guess);
                }
            }
        }
        println!("-------------------")
    }
}
