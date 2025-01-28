use backhand::{FilesystemReader, InnerNode};
use bootsector::pio::ReadAt;
use bootsector::{list_partitions, Options};
use std::env;
use std::io;
use std::io::{BufReader, Cursor};
use std::{fs::File, io::Read};

fn read_file_to_string(read_filesystem: &FilesystemReader, filename: &str) -> Option<String> {
    for node in read_filesystem.files() {
        if node.fullpath.to_str().unwrap() != filename {
            continue;
        }
        // extract
        match &node.inner {
            InnerNode::File(file) => {
                let x = read_filesystem.file(&file);
                let mut reader = x.reader();

                let mut s = String::new();
                reader.read_to_string(&mut s).unwrap();

                return Some(s);
            }
            _ => (),
        }
    }

    return None;
}

// ReadAt wrapper for Vec<u8>

struct ReadAtVec {
    data: Vec<u8>,
}

impl ReadAtVec {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl ReadAt for ReadAtVec {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        let start = offset as usize;
        let end = start + buf.len();
        buf.copy_from_slice(&self.data[start..end]);
        Ok(buf.len())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <image>", args[0]);
        return;
    }

    let compressed_diskpath = std::path::Path::new(&args[1]);

    if !compressed_diskpath.exists() {
        println!("File {:?} does not exist!", compressed_diskpath);
        return;
    }

    let file = File::open(compressed_diskpath).unwrap();
    let mut decoder = flate2::read::GzDecoder::new(file);
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer).unwrap();

    let x = ReadAtVec::new(buffer.clone());
    // let reader = ...;
    let partitions = list_partitions(&x, &Options::default()).unwrap();
    if partitions.len() < 2 {
        println!("Not enough partitions!");
        return;
    }
    let part = &partitions[1];

    let cursor2 = Cursor::new(buffer);

    // read
    let file = BufReader::new(cursor2);
    let read_filesystem = FilesystemReader::from_reader_with_offset(file, part.first_byte).unwrap();

    let gluon_release =
        read_file_to_string(&read_filesystem, "/lib/gluon/release").map(|f| f.trim().to_owned());

    let gluon_version = read_file_to_string(&read_filesystem, "/lib/gluon/gluon-version")
        .map(|f| f.trim().to_owned());

    let site_version = read_file_to_string(&read_filesystem, "/lib/gluon/site-version")
        .map(|f| f.trim().to_owned());

    let autoupdater_default_branch =
        read_file_to_string(&read_filesystem, "/lib/gluon/autoupdater/default_branch")
            .map(|f| f.trim().to_owned());

    let autoupdater_default_enabled =
        read_file_to_string(&read_filesystem, "/lib/gluon/autoupdater/default_enabled").is_some();

    let openwrt_releaseinfo =
        read_file_to_string(&read_filesystem, "/etc/openwrt_release").map(|f| f.trim().to_owned());

    if let Some(openwrt_releaseinfo) = openwrt_releaseinfo {
        openwrt_releaseinfo.lines().for_each(|line| {
            if line.starts_with("DISTRIB_RELEASE=") {
                println!(
                    "openwrt-release: {}",
                    line.trim_start_matches("DISTRIB_RELEASE='")
                        .trim_end_matches("'")
                );
            }
        });
    }

    let maybe_info = |x: Option<String>| x.unwrap_or("n/a".to_owned());

    println!("gluon-version: {:}", maybe_info(gluon_version));
    println!("gluon-release: {:}", maybe_info(gluon_release));
    println!("site-version: {:}", maybe_info(site_version));
    println!(
        "autoupdater-default-branch: {:}",
        maybe_info(autoupdater_default_branch)
    );

    if autoupdater_default_enabled {
        println!("autoupdater-default-enabled: true");
    } else {
        println!("autoupdater-default-enabled: false");
    }
}
