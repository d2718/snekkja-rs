/*!
snekkja
A simple, static HTML/CSS/JS image gallery generator.
https://github.com/d2718/snekkja-rs
*/
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const VERSION: &str = env!("CARGO_PKG_VERSION");
static CONFIG: &str = "config.toml";
static USAGE: &str = "
snekkja
Generate a static HTML gallery in the current directory.

usage: snekkja [ OPTIONS ]

where OPTIONS is any of

    -c, --config    write a default config file and exit
    -h, --help      display this message and exit
";

static STATICS: &[(&str, &str)] = &[
    ("index.html", include_str!("../static/index.html")),
    ("snekkja.js", include_str!("../static/snekkja.js")),
    ("snekkja.css", include_str!("../static/snekkja.css")),
    ("next.svg", include_str!("../static/next.svg")),
    ("prev.svg", include_str!("../static/prev.svg")),
];

/// Data format for serializing/deserializing configuration data
/// to/from a file.
#[derive(Deserialize, Serialize)]
struct CfgFile {
    gallery_title: Option<String>,
    thumbnail_size: Option<u32>,
    file_extensions: Option<Vec<String>>,
    default_caption: Option<String>,
    pretty_json: Option<bool>,
}

impl std::default::Default for CfgFile {
    fn default() -> Self {
        CfgFile {
            gallery_title: Some("My Gallery".to_owned()),
            thumbnail_size: Some(100),
            file_extensions: Some(
                ["jpg", "jpeg", "png", "gif", "bmp", "webp"]
                .iter().map(|s| s.to_string()).collect()
            ),
            default_caption: Some("My Image".to_owned()),
            pretty_json: Some(false),
        }
    }
}

/// Container for user-configurable data; populated from data in a
/// [`CfgFile`] that's been deserizlized from a file.
struct Cfg {
    gallery_title: Option<String>,
    thumbnail_size: u32,
    file_extensions: Vec<String>,
    default_caption: Option<String>,
    pretty_json: bool,
}

impl std::default::Default for Cfg {
    fn default() -> Self {
        Cfg {
            gallery_title: None,
            thumbnail_size: 100,
            file_extensions: ["jpg", "jpeg", "png", "gif", "bmp", "webp"]
                .iter().map(|s| s.trim().to_string()).collect(),
            default_caption: None,
            pretty_json: false
        }
    }
}

impl Cfg {
    /// Populate configuration data from the path to a configuration file.
    fn from_file<P: AsRef<Path>>(p: P) -> Result<Cfg, String> {
        let mut c = Cfg::default();
        
        let p = p.as_ref();
        let file_bytes = std::fs::read(p).map_err(|e| format!(
            "Error reading configuration file {}: {}", p.display(), &e
        ))?;
        let cf: CfgFile = toml::from_slice(&file_bytes).map_err(|e| format!(
            "Error parsing configuration file {}: {}", p.display(), &e
        ))?;

        if let Some(s) = cf.gallery_title {
            c.gallery_title = Some(s);
        }
        if let Some(n) = cf.thumbnail_size {
            c.thumbnail_size = n;
        }
        if let Some(exts) = cf.file_extensions {
            c.file_extensions = exts.iter()
                .map(|s| s.trim().to_lowercase())
                .collect();
        }
        if let Some(s) = cf.default_caption {
            c.default_caption = Some(s);
        }
        if let Some(b) = cf.pretty_json {
            c.pretty_json = b;
        }

        Ok(c)
    }
}

/// Get a list of image files in the current directory.
/// 
/// An "image file" in this case specifically means a regula file with an
/// extension that case-insensitively matches one of the configured extensions.
fn get_file_list(exts: &[String]) -> Result<Vec<String>, String> {
    let mut filenames: Vec<String> = Vec::new();

    for ent_opt in std::fs::read_dir(".").map_err(|e| format!(
        "Unable to read current directory: {}", &e
    ))? {
        let ent = match ent_opt {
            Ok(ent) => ent,
            Err(_) => { continue; },
        };
        let p = ent.path();
        let ft = match ent.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                eprintln!("Error getting file type for {}: {}", p.display(), &e);
                continue;
            }
        };
        if !ft.is_file() { continue; }

        let pstr = match p.file_name()
            .map(|s| s.to_str())
            .flatten()
        {
            Some(s) => s,
            None => {
                eprintln!("{} is not UTF-8.", p.display());
                continue;
            }
        };
        let ext = match p.extension()
            .map(|s| s.to_str())
            .flatten()
            .map(|s| String::from(s).to_lowercase()) {
            Some(ext) => ext,
            None => { continue; },
        };

        for used_ext in exts.iter() {
            if used_ext == &ext {
                filenames.push(String::from(pstr));
                break;
            }
        }
    }

    Ok(filenames)
}

/// Helper function to transfer map entries from `source` to `dest`.
fn transfer_filenames<'a>(
    filenames: &[String],
    source: &mut HashMap<String, String>,
    dest: &mut HashMap<String, String>
) {
    for fname in filenames.iter() {
        if let Some(cap) = source.remove(fname) {
            dest.insert(fname.clone(), cap);
        }
    }
}

