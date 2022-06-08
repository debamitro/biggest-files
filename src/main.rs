use std::env;
use std::fs;
use std::path::Path;
use std::vec::Vec;

fn findsize(arg1: &Path) -> u64 {
    let mut size: u64 = 0;
    if let Ok(itr) = fs::read_dir(arg1) {
        for e in itr {
            if let Ok(entry) = e {
                if let Ok(m) = entry.metadata() {
                    if m.is_dir() {
                        size += findsize(&entry.path());
                    } else if m.is_file() {
                        size += m.len();
                    }
                }
            }
        }
    }

    return size / 1024 / 1024;
}

struct DirAndSize {
    name: String,
    size: u64,
}

impl DirAndSize {
    fn new(name_: &str, size_: u64) -> DirAndSize {
        DirAndSize {
            name: String::from(name_),
            size: size_,
        }
    }
}

fn findbiggestfiles(dirname: &str, cutoff: u64) {
    if let Ok(itr) = fs::read_dir(dirname) {
        let mut allsizes: Vec<DirAndSize> = Vec::new();
        let mut totalsize: u64 = 0;
        for e in itr {
            if let Ok(entry) = e {
                if let Ok(m) = entry.metadata() {
                    if m.is_dir() {
                        let dirsize = findsize(&entry.path());
                        totalsize += dirsize;
                        if dirsize > cutoff {
                            if let Some(s) = entry.path().to_str() {
                                allsizes.push(DirAndSize::new(s, dirsize));
                            }
                        }
                    }
                }
            }
        }

        allsizes.sort_by(|a, b| b.size.cmp(&a.size));

        println!("||Directory\t|Size in MB||");
        println!("|{}\t|{}|", dirname, totalsize);
        for a in allsizes {
            println!("|{}\t|{}|", a.name, a.size);
        }
    }
}

fn main() {
    if let Some(arg1) = env::args().nth(1) {
        findbiggestfiles(&arg1, 2);
    }
}
