use crate::pp::WGSLError;

pub fn parse_u32(value: &str, line: usize) -> Result<u32, WGSLError> {
    let value = value.trim().trim_end_matches('u');
    if value.starts_with("0x") {
        <u32>::from_str_radix(value.strip_prefix("0x").unwrap(), 16)
    } else {
        value.parse::<u32>()
    }
    .or(Err(WGSLError::new(
        format!("Cannot parse '{value}' as u32"),
        line,
    )))
}

pub fn fetch_include(name: String) -> Option<String> {
    let filename = format!("./include/{name}.wgsl");
    std::fs::read_to_string(filename).ok()
}
