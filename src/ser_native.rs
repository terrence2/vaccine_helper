use anyhow::Result;
use std::fs;

pub fn download_file(data: &str, filename: &str, _mime_type: &str) -> Result<()> {
    let filename = rfd::FileDialog::default()
        .set_title("Save Records")
        .set_file_name(filename)
        .add_filter("RON Files", &["ron", "RON"])
        .pick_file();
    if let Some(name) = filename {
        fs::write(name, data)?;
    }
    Ok(())
}

pub fn create_file_picker<F>(callback: F) -> Result<()>
where
    F: Fn(String) + 'static,
{
    let filename = rfd::FileDialog::default()
        .set_title("Load Records")
        .add_filter("RON Files", &["ron", "RON"])
        .pick_file();
    if let Some(name) = filename {
        let data = fs::read_to_string(name)?;
        callback(data);
    }
    Ok(())
}
