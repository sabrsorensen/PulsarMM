use quick_xml::events::Event;
use quick_xml::{Reader, Writer};
use std::io::Write;

pub fn normalize_pulsar_xml(xml_body: &str) -> String {
    format!("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n{}", xml_body)
        .replace(" name=\"Data\" value=\"\"", " name=\"Data\"")
        .replace(
            " name=\"Dependencies\" value=\"\"",
            " name=\"Dependencies\"",
        )
        .replace("\"/>", "\" />")
}

fn write_pretty_xml_into<W: Write>(unformatted_xml: &str, output: W) -> Result<W, String> {
    let mut reader = Reader::from_str(unformatted_xml);
    reader.config_mut().trim_text(true);
    let mut writer = Writer::new_with_indent(output, b' ', 2);

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(event) => writer
                .write_event(event)
                .map_err(|e| format!("XML formatting error: {:?}", e))?,
            Err(e) => return Err(format!("XML formatting error: {:?}", e)),
        }
    }

    Ok(writer.into_inner())
}

fn utf8_bytes_to_string(bytes: Vec<u8>) -> Result<String, String> {
    String::from_utf8(bytes)
        .map_err(|e| format!("Failed to convert formatted XML to string: {}", e))
}

pub fn pretty_print_xml(unformatted_xml: &str) -> Result<String, String> {
    let bytes = write_pretty_xml_into(unformatted_xml, Vec::new())?;
    utf8_bytes_to_string(bytes)
}

pub fn format_pulsar_xml(unformatted_xml: &str) -> Result<String, String> {
    let xml_body = pretty_print_xml(unformatted_xml)?;
    Ok(normalize_pulsar_xml(&xml_body))
}

#[cfg(test)]
#[path = "xml_format_tests.rs"]
mod tests;
