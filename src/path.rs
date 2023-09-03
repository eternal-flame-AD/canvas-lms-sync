use std::{
    fs::File,
    io::{self, Write},
};

pub fn sanitize_file_name(file_name: &str) -> String {
    file_name
        .replace("/", "_")
        .replace("\\", "_")
        .replace(":", "_")
        .replace("*", "_")
        .replace("?", "_")
        .replace("\"", "_")
        .replace("<", "_")
        .replace(">", "_")
        .replace("|", "_")
}

#[cfg(target_os = "windows")]
pub(crate) fn write_url_file(
    url: &str,
    _name: &str,
    file_name_base: &str,
) -> Result<(), io::Error> {
    eprintln!("Writing url file: {}", file_name_base);
    let mut file = File::create(format!("{}.url", file_name_base))?;
    writeln!(file, "[InternetShortcut]")?;
    writeln!(file, "URL={}", url)?;

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn write_url_file(url: &str, name: &str, file_name_base: &str) -> Result<(), io::Error> {
    let mut file = File::create(format!("{}.desktop", file_name_base))?;
    writeln!(file, "[Desktop Entry]")?;
    writeln!(file, "Encoding=UTF-8")?;
    writeln!(file, "Name={}", file_name_base)?;
    writeln!(file, "Type=Link")?;
    writeln!(file, "URL={}", url)?;
    writeln!(file, "Icon=text-html")?;

    Ok(())
}
