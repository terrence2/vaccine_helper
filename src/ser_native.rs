use anyhow::Result;

pub fn download_file(data: &str, filename: &str, mime_type: &str) -> Result<()> {
    anyhow::bail!("Would download file")
}

pub fn create_file_picker<F>(callback: F) -> Result<()>
where
    F: Fn(String) + 'static,
{
    anyhow::bail!("Would pick file")
}
