use std::fs::File;
use std::io::Write;
use std::process::exit;
use prost_build::Config;

const PROTOC_URL_W64: &str = "https://github.com/protocolbuffers/protobuf/releases/download/v26.1/protoc-26.1-win64.zip";
const PROTOC_URL_W32: &str = "https://github.com/protocolbuffers/protobuf/releases/download/v26.1/protoc-26.1-win32.zip";
const PROTOC_URL_LINUX_ARM64: &str = "https://github.com/protocolbuffers/protobuf/releases/download/v26.1/protoc-26.1-linux-aarch_64.zip";
const PROTOC_URL_LINUX_X64: &str = "https://github.com/protocolbuffers/protobuf/releases/download/v26.1/protoc-26.1-linux-x86_64.zip";
const PROTOC_URL_LINUX_X32: &str = "https://github.com/protocolbuffers/protobuf/releases/download/v26.1/protoc-26.1-linux-x86_32.zip";
const PROTOC_URL_OSX: &str = "https://github.com/protocolbuffers/protobuf/releases/download/v26.1/protoc-26.1-osx-universal_binary.zip";

fn main() {
    check_protoc();

    let mut config = Config::new();
    config
        .out_dir("src/pb")
        .include_file("session");

    let mut protos = Vec::new();
    for entry in walkdir::WalkDir::new("protos") {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            if let Some(extension) = entry.path().extension() {
                if extension == "proto" {
                    let proto_file = entry.path().to_str().unwrap();
                    protos.push(proto_file.to_string());
                }
            }
        }
    }

    config.compile_protos(protos.as_slice(), &["protos"]).unwrap();
}

fn check_protoc() {
    use std::process::Command;

    let output = Command::new("protoc")
        .arg("--version")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("protoc version: {}", version);
        } else {
            try_install_protoc();
        }
    } else {
        try_install_protoc();
    }
}

fn try_install_protoc() {
    use std::env;
    use std::fs;

    if env::var("PROTOC").is_ok() {
        println!("protoc is already installed");
        return;
    }


    let target = env::var("TARGET").unwrap();
    let is_win = target.contains("windows");

    // if bin/protoc exists, return
    let current_dir = env::current_dir().unwrap();
    let protoc_path = current_dir.join("bin").join(if is_win { "protoc.exe" } else { "protoc" });
    if protoc_path.exists() {
        println!("protoc is already installed");
        env::set_var("PROTOC", protoc_path);
        return;
    }

    let url = match target.as_str() {
        "x86_64-pc-windows-msvc" => PROTOC_URL_W64,
        "i686-pc-windows-msvc" => PROTOC_URL_W32,
        "aarch64-unknown-linux-gnu" => PROTOC_URL_LINUX_ARM64,
        "x86_64-unknown-linux-gnu" => PROTOC_URL_LINUX_X64,
        "i686-unknown-linux-gnu" => PROTOC_URL_LINUX_X32,
        "x86_64-apple-darwin" => PROTOC_URL_OSX,
        _ => {
            println!("Unsupported target: {}", target);
            exit(1);
        }
    };

    let resp = reqwest::blocking::get(url).unwrap();
    let status = resp.status();
    if !status.is_success() {
        println!("Failed to download protoc: {}", status);
        exit(1);
    }
    // write it into protoc.zip, if it exists, overwrite it.
    let mut file = File::create("protoc.zip").unwrap();
    file.write_all(&resp.bytes().unwrap()).unwrap();

    let file = File::open("protoc.zip").unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    archive.extract("")
        .expect("Failed to extract protoc");

    // remove `protoc.zip` and `include` dir
    fs::remove_file("protoc.zip").unwrap();
    fs::remove_file("readme.txt").unwrap();
    fs::remove_dir_all("include").unwrap();

    let protoc = if is_win {
        "protoc.exe"
    } else {
        "protoc"
    };

    // add the protoc to `PROTOC`
    let current_dir = env::current_dir().unwrap();
    let protoc_path = current_dir.join("bin").join(protoc);
    println!("protoc path: {:?}", protoc_path);
    env::set_var("PROTOC", protoc_path);
}