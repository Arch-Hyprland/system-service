use std::{
    env, fs::DirEntry, path::PathBuf
};

use freedesktop_entry_parser::parse_entry;
use freedesktop_icon_lookup::Cache;
use log::warn;
use serde::{Deserialize, Serialize};
use xdgkit::icon_finder;

#[derive(Deserialize, Serialize)]
pub struct Entry {
    pub cls: String,
    pub name: String,
    pub icon: String,
    pub cmd: String,
}

impl Entry {
    pub fn new(cls: String, name: String, icon: String, cmd: String) -> Self {
        Self {
            cls,
            name,
            icon,
            cmd,
        }
    }
}

pub fn transform_entry() -> Vec<Entry> {
    let mut entries = Vec::new();

    // 缓存主题
    let theme = env::var_os("GTK_THEME").unwrap(); // "WhiteSur";
    let mut cache = Cache::new().unwrap();
    cache.load(theme.to_string_lossy()).unwrap();
    for desktop in collect_desktop_files() {
        let app = parse_entry(desktop.path());
        match app {
            Ok(app) => {
                let display = app.section("Desktop Entry").attr("NoDisplay");
                match display {
                    Some("true") => continue,
                    _ => (),
                }
                let name = app.section("Desktop Entry").attr("Name");
                let icon = app.section("Desktop Entry").attr("Icon");
                let cmd = app.section("Desktop Entry").attr("Exec");
                let cls = app.section("Desktop Entry").attr("StartupWMClass");
                let terminal = app.section("Desktop Entry").attr("Terminal");
                if let (Some(name), Some(icon), Some(cmd)) = (name, icon, cmd) {
                    let path = if icon.ends_with(".png") || icon.ends_with(".svg") {
                        PathBuf::from(icon)
                    } else if let Some(theme_icon) = cache.lookup(icon, theme.to_str()) {
                        theme_icon
                    } else if let Some(other_icon) = icon_finder::find_icon(icon.to_string(), 256, 1){
                       other_icon
                    } else {
                        env::var_os("HOME")
                            .map(|p| {
                                PathBuf::from(p)
                                    .join(".local/share/icons/custom")
                                    .join(format!("{}.svg", icon))
                            })
                            .unwrap()
                    };

                    let cmd = match terminal == Some("true") {
                        true => String::from("kitty ") + cmd,
                        false => String::from(cmd),
                    };
                    
                    let entry = Entry::new(
                        String::from(cls.unwrap_or("")),
                        String::from(name),
                        path.as_path().to_str().unwrap().to_string(),
                        cmd,
                    );
                    entries.push(entry);
                };
            }
            Err(e) => {
                warn!("Failed to read file {} : {e}", desktop.path().display());
            }
        }
    }
    entries
}

//
// 查找所有的应用安装目录
//
fn find_application_dir() -> Vec<PathBuf> {
    let mut dirs = env::var_os("XDG_DATA_DIRS")
        .map(|val| env::split_paths(&val).map(PathBuf::from).collect())
        .unwrap_or_else(|| {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
            ]
        });
    if let Some(data_home) = env::var_os("XDG_DATA_HOME").map(PathBuf::from).map_or_else(
        || {
            env::var_os("HOME")
                .map(|p| PathBuf::from(p).join(".local/share"))
                .or_else(|| {
                    warn!("No XDG_DATA_HOME and HOME environment variable found");
                    None
                })
        },
        Some,
    ) {
        dirs.push(data_home)
    }
    let mut res = Vec::new();
    for dir in dirs {
        res.push(dir.join("applications"));
    }
    // println!("查找入口文件的目录: {:?}", res);
    res
}

//
// 汇总当前安装 Application 的 entry.desktop 文件
//
fn collect_desktop_files() -> Vec<DirEntry> {
    let mut res = Vec::new();

    for dir in find_application_dir() {
        // 判断文件夹是否存在
        if !dir.exists() {
            continue;
        }
        // 读取当前文件夹中的所有文件
        match dir.read_dir() {
            Ok(dir) => {
                // 展开所有文件
                for entry in dir.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |e| e == "desktop") {
                        res.push(entry);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read dir : {dir:?}: {e}");
                continue;
            }
        }
    }
    // println!("找到如下的入口文件：{:?}", res);
    res
}
