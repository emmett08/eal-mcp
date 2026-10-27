//! Interpret one array of explicit selections without merging distinct populations.
use serde_json::Value;

pub(super) fn values(value: &Value) -> Result<&[Value], String> {
    let selected = value
        .as_array()
        .ok_or("Operation requires an array of selected inputs")?;
    Ok(match selected.as_slice() {
        [Value::Array(values)] => values.as_slice(),
        values => values,
    })
}
