use std::error::Error;
use std::path::{Path, PathBuf};

use clap::{App, Arg};
use sqlx::postgres::{PgConnectOptions, PgSslMode};

use crate::domain;
use crate::domain::LoadOptions;

pub struct Config {
    pub database: DatabaseConfig,
    pub source: Source,
    pub load_options: domain::LoadOptions,
}

pub enum Source {
    File(PathBuf),
    Directory(PathBuf),
}

pub struct DatabaseConfig {
    port: u16,
    host: String,
    username: String,
    password: String,
    database_name: String,
    tls: bool,
    table_name: String,
    init: bool,
}

impl DatabaseConfig {
    pub fn get_table_name(&self) -> String {
        self.table_name.clone()
    }

    pub fn is_init(&self) -> bool {
        self.init
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            port: 0,
            host: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            database_name: "".to_string(),
            tls: false,
            table_name: "".to_string(),
            init: false,
        }
    }
}

impl From<&DatabaseConfig> for PgConnectOptions {
    fn from(c: &DatabaseConfig) -> Self {
        let ssl_mode = if c.tls {
            PgSslMode::Require
        } else {
            // Try an encrypted connection, fallback to unencrypted if it fails
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&c.host)
            .username(&c.username)
            .password(&c.password)
            .port(c.port)
            .database(&c.database_name)
            .ssl_mode(ssl_mode)
    }
}

impl From<clap::ArgMatches<'_>> for DatabaseConfig {
    fn from(matches: clap::ArgMatches) -> Self {
        let port = matches
            .value_of("db_port")
            .unwrap_or("5432")
            .parse::<u16>()
            .unwrap_or(5432);
        let host = matches
            .value_of("db_host")
            .unwrap_or("localhost")
            .to_string();
        let username = matches
            .value_of("db_username")
            .unwrap_or("postgres")
            .to_string();
        let password = matches
            .value_of("db_password")
            .unwrap_or("postgres")
            .to_string();
        let database_name = matches
            .value_of("db_name")
            .unwrap_or("postgres")
            .to_string();
        let tls = matches
            .value_of("tls")
            .unwrap_or("false")
            .parse::<bool>()
            .unwrap_or(false);
        let table_name = matches
            .value_of("table_name")
            .unwrap_or("transactions")
            .to_string();
        let init = matches.is_present("init_db");

        Self {
            port,
            host,
            username,
            password,
            database_name,
            tls,
            table_name,
            init,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(String),
    DirectoryNotFound(String),
    DirectoryEmpty(String),
    RequiredConfigurationMissing(String),
}

impl ConfigError {
    fn file_not_found(s: &str) -> Self {
        ConfigError::FileNotFound(s.to_string())
    }

    fn directory_not_found(s: &str) -> Self {
        ConfigError::DirectoryNotFound(s.to_string())
    }

    fn directory_empty(s: &str) -> Self {
        ConfigError::DirectoryEmpty(s.to_string())
    }

    fn required_configuration_missing(s: &str) -> Self {
        ConfigError::RequiredConfigurationMissing(s.to_string())
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Self::FileNotFound(s) => write!(f, "File not found: {}", s),
            Self::DirectoryNotFound(s) => write!(f, "Directory not found: {}", s),
            Self::DirectoryEmpty(s) => write!(f, "Directory {} does not contain any CSV files.", s),
            Self::RequiredConfigurationMissing(s) => {
                write!(f, "Required configuration argument missing: {}", s)
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn description(&self) -> &str {
        "Could not load configuration."
    }
}

pub fn parse_args() -> Result<Config, Box<dyn Error>> {
    let matches = App::new("CSV Importer")
        .version("1.0")
        .author("Trey Hutcheson")
        .about("Imports formatted CSV files into a financial database")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .takes_value(true)
                .required_unless("directory"),
        )
        .arg(
            Arg::with_name("directory")
                .short("d")
                .long("directory")
                .value_name("DIR")
                .takes_value(true)
                .conflicts_with("file")
                .required_unless("file"),
        )
        .arg(
            Arg::with_name("db_port")
                .long("port")
                .value_name("db_port")
                .takes_value(true)
                .default_value("5432")
                .env("DB_PORT"),
        )
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .value_name("db_host")
                .takes_value(true)
                .default_value("localhost")
                .env("DB_HOST"),
        )
        .arg(
            Arg::with_name("username")
                .short("u")
                .long("uid")
                .value_name("db_username")
                .takes_value(true)
                .default_value("postgres")
                .env("DB_UID"),
        )
        .arg(
            Arg::with_name("password")
                .short("pwd")
                .long("password")
                .value_name("db_password")
                .default_value("postgress")
                .takes_value(true)
                .env("DB_PASSWORD"),
        )
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .value_name("db_name")
                .default_value("postgress")
                .takes_value(true)
                .env("DB_NAME"),
        )
        .arg(
            Arg::with_name("tls")
                .short("tls")
                .value_name("db_tls")
                .default_value("false")
                .takes_value(true)
                .env("DB_TLS"),
        )
        .arg(
            Arg::with_name("table")
                .long("db_table")
                .value_name("db_table")
                .default_value("transactions")
                .takes_value(true)
                .env("DB_TABLE"),
        )
        .arg(Arg::with_name("init_db").long("init").takes_value(false))
        .arg(Arg::with_name("load_all").long("all").takes_value(false))
        .arg(
            Arg::with_name("load_new")
                .long("new")
                .takes_value(false)
                .conflicts_with("load_all"),
        )
        .get_matches();

    let source = if let Some(f) = matches.value_of("file") {
        let p = Path::new(f);
        if p.exists() {
            Source::File(p.to_path_buf())
        } else {
            return Err(Box::new(ConfigError::file_not_found(f)));
        }
    } else if let Some(d) = matches.value_of("directory") {
        let p = Path::new(d);
        if p.exists() {
            if directory_contains_csvs(p) {
                Source::Directory(p.to_path_buf())
            } else {
                return Err(Box::new(ConfigError::directory_empty(d)));
            }
        } else {
            return Err(Box::new(ConfigError::directory_not_found(d)));
        }
    } else {
        return Err(Box::new(ConfigError::required_configuration_missing(
            "file or directory",
        )));
    };

    let load_options = if matches.is_present("load_new") {
        LoadOptions::New
    } else {
        LoadOptions::All
    };

    let database = DatabaseConfig::from(matches);

    let c = Config {
        database,
        source,
        load_options,
    };
    Ok(c)
}

fn directory_contains_csvs(p: &Path) -> bool {
    let ext = Some(std::ffi::OsStr::new("csv"));

    let read_result = std::fs::read_dir(p);
    return if let Ok(read_dir) = read_result {
        // let read_dir: std::fs::ReadDir = read_result.unwrap();
        for entry in read_dir {
            if let std::io::Result::Ok(dir_entry) = entry {
                let path: PathBuf = dir_entry.path();
                if ext == path.extension() {
                    return true;
                }
            }
        }

        false
    } else {
        false
    };
}