/// Collect captions from all possible sources.
fn read_captions<'a>(filenames: &[String]) -> HashMap<String, String> {
    let mut captions: HashMap<String, String> = HashMap::with_capacity(filenames.len());
    
    if let Ok(v) = std::fs::read("captions.toml") {
        match toml::from_slice::<HashMap<String, String>>(&v) {
            Ok(mut map) => {
                transfer_filenames(filenames, &mut map, &mut captions);
            },
            Err(e) => {
                eprintln!("Error deserializing \"captions.toml\": {}", &e);
            },
        }
    }

    if let Ok(v) = std::fs::read("captions.json") {
        match serde_json::from_slice::<HashMap<String, String>>(&v) {
            Ok(mut map) => {
                transfer_filenames(filenames, &mut map, &mut captions);
            },
            Err(e) => {
                eprintln!("Error deserializing \"captions.json\": {}", &e);
            },
        }
    }

    for fname in filenames.iter() {
        let mut cap_path = PathBuf::from(&fname);
        cap_path.set_extension("html");
        if let Ok(cap) = std::fs::read_to_string(&cap_path) {
            captions.insert(fname.clone(), cap);
        }
    }

    captions
}

fn show_usage() {
    println!("snekkja (Rust Implementation) v. {}", VERSION);
    println!("{}", USAGE);
}

fn write_config() -> Result<(), String> {
    let default_config = CfgFile::default();
    let cfg_bytes = toml::to_vec(&default_config).map_err(|e| format!(
        "Error serializing default configuration: {}", &e
    ))?;

    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("config.toml")
        .map_err(|e| format!(
            "Error opening default configuration file \"config.toml\" for writing: {}", &e
        ))?;

    f.write(&cfg_bytes).map_err(|e| format!(
        "Error writing to default configuration file \"config.toml\": {}", &e
    ))?;
    
    Ok(())
}

static ERROR: &str = "Error writing to \"data.js\": ";

fn write_data(
    filenames: &[String],
    captions: &HashMap<String, String>,
    cfg: &Cfg,
) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("data.js")
        .map_err(|e| format!(
            "Error opening \"data.js\" for writing: {}", &e
        ))?;
    
    write!(&mut f, "const FILES = ").map_err(|e| format!("{}{}", ERROR, &e))?;
    serde_json::to_writer(&mut f, filenames).map_err(|e| format!(
        "Error serializing filenames to \"data.js\": {}", &e
    ))?;
    write!(&mut f, ";\nconst CAPTIONS = new Map(Object.entries(").map_err(|e| format!("{}{}", ERROR, &e))?;
    serde_json::to_writer(&mut f, captions).map_err(|e| format!(
        "Error serializing captions to \"data.js\": {}", &e
    ))?;
    match &cfg.gallery_title {
        Some(title) => writeln!(&mut f, "));\nconst TITLE = {:?};", title)
            .map_err(|e| format!("{}{}", ERROR, &e))?,
        None => writeln!(&mut f, "));\nconst TITLE = null;")
            .map_err(|e| format!("{}{}", ERROR, &e))?,
    }
    match &cfg.default_caption {
        Some(cap) => writeln!(&mut f, "const DEFAULT_CAPTION = {:?}", cap)
            .map_err(|e| format!("{}{}", ERROR, &e))?,
        None => writeln!(&mut f, "const DEFAULT_CAPTION = null;")
            .map_err(|e| format!("{}{}", ERROR, &e))?,
    }
    writeln!(&mut f, "const THUMB_SIZE = {};", &cfg.thumbnail_size)
        .map_err(|e| format!("{}{}", ERROR, &e))?;

    Ok(())
}

fn write_statics() -> Result<(), String> {
    for (fname, contents) in STATICS.iter() {
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(fname)
            .map_err(|e| format!(
                "Error opening {:?} for writing: {}", fname, &e
            ))?;
        f.write(contents.as_bytes()).map_err(|e| format!(
            "Error writing contents of {:?}: {}", fname, &e
        ))?;
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(arg) = args.get(1) {
        match arg.as_str() {
            "-h" | "--help" => show_usage(),
            "-c" | "--config" => {
                write_config().map_err(|e| format!(
                    "Error writing default config file to \"config.toml\": {}", &e
                ))?;
                println!("Default configuration file written to \"config.toml\".");
            },
            x => {
                eprintln!("Unrecognized option: {}", &x);
            },
        }

        std::process::exit(0);
    }

    let cfg_path: &Path = CONFIG.as_ref();
    let cfg = if cfg_path.exists() {
        Cfg::from_file(cfg_path)?
    } else {
        println!("No config file {:?}, using defaults.", CONFIG);
        Cfg::default()
    };
    let files = get_file_list(&cfg.file_extensions)?;
    let captions = read_captions(&files);

    write_data(&files, &captions, &cfg)?;
    write_statics()?;

    Ok(())
}
