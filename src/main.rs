#![feature(new_uninit)]
#![feature(downcast_unchecked)]
#![feature(cell_update)]
#![feature(str_from_raw_parts)]
#![feature(extend_one)]

use std::{
    collections::HashSet,
    fs::{self},
    path::Path,
    process::{exit, Command},
    sync::{Arc, LazyLock},
};

use ansi_term::Colour::Red;
use args::{Args, LanguageOption};
use clap::Parser;
use config::{
    CFG, CONFIG_COLLECTION_PATH, LANG_OUTPUT_DIR, OUTPUT_ENUM_CODE_DIR, OUTPUT_SCRIPT_CODE_DIR,
    OUTPUT_SERVER_ENUM_CODE_DIR, OUTPUT_SERVER_SCRIPT_CODE_DIR, REF_TEXT_DIR, SOURCE_XLSXS_DIR,
};
use table::{Generator, TableEntity};

mod args;
mod config;
mod error;
mod lex;
mod parser;
mod preconfig;
mod table;
mod types;
mod util;

pub static THREADS: LazyLock<rayon::ThreadPool> = LazyLock::new(|| {
    let available_parallelism = match std::thread::available_parallelism() {
        Ok(num) => num.get(),
        Err(e) => {
            eprintln!(
                "Unable to get the number of available parallelism units: {}",
                e
            );
            32
        }
    };
    println!("use {} threads", available_parallelism);
    rayon::ThreadPoolBuilder::new()
        .num_threads(available_parallelism)
        .build()
        .unwrap()
});

fn create_dest_dirs(args: &Args) {
    if let Err(_) = fs::metadata(unsafe { OUTPUT_SCRIPT_CODE_DIR }) {
        if let Err(_) = fs::create_dir_all(unsafe { OUTPUT_SCRIPT_CODE_DIR }) {
            exit(-1)
        }
    }

    if let Err(_) = fs::metadata(unsafe { OUTPUT_ENUM_CODE_DIR }) {
        if let Err(_) = fs::create_dir_all(unsafe { OUTPUT_ENUM_CODE_DIR }) {
            exit(-1)
        }
    }

    if let Err(_) = fs::metadata(unsafe { OUTPUT_SERVER_SCRIPT_CODE_DIR }) {
        if let Err(_) = fs::create_dir_all(unsafe { OUTPUT_SERVER_SCRIPT_CODE_DIR }) {
            exit(-1)
        }
    }

    if let Err(_) = fs::metadata(unsafe { OUTPUT_SERVER_ENUM_CODE_DIR }) {
        if let Err(_) = fs::create_dir_all(unsafe { OUTPUT_SERVER_ENUM_CODE_DIR }) {
            exit(-1)
        }
    }

    if let Err(_) = fs::metadata(unsafe { REF_TEXT_DIR }) {
        if let Err(_) = fs::create_dir_all(unsafe { REF_TEXT_DIR }) {
            exit(-1)
        }
    }

    let mut lang_path = args.output_lang_dir.clone();
    let option = args.loption.as_str().into();
    match option {
        LanguageOption::CN => lang_path.push_str("/Language_CN"),
        LanguageOption::CNH => lang_path.push_str("/Language_CNH"),
        LanguageOption::EN => lang_path.push_str("/Language_EN"),
        LanguageOption::JP => lang_path.push_str("/Language_JP"),
        LanguageOption::Invalid => {
            eprintln!(
                "{}",
                Red.bold().paint(format!(
                    "[Error]: Invalid language option: {}",
                    args.loption
                ))
            );
        }
    }

    if let Err(_) = fs::metadata(lang_path.as_str()) {
        if let Err(_) = fs::create_dir_all(lang_path.as_str()) {
            exit(-1)
        }
    }

    unsafe {
        LANG_OUTPUT_DIR = Box::leak(lang_path.into_boxed_str());
    }
}

fn update_git() {
    let output = Command::new("cmd")
        .arg("/C")
        .arg(format!(r#"{}\update.bat"#, unsafe { SOURCE_XLSXS_DIR }))
        .arg(format!("{}", unsafe { SOURCE_XLSXS_DIR }))
        .output()
        .expect("Failed to execute command");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}

fn load_tables<P: AsRef<Path>>(
    dir: P,
    excluded: Arc<ExcludedFolders<'static>>,
    tables: Arc<util::AtomicLinkedList<TableEntity>>,
) -> Result<(), error::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .ok_or::<error::Error>(
                format!("path terminates in .. : `{:?}`", path.as_os_str()).into(),
            )?
            .to_str()
            .ok_or::<error::Error>("invalid unicode".into())?;

        // ban-lists
        if config::TABLE_XLSX_FILTER.contains(file_name) {
            continue;
        }

        if path.is_dir() && !file_name.starts_with('.') && !excluded.0.contains(file_name) {
            let excluded_clone = excluded.clone();
            let tables_clone = tables.clone();
            load_tables(path, excluded_clone, tables_clone)?;
        } else if path
            .extension()
            .is_some_and(|x| x.to_str().is_some_and(|x| x == CFG.source_table_suffix))
            && !file_name.starts_with('~')
        {
            let idx = file_name
                .find('.')
                .ok_or::<error::Error>("can't find `.` in xlsx file name".into())?;
            THREADS.install(|| {
                match util::load_execl_table(&path, &file_name[..idx]) {
                    Ok(v) => {
                        tables.push(v);
                    },
                    Err(e) => {
                        eprintln!("{}", Red.bold().paint(format!("load_tables failed: {}", e)));
                    },
                }
            });
        }
    }
    Ok(())
}

