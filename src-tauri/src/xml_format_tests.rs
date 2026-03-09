use super::*;
use std::io::{Error, Result as IoResult, Write};

#[derive(Debug)]
struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> IoResult<usize> {
        Err(Error::other("writer failed"))
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

#[test]
fn normalize_adds_header_and_spacing() {
    let input =
        r#"<Data><Property name="Data" value=""/><Property name="Dependencies" value=""/></Data>"#;
    let out = normalize_pulsar_xml(input);
    assert!(out.starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n"));
    assert!(out.contains(r#"name="Data" />"#));
    assert!(out.contains(r#"name="Dependencies" />"#));
}

#[test]
fn pretty_print_xml_indents_output() {
    let input = r#"<Data><Property name="Data"></Property></Data>"#;
    let out = pretty_print_xml(input).expect("pretty print should succeed");
    assert!(out.contains('\n'));
    assert!(out.contains("  <Property"));
}

#[test]
fn format_pulsar_xml_combines_steps() {
    let input = r#"<Data><Property name="Data" value=""/></Data>"#;
    let out = format_pulsar_xml(input).expect("format should succeed");
    assert!(out.starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n"));
    assert!(out.contains(r#"name="Data" />"#));
}

#[test]
fn pretty_print_invalid_xml_errors() {
    let input = "<Data><Property></Data>";
    let err = pretty_print_xml(input).expect_err("invalid XML should error");
    assert!(err.contains("XML formatting error"));
}

#[test]
fn write_pretty_xml_into_propagates_writer_errors() {
    let input = r#"<Data><Property name="Data"></Property></Data>"#;
    let err =
        write_pretty_xml_into(input, FailingWriter).expect_err("writer failure should be mapped");
    assert!(err.contains("XML formatting error"));
}

#[test]
fn utf8_bytes_to_string_rejects_invalid_utf8() {
    let err = utf8_bytes_to_string(vec![0xff]).expect_err("invalid utf8 should error");
    assert!(err.contains("Failed to convert formatted XML to string"));
}

#[test]
fn failing_writer_flush_is_ok() {
    let mut writer = FailingWriter;
    writer.flush().expect("flush should succeed");
}
