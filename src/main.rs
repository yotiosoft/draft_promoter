use std::fs;
use std::path::Path;
use std::env;
use regex::Regex;
use chrono::Local;

mod parse;

struct MdFileStruct {
    name: String,
    modified: std::time::SystemTime,
}

fn get_destination_dir_files(destination_dir: String) -> Vec<String> {
    let mut list: Vec<String> = Vec::new();
    let mut files = fs::read_dir(&destination_dir).expect("cannot read directory");
    while let Some(file) = files.next() {
        let file = file.expect("cannot get file");
        let file_name = file.file_name().into_string().expect("cannot convert file name");
        if file_name.ends_with(".md") {
            list.push(file_name);
        }
    }
    list
}

fn get_md_file_struct_list(dir: &String) -> Vec<MdFileStruct> {
    let mut md_files: Vec<MdFileStruct> = Vec::new();
    let mut files = fs::read_dir(&dir).expect("cannot read directory");
    while let Some(file) = files.next() {
        let file = file.expect("cannot get file");
        let file_name = file.file_name().into_string().expect("cannot convert file name");
        if file_name.ends_with(".md") {
            if let Ok(modified) = file.metadata().expect("cannot get metadata").modified() {
                md_files.push(MdFileStruct {
                    name: file_name,
                    modified: modified,
                });
            }
        }
    }
    md_files
}

fn get_title(content: &String) -> Option<String> {
    let re = Regex::new(r#"---\s((.|\s)*?)\s---"#).expect("cannot create regex");
    match re.find(&content) {
        Some(m) => {
            // println!("Header found `{}` at {}-{}", m.as_str(), m.start(), m.end());
            let title_re = Regex::new(r#"title: (.*)"#).expect("cannot create regex");
            let title = title_re.captures(m.as_str()).expect("cannot find title")[1].to_string();
            
            let ret_name = title.trim_end().replace("\"", "") + ".md";
            println!("title: {}", ret_name);
            Some(ret_name)
        },
        None => {
            println!("Header not found");
            None
        },
    }
}

fn main() {
    // 引数受け取り
    let arg_struct = parse::parser();

    // 既定値
    // writing_posts にいる場合
    let current_dir = env::current_dir().expect("cannot get current dir");
    let (default_from, default_to) = if current_dir.ends_with("writing_posts") {
        (String::from("./"), String::from("../_posts/"))
    }
    // _posts にいる場合
    else if current_dir.ends_with("_posts") {
        (String::from("../writing_posts/"), String::from("./"))
    }
    // どちらでもない場合
    else {
        (String::from("./writing_posts/"), String::from("./_posts/"))
    };

    // ディレクトリの指定
    let from = if arg_struct.from.is_empty() {
        default_from
    }
    else {
        arg_struct.from
    };
    let to = if arg_struct.to.is_empty() {
        default_to
    }
    else {
        arg_struct.to
    };

    // ディレクトリの存在確認
    if !Path::new(&from).exists() {
        println!("directory {} does not exist.", from);
        return;
    }
    if !Path::new(&to).exists() {
        println!("directory {} does not exist.", to);
        return;
    }

    // コピー元ファイル名が空の場合 -> 最新のファイルを採用
    let source_file_string = if arg_struct.source_file.is_empty() {
        let mut md_files = get_md_file_struct_list(&from);
        // 最新のファイルを採用
        md_files.sort_by(|a, b| b.modified.cmp(&a.modified));
        md_files[0].name.clone()
    }
    else {
        arg_struct.source_file.clone()
    };
    let source_file = Path::new(&from).join(&source_file_string);

    // コピー先ファイル名
    let destination_file = if arg_struct.destination_file.is_empty() {
        // markdown ファイルを開いて、タイトルを抽出する
        let content = fs::read_to_string(&source_file).expect("cannot read file");

        let md_name = match get_title(&content) {
            Some(title) => title,
            None => source_file_string.clone(),
        };
        
        // コピー先のファイルリストを取得
        let destination_dir_files = get_destination_dir_files(to.clone());
        // コピー先にタイトルと同名のファイルが存在するなら、そのファイル名を採用（置き換え = 日付を合わせる）
        let mut exist_file = String::new();
        for file in destination_dir_files {
            if file.contains(&md_name) {
                exist_file = file;
                break;
            }
        }
        if !exist_file.is_empty() {
            exist_file
        }
        else {
            // 日付を付与
            let date = Local::now().format("%Y-%m-%d").to_string();
            date + "-" + &md_name
        }
    }
    else {
        arg_struct.destination_file
    };
    if destination_file.is_empty() {
        println!("source file name is empty.");
        return;
    }

    // ファイルパスの用意
    let destination_file = Path::new(&to).join(&destination_file);

    // ファイルの存在確認
    if !source_file.exists() {
        println!("file {} does not exist.", source_file.display());
        return;
    }

    // ファイルのコピー
    print!("copy {} -> {} ", source_file.display(), destination_file.display());
    fs::copy(&source_file, &destination_file).expect("cannot copy file");
    println!("done.");

    // ソースファイルの削除
    if arg_struct.remove_source {
        fs::remove_file(&source_file).expect("cannot remove file");
        println!("remove {} done.", source_file.display());
    }
}