fn build(
    tables: Arc<util::AtomicLinkedList<TableEntity>>,
    loption: &str,
) -> Result<(), error::Error> {
    // SAFETY: no data-race here, read-only
    let tables = unsafe {
        Arc::into_inner(tables)
            .ok_or::<error::Error>("".into())?
            .into_unsafe_vector()
    };
    let genarator = Generator {
        entities: tables,
        loption,
    };
    genarator.build()?;
    Ok(())
}

#[derive(Default)]
struct ExcludedFolders<'a>(HashSet<&'a str>);

fn main() {
    //let _ = std::io::Read::read(&mut std::io::stdin(), &mut [0; 1]);
    let args = Args::parse();
    create_dest_dirs(&args);
    unsafe {
        OUTPUT_SCRIPT_CODE_DIR = Box::leak(args.output_script_dir.into_boxed_str());
        OUTPUT_ENUM_CODE_DIR = Box::leak(args.output_enum_dir.into_boxed_str());
        OUTPUT_SERVER_SCRIPT_CODE_DIR = Box::leak(args.output_server_script_dir.into_boxed_str());
        OUTPUT_SERVER_ENUM_CODE_DIR = Box::leak(args.output_server_enum_dir.into_boxed_str());
        SOURCE_XLSXS_DIR = Box::leak(args.src_table_dir.into_boxed_str());
        REF_TEXT_DIR = Box::leak(args.ref_mapping_dir.into_boxed_str());
        CONFIG_COLLECTION_PATH = Box::leak(args.config_collection_path.into_boxed_str());
    }

    match args.command {
        args::Command::Build => {
            println!("[Begin]");

            // excluded folders
            let mut excluded_folders = ExcludedFolders::default();
            let exstr = Box::leak(args.excluded_folders.replace(" ", "").into_boxed_str());
            for v in exstr.split(",") {
                excluded_folders.0.insert(v);
            }
            let excluded = Arc::new(excluded_folders);

            // pull origin
            if args.update_git {
                update_git();
            }

            if !args.lstring {
                let tables = Arc::new(util::AtomicLinkedList::new());
                // load regular tables
                THREADS.join(
                    || println!("Loading tables..."),
                    || match load_tables(unsafe { SOURCE_XLSXS_DIR }, excluded, tables.clone()) {
                        Ok(_) => match build(tables, args.loption.as_str()) {
                            Err(e) => eprintln!(
                                "{}",
                                Red.bold().paint(format!("tables build failed: {}", e))
                            ),
                            _ => {}
                        },
                        Err(e) => {
                            eprintln!("{}", Red.bold().paint(format!("load_tables failed: {}", e)));
                            exit(-1);
                        }
                    },
                );
            } else {
                match util::load_execl_table(
                    format!("{}/LString.xlsx", unsafe { SOURCE_XLSXS_DIR }),
                    "LString",
                ) {
                    Ok(entity) => {
                        let generator = Generator {
                            entities: vec![entity],
                            loption: &args.loption,
                        };
                        if let Err(e) = generator.build() {
                            eprintln!("{}", Red.bold().paint(format!("Build failed: {}", e)));
                        }
                    }
                    Err(e) => eprintln!(
                        "{}",
                        Red.bold()
                            .paint(format!("Load table failed when building language: {}", e))
                    ),
                }
            }

            println!("[End]");
            println!("\nPress any key to exit the program...");
            let mut empty = [0; 1];
            let _ = std::io::Read::read(&mut std::io::stdin(), &mut empty);
        }
        args::Command::Clean => {
            if let Err(e) = fs::remove_dir_all(unsafe { OUTPUT_SCRIPT_CODE_DIR }) {
                eprintln!("{}", Red.bold().paint(format!("{}", e)));
                exit(-1)
            }
            if let Err(e) = fs::remove_dir_all(unsafe { OUTPUT_ENUM_CODE_DIR }) {
                eprintln!("{}", Red.bold().paint(format!("{}", e)));
                exit(-1)
            }
            if let Err(e) = fs::remove_dir_all(unsafe { REF_TEXT_DIR }) {
                eprintln!("{}", Red.bold().paint(format!("{}", e)));
                exit(-1)
            }
        }
    }
}

#[test]
fn test() {
    let mut display = String::new();
    let ty = parser::parse_type("enum LifeRecord.ScoreType", 0, 0).unwrap();
    let val = parser::parse_assign_with_type(&ty, "c", None, None).unwrap();
    if val.check() {
        let _ = val.code_fmt(&mut display);
    } else {
        println!("check failed");
    }
    println!("{}", display);
}

#[test]
fn generate() {
    let path = "D:\\taiwu\\config\\GlobalConfig.xlsx";
    let table = util::load_execl_table(path, "GlobalConfig").unwrap();
    let ctx = std::sync::Arc::new(table::BuildContext::default());
    match table.view(ctx.clone()).unwrap().build(ctx.as_ref()) {
        Ok(_) => {}
        Err(e) => println!("{}", e),
    };
}

#[test]
fn test_split() {
    let val = "{hairup_double_l_front,hair/female_double_hairup_22,{hairup_double_r_front,hair/female_double_hairup_22}";
    println!("{:?}", util::split(val));
}
