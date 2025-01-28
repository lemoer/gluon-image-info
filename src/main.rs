use backhand::{FilesystemReader, InnerNode};
use bootsector::pio::ReadAt;
use bootsector::{list_partitions, Options};
use std::env;
use std::io;
use std::io::{BufReader, Cursor};
use std::{fs::File, io::Read};

fn read_file_to_string(read_filesystem: &FilesystemReader, fullpath: &str) -> Option<String> {
    for node in read_filesystem.files() {
        let Some(current_fullpath) = node.fullpath.to_str() else {
            // If an inode doesn't have a fullpath, we do not care.
            continue;
        };

        if current_fullpath != fullpath {
            continue;
        }

        let InnerNode::File(file) = &node.inner else {
            // skip everyhting that is not a file
            continue;
        };

        let mut reader = read_filesystem.file(&file).reader();

        let mut s = String::new();
        return match reader.read_to_string(&mut s) {
            Ok(_) => Some(s),
            Err(_) => None, // non utf-8 file
        };
    }

    return None;
}

// ReadAt wrapper for Vec<u8>

struct ReadAtVec<'a> {
    data: &'a Vec<u8>,
}

impl<'a> ReadAtVec<'a> {
    pub fn new(data: &'a Vec<u8>) -> Self {
        Self { data }
    }
}

impl<'a> ReadAt for ReadAtVec<'a> {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        let start = offset as usize;
        let end = start + buf.len();
        if end > self.data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Read past end of buffer",
            ));
        }
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

    let file = File::open(compressed_diskpath).expect("Opening file failed");
    let mut decoder = flate2::read::GzDecoder::new(file);
    let mut buffer = Vec::new();
    decoder
        .read_to_end(&mut buffer)
        .expect("Decoding gzip failed");

    let read_at = ReadAtVec::new(&buffer);
    let partitions =
        list_partitions(&read_at, &Options::default()).expect("Listing partitions failed");

    let Some(part) = partitions.get(1) else {
        println!("Not enough partitions!");
        return;
    };

    let cursor = Cursor::new(buffer);

    // read
    let reader = BufReader::new(cursor);
    let read_filesystem = FilesystemReader::from_reader_with_offset(reader, part.first_byte)
        .expect("Opening squashfs failed");

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

    let mut openwrt_release: Option<String> = None;
    if let Some(openwrt_releaseinfo) = openwrt_releaseinfo {
        openwrt_releaseinfo.lines().for_each(|line| {
            if line.starts_with("DISTRIB_RELEASE=") {
                openwrt_release = Some(
                    line.trim_start_matches("DISTRIB_RELEASE='")
                        .trim_end_matches("'")
                        .to_owned(),
                );
            }
        });
    }

    let maybe_info = |x: Option<String>| x.unwrap_or("n/a".to_owned());
    println!("openwrt-release: {}", maybe_info(openwrt_release));
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
