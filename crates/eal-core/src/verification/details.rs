use serde_json::{Value, json};

// JSON browser consumers cannot preserve integer numbers outside ±(2^53−1).
// This is an inspection projection only: checks always use the original values.
pub(super) fn project(value: &mut Value) {
    match value {
        Value::Number(number) if !number.to_string().contains(['.', 'e', 'E']) => {
            let safe = number.as_i64().is_some_and(|value| {
                (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&value)
            });
            if !safe {
                *value = json!({"schema":"eal.exact-number/1","numerator":number.to_string(),"denominator":"1","unit":"1"});
            }
        }
        Value::Array(values) => {
            for value in values {
                project(value)
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                project(value)
            }
        }
        _ => {}
    }
}
