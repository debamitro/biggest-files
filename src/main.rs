use std::env;
use std::fs;
//use std::path::Path;
use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;
use std::vec::Vec;

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

fn findsize(toplevel: &str, arg1: &str, chan: &Sender<DirAndSize>) {
    let mut size: u64 = 0;
    let txi = chan.clone();
    if let Ok(itr) = fs::read_dir(arg1) {
        for e in itr {
            if let Ok(entry) = e {
                if let Ok(m) = entry.metadata() {
                    if m.is_dir() {
                        findsize(toplevel, entry.path().to_str().unwrap(), chan);
                    } else if m.is_file() {
                        size += m.len();
                    }
                }
            }
        }
    }

    let arg1_copy = String::from(toplevel);
    thread::spawn(move || {
        txi.send(DirAndSize::new(&arg1_copy, size / 1024 / 1024))
            .unwrap();
    });
}

fn findbiggestfiles(dirname: &str, cutoff: u64) {
    if let Ok(itr) = fs::read_dir(dirname) {
        let mut allsizes: HashMap<String, DirAndSize> = HashMap::<String, DirAndSize>::new();
        let mut totalsize: u64 = 0;

        let (tx, rx) = channel();
        for e in itr {
            if let Ok(entry) = e {
                if let Ok(m) = entry.metadata() {
                    if m.is_dir() {
                        findsize(
                            entry.path().to_str().unwrap(),
                            entry.path().to_str().unwrap(),
                            &tx,
                        );
                    } else if m.is_file() {
                        let txi = tx.clone();

                        thread::spawn(move || {
                            txi.send(DirAndSize::new(
                                entry.path().to_str().unwrap(),
                                m.len() / 1024 / 1024,
                            ))
                            .unwrap();
                        });
                    }
                }
            }
        }

        drop(tx);

        let mut lines_printed: u64 = 0;
        while let Ok(msg) = rx.recv() {
            totalsize += msg.size;
            let entrysize = if let Some(entry) = allsizes.get(&msg.name) {
                entry.size + msg.size
            } else {
                msg.size
            };

            allsizes.insert(
                String::from(&msg.name),
                DirAndSize::new(&msg.name, entrysize),
            );

            let mut sortedsizes = Vec::new();
            for v in allsizes.values() {
                if v.size > cutoff {
                    sortedsizes.push(v);
                }
            }
            sortedsizes.sort_by(|a, b| b.size.cmp(&a.size));

            if lines_printed > 0 {
                print!("\x1b[{}A\x1b[0J", lines_printed);
            }
            println!("\x1b[31m||File/directory\t|Size in MB||\x1b[0m");
            lines_printed = 1;
            for a in &sortedsizes {
                println!("|{}\t|{}|", a.name, a.size);
                lines_printed += 1;
            }
        }

        if lines_printed > 0 {
            print!("\x1b[{}A\x1b[0J", lines_printed);
        }
        println!("\x1b[31m||Directory\t|Size in MB||\x1b[0m");
        println!("||Directory\t|Size in MB||");
        println!("|{}\t|{}|", dirname, totalsize);
        let mut sortedsizes = Vec::new();
        for v in allsizes.values() {
            if v.size > cutoff {
                sortedsizes.push(v);
            }
        }
        sortedsizes.sort_by(|a, b| b.size.cmp(&a.size));
        for a in &sortedsizes {
            println!("|{}\t|{}|", a.name, a.size);
        }
    }
}

fn main() {
    let mut args = env::args();
    if args.len() > 2 {
        let _arg0 = args.next().unwrap();
        let arg1 = args.next().unwrap();
        let arg2 = args.next().unwrap();
        findbiggestfiles(&arg1, u64::from_str_radix(&arg2, 10).unwrap());
    } else {
        println!("Usage:\nbiggest-files <directory> <minimum-size-in-MB>");
    }
}
