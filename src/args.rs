use clap::{Parser, Subcommand};

pub enum LanguageOption {
    CN,
    CNH,
    EN,
    JP,
    Invalid,
}

impl From<&str> for LanguageOption {
    fn from(value: &str) -> Self {
        match value {
            "CN" | "cn" | "Cn" | "cN" => Self::CN,
            "CNH" | "cnh" => Self::CNH,
            "EN" | "en" => Self::EN,
            "JP" | "jp" => Self::JP,
            _ => Self::Invalid,
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// need to update svn or not
    #[arg(default_value_t = false, short, long)]
    pub update_git: bool,
    /// path of the config dir
    #[arg(default_value_t = String::from("D:/config/"), long)]
    pub src_table_dir: String,
    /// path of the output script dir
    #[arg(default_value_t = String::from("ExportScripts/"), long)]
    pub output_script_dir: String,
    /// path of the output server script dir
    #[arg(default_value_t = String::from("ServerExportScripts/"), long)]
    pub output_server_script_dir: String,
    /// path of the output server enum dir
    #[arg(default_value_t = String::from("ServerConfigExportEnum/"), long)]
    pub output_server_enum_dir: String,
    /// path of the output enum dir
    #[arg(default_value_t = String::from("ConfigExportEnum/"), long)]
    pub output_enum_dir: String,
    /// path of the config ref mapping dir
    #[arg(default_value_t = String::from("ConfigRefNameMapping/"), long)]
    pub ref_mapping_dir: String,
    /// path of the ConfigCollection.cs
    #[arg(default_value_t = String::from(""), long)]
    pub config_collection_path: String,
    /// language_option
    #[arg(default_value_t = String::from("CN"), long)]
    pub loption: String,
    /// language file output dir
    #[arg(default_value_t = String::from("Data/"), long)]
    pub output_lang_dir: String,
    /// force process all tables or not
    #[arg(default_value_t = false, short, long)]
    pub force_all: bool,
    /// only export LString.xlsx
    #[arg(default_value_t = false, short, long)]
    pub lstring: bool,
    /// excluded folders
    #[arg(default_value_t = String::default(), long)]
    pub excluded_folders: String,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(name = "build", visible_alias = "b")]
    Build,

    #[command(name = "clean", visible_alias = "c")]
    Clean,
}
