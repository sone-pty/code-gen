#![feature(lazy_cell)]
#![feature(new_uninit)]
#![feature(downcast_unchecked)]

use std::{
    collections::HashSet,
    fs::{self, File},
    path::{Path, PathBuf},
    process::{exit, Command},
    sync::Arc,
    thread::JoinHandle,
};

use ansi_term::Colour::Red;
use args::{Args, LanguageOption};
use clap::Parser;
use config::{
    CFG, LANG_OUTPUT_DIR, OUTPUT_ENUM_CODE_DIR, OUTPUT_SCRIPT_CODE_DIR,
    OUTPUT_SERVER_ENUM_CODE_DIR, OUTPUT_SERVER_SCRIPT_CODE_DIR, REF_TEXT_DIR, SOURCE_XLSXS_DIR,
};
use table::Table;
use xlsx_read::{excel_file::ExcelFile, excel_table::ExcelTable};

mod args;
mod config;
mod error;
mod lex;
mod parser;
mod preconfig;
mod table;
mod types;
mod util;

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

#[allow(unused_must_use)]
fn process_lstring_xlsx<P: AsRef<Path> + Send + 'static>(
    path: P,
    sx: std::sync::mpsc::Sender<JoinHandle<()>>,
    langdir: String,
) {
    use std::io::Write;
    let handle = std::thread::spawn(move || {
        let file = ExcelFile::load_from_path(path);
        let mut tables = Vec::<ExcelTable>::default();

        if let Ok(mut ff) = file {
            match ff.parse_workbook() {
                Ok(ret) => {
                    let output_path = format!(
                        "{}/{}.{}",
                        unsafe { OUTPUT_SCRIPT_CODE_DIR },
                        "LanguageKey",
                        CFG.language_file_suffix
                    );

                    let output_cn_path = format!(
                        "{}/{}/{}.{}",
                        langdir, "Language_CN", CFG.language_file_name, CFG.language_file_suffix,
                    );
                    let output_cnh_path = format!(
                        "{}/{}/{}.{}",
                        langdir, "Language_CNH", CFG.language_file_name, CFG.language_file_suffix,
                    );
                    let output_en_path = format!(
                        "{}/{}/{}.{}",
                        langdir, "Language_EN", CFG.language_file_name, CFG.language_file_suffix,
                    );
                    let output_jp_path = format!(
                        "{}/{}/{}.{}",
                        langdir, "Language_JP", CFG.language_file_name, CFG.language_file_suffix,
                    );

                    if let (
                        Ok(mut file),
                        Ok(mut cn_file),
                        Ok(mut cnh_file),
                        Ok(mut en_file),
                        Ok(mut jp_file),
                    ) = (
                        File::create(output_path),
                        File::create(output_cn_path),
                        File::create(output_cnh_path),
                        File::create(output_en_path),
                        File::create(output_jp_path),
                    ) {
                        writeln!(file, "{}", CFG.file_banner);
                        writeln!(file, "using System.Collections.Generic;");
                        writeln!(file, "// ReSharper disable InconsistentNaming");
                        writeln!(file, "// ReSharper disable IdentifierTypo");
                        writeln!(file, "// ReSharper disable StringLiteralTypo");
                        writeln!(file, "public class LanguageKey");
                        writeln!(file, "{{");
                        writeln!(file, "#region const keys");

                        for (_, id) in ret.into_iter() {
                            if let Ok(table) = ff.parse_sheet(id) {
                                tables.push(table);
                            }
                        }

                        let mut count = 0;
                        for table in tables.iter() {
                            let height = table.height();
                            for row in 2..height {
                                table.cell(0, row).map(|v| {
                                    writeln!(file, "    public const ushort {} = {};", v, count);
                                    count += 1;
                                });

                                /* lang file */

                                // CN
                                table.cell(1, row).map(|v| {
                                    writeln!(cn_file, "{}", v.as_str());
                                });
                                // CNH
                                table.cell(2, row).map(|v| {
                                    writeln!(cnh_file, "{}", v.as_str());
                                });
                                // EN
                                table.cell(3, row).map(|v| {
                                    writeln!(en_file, "{}", v.as_str());
                                });
                                // JP
                                table.cell(4, row).map(|v| {
                                    writeln!(jp_file, "{}", v.as_str());
                                });

                                /* lang file */
                            }
                        }

                        writeln!(file, "    public const ushort Invalid = ushort.MaxValue;");
                        writeln!(file, "#endregion");
                        writeln!(file, "");
                        writeln!(
                            file,
                            "    public static ushort LanguageKeyToId(string languageKey)"
                        );
                        writeln!(file, "    {{");
                        writeln!(
                            file,
                            "        if (_filedIdMap.TryGetValue(languageKey, out ushort id))"
                        );
                        writeln!(file, "            return id;");
                        writeln!(file, "        return Invalid;");
                        writeln!(file, "    }}");
                        writeln!(file, "");

                        writeln!(file, "    private static readonly Dictionary<string,ushort> _filedIdMap = new Dictionary<string,ushort>()");
                        writeln!(file, "    {{");
                        for table in tables.iter() {
                            let height = table.height();
                            for row in 2..height {
                                table.cell(0, row).map(|v| {
                                    writeln!(file, "        {{\"{}\", {}}},", v, v);
                                });
                            }
                        }
                        writeln!(file, "    }};");
                        writeln!(file, "}}");
                    }
                }
                _ => {}
            }
        }
    });

    sx.send(handle).unwrap();
}

