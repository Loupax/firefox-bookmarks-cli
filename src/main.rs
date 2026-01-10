use directories::BaseDirs;
use ini::Ini;
use rusqlite::Connection;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct Bookmark {
    title: Option<String>,
    url: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Get the source path from Command Line Arguments
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&"--list-profiles".to_string()) {
        list_profiles()?;
        return Ok(());
    }

    let mut target_profile_name: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--profile" && i + 1 < args.len() {
            target_profile_name = Some(args[i + 1].clone());
            i += 1;
        }
        i += 1;
    }

    let source_path = find_firefox_database(target_profile_name.as_deref())?;

    // 2. Determine the Temporary Directory path
    // This finds /tmp on Linux or %TEMP% on Windows automatically
    let mut temp_path = env::temp_dir();
    temp_path.push("firefox_places_copy.sqlite");

    // 3. Perform the Copy
    // This will overwrite the file if it already exists in temp
    fs::copy(source_path, &temp_path)?;

    // 4. Connect to the *Copy*
    let conn = Connection::open(&temp_path)?;

    let mut stmt = conn.prepare(
        "SELECT b.title, p.url 
         FROM moz_bookmarks b 
         JOIN moz_places p ON b.fk = p.id 
         WHERE b.type = 1"
    )?;

    let bookmark_iter = stmt.query_map([], |row| {
        Ok(Bookmark {
            title: row.get(0)?,
            url: row.get(1)?,
        })
    })?;

    for bookmark in bookmark_iter {
        let b = bookmark?;
        let title = b.title.unwrap_or_else(|| "No Title".to_string());

        println!("{}\t{}", title, b.url);
    }

    Ok(())
}
fn find_firefox_database(profile_name: Option<&str>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // 1. Find the Firefox root directory (e.g. ~/.mozilla/firefox)
    let base_dirs = BaseDirs::new().ok_or("Could not find home directory")?;
    
    let firefox_root = if cfg!(target_os = "windows") {
        base_dirs.data_dir().join("Mozilla").join("Firefox")
    } else if cfg!(target_os = "macos") {
        base_dirs.data_dir().join("Firefox")
    } else {
        base_dirs.home_dir().join(".mozilla").join("firefox")
    };

    // 2. Read profiles.ini
    let ini_path = firefox_root.join("profiles.ini");
    if !ini_path.exists() {
        return Err(format!("Could not find profiles.ini at {:?}", ini_path).into());
    }

    let conf = Ini::load_from_file(&ini_path)?;

    // 3. Search for the correct profile section
    let mut relative_path: Option<String> = None;
    let mut is_relative = true;

    for (sec, prop) in conf.iter() {
        // We only care about sections named [Profile0], [Profile1], etc.
        if let Some(section_name) = sec {
            if !section_name.starts_with("Profile") {
                continue;
            }

            // Check if this is the profile we want
            let name = prop.get("Name");
            let path = prop.get("Path");
            let is_default = prop.get("Default").unwrap_or("0") == "1";

            if let Some(p) = path {
                if let Some(target) = profile_name {
                    // User asked for a specific profile name
                    if name == Some(target) {
                        relative_path = Some(p.to_string());
                        is_relative = prop.get("IsRelative").unwrap_or("1") == "1";
                        break;
                    }
                } else {
                    // User didn't ask, look for the default marked in INI
                    if is_default {
                        relative_path = Some(p.to_string());
                        is_relative = prop.get("IsRelative").unwrap_or("1") == "1";
                        // We don't break here immediately because sometimes there are multiple defaults 
                        // (install-specific), but usually the last one wins or the one marked Default=1.
                        // For simplicity, we take the first one marked Default=1.
                        break;
                    }
                }
            }
        }
    }

    let rel_path = relative_path.ok_or(
        if let Some(n) = profile_name {
            format!("Profile '{}' not found in profiles.ini", n)
        } else {
            "No default profile found in profiles.ini".to_string()
        }
    )?;

    // 4. Construct the full path
    // If IsRelative=1, the path is relative to the firefox_root.
    // If IsRelative=0, the path is absolute.
    let full_profile_dir = if is_relative {
        firefox_root.join(rel_path)
    } else {
        PathBuf::from(rel_path)
    };

    Ok(full_profile_dir.join("places.sqlite"))
}

// --- NEW FUNCTION ---
fn list_profiles() -> Result<(), Box<dyn std::error::Error>> {
    let base_dirs = BaseDirs::new().ok_or("Could not find home directory")?;
    
    let firefox_root = if cfg!(target_os = "windows") {
        base_dirs.data_dir().join("Mozilla").join("Firefox")
    } else if cfg!(target_os = "macos") {
        base_dirs.data_dir().join("Firefox")
    } else {
        base_dirs.home_dir().join(".mozilla").join("firefox")
    };

    let ini_path = firefox_root.join("profiles.ini");
    if !ini_path.exists() {
        return Ok(()); // No profiles found, just exit silently
    }

    let conf = Ini::load_from_file(&ini_path)?;

    for (sec, prop) in conf.iter() {
        if let Some(section_name) = sec {
            if section_name.starts_with("Profile") {
                if let Some(name) = prop.get("Name") {
                    // Print plain names, one per line
                    println!("{}", name);
                }
            }
        }
    }
    Ok(())
}
