use backhand::{FilesystemReader, InnerNode};
use std::io::BufReader;
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

fn main() {
    let diskpath = std::path::Path::new("gluon-ffh-vH39.pre-x86-64-sysupgrade.img");
    let compressed_diskpath = std::path::Path::new("gluon-ffh-vH39.pre-x86-64-sysupgrade.img.gz");

    let disk = gpt::GptConfig::new()
        .open(diskpath)
        .expect("failed to open disk");

    let blocksize = 512;
    let root_partition_index = 2;

    let partitions = disk.partitions();
    let partition = partitions.get(&root_partition_index).unwrap();

    // read
    let file = BufReader::new(File::open(diskpath).unwrap());
    let read_filesystem =
        FilesystemReader::from_reader_with_offset(file, partition.first_lba * blocksize).unwrap();

    let gluon_release =
        read_file_to_string(&read_filesystem, "/lib/gluon/release").map(|f| f.trim().to_owned());

    let gluon_version = read_file_to_string(&read_filesystem, "/lib/gluon/gluon-version")
        .map(|f| f.trim().to_owned());

    let autoupdater_default_branch =
        read_file_to_string(&read_filesystem, "/lib/gluon/autoupdater/default_branch")
            .map(|f| f.trim().to_owned());

    let autoupdater_default_enabled =
        read_file_to_string(&read_filesystem, "/lib/gluon/autoupdater/default_enabled").is_some();

    if let Some(gluon_version) = gluon_version {
        println!("gluon-version: {:}", gluon_version);
    }

    if let Some(gluon_release) = gluon_release {
        println!("gluon-release: {:}", gluon_release);
    }

    if let Some(autoupdater_default_branch) = autoupdater_default_branch {
        println!(
            "autoupdater-default-branch: {:}",
            autoupdater_default_branch
        );
    }

    if autoupdater_default_enabled {
        println!("autoupdater-default-enabled: true");
    } else {
        println!("autoupdater-default-enabled: false");
    }
}
