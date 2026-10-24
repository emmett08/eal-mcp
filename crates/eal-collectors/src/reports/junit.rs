//! Count explicit JUnit test-case outcomes without trusting suite summary attributes.
use quick_xml::{Reader, events::Event};
use serde_json::{Value, json};

pub(super) fn summarise(bytes: &[u8]) -> Result<Value, String> {
    let xml = std::str::from_utf8(bytes).map_err(|_| "JUnit requires UTF-8 XML")?;
    let mut reader = Reader::from_str(xml);
    reader.config_mut().enable_all_checks(true);
    reader.config_mut().expand_empty_elements = true;
    let mut stack: Vec<String> = Vec::new();
    let mut roots = 0;
    let mut cases = Vec::new();
    let mut current = None;
    let mut counts = [0u64; 4];
    let mut events = 0usize;
    loop {
        events += 1;
        if events > 100_000 {
            return Err("JUnit exceeds event allowance".into());
        }
        match reader
            .read_event()
            .map_err(|_| "JUnit contains malformed XML")?
        {
            Event::Start(event) => {
                let name = event.name().as_ref().to_owned();
                if stack.is_empty() {
                    roots += 1;
                    if roots != 1 || !matches!(name.as_str(), "testsuites" | "testsuite") {
                        return Err("JUnit requires one testsuites or testsuite root".into());
                    }
                }
                if stack.len() >= 128 {
                    return Err("JUnit exceeds nesting allowance".into());
                }
                if name == "testcase" {
                    if current.is_some() || stack.last().is_none_or(|p| p != "testsuite") {
                        return Err("JUnit testcase has invalid nesting".into());
                    }
                    if cases.len() >= 10_000 {
                        return Err("JUnit exceeds test-case allowance".into());
                    }
                    let mut attributes = serde_json::Map::new();
                    for attribute in event.attributes() {
                        let attribute =
                            attribute.map_err(|_| "JUnit has malformed or repeated attributes")?;
                        let key = attribute.key.as_ref();
                        let value = attribute
                            .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                            .map_err(|_| "JUnit attribute contains an unsupported entity")?;
                        attributes.insert(key.into(), json!(value));
                    }
                    if attributes
                        .get("name")
                        .and_then(Value::as_str)
                        .is_none_or(|v| v.is_empty())
                    {
                        return Err("JUnit testcase requires a name".into());
                    }
                    current = Some((Value::Object(attributes), None));
                } else if stack.last().is_some_and(|p| p == "testcase") {
                    let outcome = match name.as_str() {
                        "failure" => Some(1usize),
                        "error" => Some(2usize),
                        "skipped" => Some(3usize),
                        "system-out" | "system-err" | "properties" => None,
                        _ => {
                            return Err(
                                "JUnit testcase contains an unsupported outcome element".into()
                            );
                        }
                    };
                    if let Some(outcome) = outcome {
                        let (_, selected) =
                            current.as_mut().ok_or("JUnit outcome lacks a testcase")?;
                        if selected.replace(outcome).is_some() {
                            return Err("JUnit testcase has ambiguous outcomes".into());
                        }
                    }
                }
                stack.push(name);
            }
            Event::End(event) => {
                let name = stack
                    .pop()
                    .ok_or("JUnit closing tag lacks an open element")?;
                if name.as_str() != event.name().as_ref() {
                    return Err("JUnit has mismatched closing tags".into());
                }
                if name == "testcase" {
                    let (attributes, outcome) = current
                        .take()
                        .ok_or("JUnit testcase state is unavailable")?;
                    let outcome = outcome.unwrap_or(0);
                    counts[outcome] += 1;
                    cases.push(json!({"attributes":attributes,"outcome":(["passed","failed","error","skipped"][outcome])}));
                }
            }
            Event::DocType(_) => {
                return Err("JUnit document types and custom entities are unsupported".into());
            }
            Event::GeneralRef(reference) => {
                if !matches!(reference.as_ref(), "amp" | "lt" | "gt" | "apos" | "quot")
                    && reference
                        .resolve_char_ref()
                        .map_err(|_| "JUnit contains an invalid character reference")?
                        .is_none()
                {
                    return Err("JUnit contains an unsupported entity".into());
                }
            }
            Event::Text(text)
                if stack.is_empty() && text.as_ref().bytes().any(|b| !b.is_ascii_whitespace()) =>
            {
                return Err("JUnit contains text outside its root".into());
            }
            Event::Decl(declaration) => {
                if declaration
                    .version()
                    .map_err(|_| "Invalid JUnit XML version")?
                    != "1.0"
                {
                    return Err("JUnit requires XML version 1.0".into());
                }
                if declaration
                    .encoding()
                    .transpose()
                    .map_err(|_| "Invalid JUnit encoding")?
                    .is_some_and(|v| !v.eq_ignore_ascii_case("UTF-8"))
                {
                    return Err("JUnit declaration must select UTF-8".into());
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if roots != 1 || !stack.is_empty() || current.is_some() {
        return Err("JUnit document is incomplete".into());
    }
    Ok(
        json!({"test_cases":cases.len(),"passed":counts[0],"failed":counts[1],"errors":counts[2],"skipped":counts[3],"cases":cases,"coverage":"reported_test_cases"}),
    )
}