fn process_config_collection<P: AsRef<Path> + Send + 'static>(
    path: P,
    sx: std::sync::mpsc::Sender<JoinHandle<()>>,
    src: &'static str,
    excluded: Arc<ExcludedFolders<'static>>,
) {
    use std::io::Write;
    let handle = std::thread::spawn(move || {
        if let Ok(mut file) = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
        {
            let mut dirs = Vec::new();
            let mut names = Vec::new();
            let dir = fs::read_dir(src);

            if dir.is_err() {
                eprintln!(
                    "{}",
                    Red.bold().paint(format!(
                        "[Error]: An error occurred while processing ConfigCollection: {}",
                        dir.unwrap_err()
                    ))
                );
            } else {
                let dir = dir.unwrap();
                dirs.push(dir);

                while !dirs.is_empty() {
                    let dir = dirs.pop().unwrap();

                    for entry in dir {
                        let entry = entry.unwrap();
                        let path = entry.path();
                        let base_name = path.file_name().unwrap().to_str().unwrap();
                        let idx = base_name.find('.').unwrap_or_default();

                        if config::TABLE_XLSX_FILTER.contains(base_name) {
                            continue;
                        }

                        if path.is_dir()
                            && !path.file_name().is_some_and(|v| {
                                v.to_str().is_some_and(|vv| {
                                    vv.starts_with('.') || excluded.0.contains(vv)
                                })
                            })
                        {
                            let d = fs::read_dir(path).unwrap();
                            dirs.push(d);
                        } else if path
                            .extension()
                            .is_some_and(|x| x.to_str().unwrap() == CFG.source_table_suffix)
                            && !path
                                .file_name()
                                .is_some_and(|v| v.to_str().is_some_and(|vv| vv.starts_with('~')))
                        {
                            names.push((&base_name[..idx]).to_string());
                        }
                    }
                }
            }

            let _ = std::io::Write::write(&mut file, CFG.file_banner.as_bytes());
            let _ = std::io::Write::write(
                &mut file,
                r##"
using Config.Common;
using System.Collections.Generic;

namespace Config
{
    /// <summary>
    /// 所有配置数据类的集合
    /// </summary>
    public static class ConfigCollection
    {
        /// <summary>
        /// 所有配置数据类的集合
        /// </summary>
        public static readonly IConfigData[] Items = new IConfigData[]
        {"##
                .as_bytes(),
            );

            // TODO: 临时代码
            let _ = file.write_fmt(format_args!("\n\t\t\tLocalSurnames.Instance,"));
            let _ = file.write_fmt(format_args!("\n\t\t\tLocalNames.Instance,"));
            let _ = file.write_fmt(format_args!("\n\t\t\tLocalZangNames.Instance,"));
            let _ = file.write_fmt(format_args!("\n\t\t\tLocalTownNames.Instance,"));
            let _ = file.write_fmt(format_args!("\n\t\t\tLocalMonasticTitles.Instance,"));

            for v in names.iter() {
                let _ = file.write_fmt(format_args!("\n\t\t\t{}.Instance,", v));
            }
            let _ = file.write("\n\t\t".as_bytes());
            let _ = file.write(r##"};

        /// <summary>
        /// 配置数据名称表
        /// </summary>
        public static readonly Dictionary<string, IConfigData> NameMap = new Dictionary<string, IConfigData>()
        {"##.as_bytes());

            // TODO: 临时代码
            let _ = file.write_fmt(format_args!(
                "\n\t\t\t{{\"{}\", {}.Instance}},",
                "LocalSurnames", "LocalSurnames"
            ));
            let _ = file.write_fmt(format_args!(
                "\n\t\t\t{{\"{}\", {}.Instance}},",
                "LocalNames", "LocalNames"
            ));
            let _ = file.write_fmt(format_args!(
                "\n\t\t\t{{\"{}\", {}.Instance}},",
                "LocalZangNames", "LocalZangNames"
            ));
            let _ = file.write_fmt(format_args!(
                "\n\t\t\t{{\"{}\", {}.Instance}},",
                "LocalTownNames", "LocalTownNames"
            ));
            let _ = file.write_fmt(format_args!(
                "\n\t\t\t{{\"{}\", {}.Instance}},",
                "LocalMonasticTitles", "LocalMonasticTitles"
            ));

            for v in names.iter() {
                let _ = file.write_fmt(format_args!("\n\t\t\t{{\"{}\", {}.Instance}},", v, v));
            }

            let _ = file.write("\n\t\t".as_bytes());
            let _ = file.write(
                r##"};
    }
}"##
                .as_bytes(),
            );

            let _ = file.flush();
        } else {
            eprintln!(
                "{}",
                Red.bold().paint(format!(
                    "[Error]: Please provide the save path of ConfigCollection"
                ))
            )
        }
    });

    sx.send(handle).unwrap();
}

fn load_tables<P: AsRef<Path>>(
    dir: P,
    tx: std::sync::mpsc::Sender<JoinHandle<()>>,
    excluded: Arc<ExcludedFolders<'static>>,
) -> Result<Vec<Table>, error::Error> {
    let mut tables = Vec::new();
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
        let file_ext = path
            .extension()
            .ok_or::<error::Error>(
                format!("can't find extension: `{:?}`", path.as_os_str()).into(),
            )?
            .to_str()
            .ok_or::<error::Error>("invalid unicode".into())?;

        // ban-lists
        if config::TABLE_XLSX_FILTER.contains(file_name) {
            continue;
        }

        if path.is_dir() && !file_name.starts_with('.') && !excluded.0.contains(file_name) {
            let tx_clone = tx.clone();
            let excluded_clone = excluded.clone();
            let handle = std::thread::spawn(move || {
                let _ = load_tables(path, tx_clone, excluded_clone);
            });
            tx.send(handle).unwrap();
        } else if file_ext == CFG.source_table_suffix && !file_name.starts_with('~') {
            let idx = file_name
                .find('.')
                .ok_or::<error::Error>("can't find `.` in xlsx file name".into())?;
            let file_name = &file_name[..idx];
            tables.push(Table::load(&path, file_name.into())?);
        }
    }
    Ok(tables)
}

fn build(tables: &[Table]) -> Result<(), error::Error> {
    Ok(())
}

#[derive(Default)]
struct ExcludedFolders<'a>(HashSet<&'a str>);

fn main() {
    let args = Args::try_parse().unwrap();
    create_dest_dirs(&args);
    unsafe {
        OUTPUT_SCRIPT_CODE_DIR = Box::leak(args.output_script_dir.into_boxed_str());
        OUTPUT_ENUM_CODE_DIR = Box::leak(args.output_enum_dir.into_boxed_str());
        OUTPUT_SERVER_SCRIPT_CODE_DIR = Box::leak(args.output_server_script_dir.into_boxed_str());
        OUTPUT_SERVER_ENUM_CODE_DIR = Box::leak(args.output_server_enum_dir.into_boxed_str());
        SOURCE_XLSXS_DIR = Box::leak(args.src_table_dir.into_boxed_str());
        REF_TEXT_DIR = Box::leak(args.ref_mapping_dir.into_boxed_str());
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

            let (tx, rx) = std::sync::mpsc::channel::<JoinHandle<()>>();
            let mut path = PathBuf::from(unsafe { SOURCE_XLSXS_DIR });
            path.push(CFG.language_xlsx_name);
            process_lstring_xlsx(path, tx.clone(), args.output_lang_dir);

            // process config collection
            process_config_collection(
                PathBuf::from(args.config_collection_path),
                tx.clone(),
                unsafe { SOURCE_XLSXS_DIR },
                excluded.clone(),
            );

            // load regular tables
            match load_tables(unsafe { SOURCE_XLSXS_DIR }, tx.clone(), excluded) {
                Ok(tables) => {}
                Err(e) => {
                    eprintln!("{}", Red.bold().paint(format!("{}", e)));
                    exit(-1);
                }
            }

            // !! drop the raw tx
            drop(tx);
            while let Ok(handle) = rx.recv() {
                let _ = handle.join();
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
    let p = parser::parse_assign(
        r#" Tuple<int[], int[2], int> = {{1,2,3,4,5}, {-1,-2}, 100} "#,
        0,
        0,
    )
    .unwrap();
    if p.check() {
        let _ = p.code_fmt(&mut display);
    } else {
        println!("check failed");
    }
    println!("{}", display);
}

#[test]
fn generate() {
    let path = "D:\\taiwu\\config\\GlobalConfig.xlsx";
    let table = Table::load(path, "GlobalConfig").unwrap();
    let mut dest = std::fs::File::options().write(true).truncate(true).open("output.cs").unwrap();
    match table.build(&mut dest, false) {
        Ok(_) => {}
        Err(e) => println!("{}", e),
    }
}
