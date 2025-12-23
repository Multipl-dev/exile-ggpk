pub fn fetch_latest_schema() -> Result<String, Box<dyn std::error::Error>> {
    let url = "https://github.com/poe-tool-dev/dat-schema/releases/download/latest/schema.min.json";
    let text = reqwest::blocking::get(url)?.text()?;
    Ok(text)
}
