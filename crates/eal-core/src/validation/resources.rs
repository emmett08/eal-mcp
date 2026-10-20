//! Streaming bounds for the Rust IR representation, without a JSON materialisation.
use crate::{Expression, semantic::Program};
use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};
use std::fmt;

/// Count scalar, compound and object-key visits in the Rust IR serializer.
/// This representation-based budget is deliberately distinct from Python's
/// dynamic dataclass type walk. No full JSON tree is constructed by this pass.
pub fn structural_value_count(program: &Program, limit: usize) -> Result<usize, String> {
    // Semantic predicates use a custom interchange serializer. Check their
    // expression shape before that serializer constructs its bounded view.
    let expressions = program
        .environments
        .values()
        .flat_map(|v| v.predicates.iter())
        .chain(program.evidence.values().flat_map(|v| v.predicates.iter()))
        .chain(program.reasoning.values().flat_map(|v| v.predicates.iter()))
        .chain(
            program
                .claims
                .values()
                .filter_map(|v| v.proposition.as_ref().map(|p| &p.result)),
        );
    for predicate in expressions {
        if let Some(expression) = &predicate.expression {
            expression_bound(expression, program, limit)?;
        }
    }
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("eal-ir-bound".into())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, || {
                let mut counter = Counter {
                    remaining: limit,
                    used: 0,
                    depth: 0,
                };
                program
                    .serialize(&mut counter)
                    .map_err(|error| error.to_string())?;
                Ok(counter.used)
            })
            .map_err(|error| format!("Cannot start bounded IR traversal: {error}"))?;
        worker
            .join()
            .map_err(|_| "Bounded IR traversal failed".to_owned())?
    })
}

fn expression_bound(
    expression: &Expression,
    program: &Program,
    limit: usize,
) -> Result<(), String> {
    let mut pending = vec![(expression, 0usize)];
    let mut count = 0usize;
    while let Some((value, depth)) = pending.pop() {
        count += 1;
        if count > program.limits.expression_nodes.min(limit)
            || depth > program.limits.expression_depth
        {
            return Err(
                "Typed IR exceeds the structural-value budget or expression-depth budget".into(),
            );
        }
        match value {
            Expression::Call { arguments, .. } => {
                pending.extend(arguments.iter().map(|value| (value, depth + 1)))
            }
            Expression::Unary { operand, .. } => pending.push((operand, depth + 1)),
            Expression::Binary { left, right, .. } => {
                pending.push((left, depth + 1));
                pending.push((right, depth + 1));
            }
            _ => {}
        }
    }
    Ok(())
}

#[derive(Debug)]
struct CountError;
impl fmt::Display for CountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Typed IR exceeds the structural-value budget or representation-depth budget")
    }
}
impl std::error::Error for CountError {}
impl ser::Error for CountError {
    fn custom<T: fmt::Display>(_: T) -> Self {
        Self
    }
}
struct Counter {
    remaining: usize,
    used: usize,
    depth: usize,
}
impl Counter {
    fn value(&mut self) -> Result<(), CountError> {
        if self.remaining == 0 {
            return Err(CountError);
        }
        self.remaining -= 1;
        self.used += 1;
        Ok(())
    }
    fn compound(&mut self) -> Result<Compound<'_>, CountError> {
        self.value()?;
        if self.depth >= 1024 {
            return Err(CountError);
        }
        self.depth += 1;
        Ok(Compound(self))
    }
}
struct Compound<'a>(&'a mut Counter);
impl Drop for Compound<'_> {
    fn drop(&mut self) {
        self.0.depth -= 1;
    }
}

macro_rules! scalar {($($name:ident($kind:ty)),*$(,)?)=>{$(fn $name(self,_value:$kind)->Result<(),CountError>{self.value()})*};}
impl<'a> ser::Serializer for &'a mut Counter {
    type Ok = ();
    type Error = CountError;
    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = Compound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = Compound<'a>;
    type SerializeStructVariant = Compound<'a>;
    scalar! {serialize_bool(bool),serialize_i8(i8),serialize_i16(i16),serialize_i32(i32),serialize_i64(i64),serialize_i128(i128),serialize_u8(u8),serialize_u16(u16),serialize_u32(u32),serialize_u64(u64),serialize_u128(u128),serialize_f32(f32),serialize_f64(f64),serialize_char(char),serialize_str(&str)}
    fn serialize_bytes(self, value: &[u8]) -> Result<(), CountError> {
        self.value()?;
        for _ in value {
            self.value()?;
        }
        Ok(())
    }
    fn serialize_none(self) -> Result<(), CountError> {
        self.value()
    }
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), CountError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), CountError> {
        self.value()
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), CountError> {
        self.value()
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
    ) -> Result<(), CountError> {
        self.value()
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), CountError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<(), CountError> {
        let compound = self.compound()?;
        compound.0.value()?;
        value.serialize(&mut *compound.0)
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Compound<'a>, CountError> {
        self.compound()
    }
    fn serialize_tuple(self, _len: usize) -> Result<Compound<'a>, CountError> {
        self.compound()
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Compound<'a>, CountError> {
        self.compound()
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Compound<'a>, CountError> {
        self.value()?;
        self.value()?;
        self.compound()
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Compound<'a>, CountError> {
        self.compound()
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Compound<'a>, CountError> {
        self.compound()
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Compound<'a>, CountError> {
        self.value()?;
        self.value()?;
        self.compound()
    }
}

macro_rules! element {
    ($trait:ident,$method:ident) => {
        impl $trait for Compound<'_> {
            type Ok = ();
            type Error = CountError;
            fn $method<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), CountError> {
                value.serialize(&mut *self.0)
            }
            fn end(self) -> Result<(), CountError> {
                Ok(())
            }
        }
    };
}
element!(SerializeSeq, serialize_element);
element!(SerializeTuple, serialize_element);
element!(SerializeTupleStruct, serialize_field);
element!(SerializeTupleVariant, serialize_field);
impl SerializeMap for Compound<'_> {
    type Ok = ();
    type Error = CountError;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), CountError> {
        key.serialize(&mut *self.0)
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), CountError> {
        value.serialize(&mut *self.0)
    }
    fn end(self) -> Result<(), CountError> {
        Ok(())
    }
}
impl SerializeStruct for Compound<'_> {
    type Ok = ();
    type Error = CountError;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), CountError> {
        self.0.value()?;
        value.serialize(&mut *self.0)
    }
    fn end(self) -> Result<(), CountError> {
        Ok(())
    }
}
impl SerializeStructVariant for Compound<'_> {
    type Ok = ();
    type Error = CountError;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), CountError> {
        self.0.value()?;
        value.serialize(&mut *self.0)
    }
    fn end(self) -> Result<(), CountError> {
        Ok(())
    }
}
