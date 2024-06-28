
use std::borrow::Cow;

use crate::{Builder, Value, Visitor};


#[repr(C)]
pub struct StringRef {
    ptr: *const u8,
    len: usize,
}

impl From<&str> for StringRef {
    fn from(value: &str) -> Self {
        Self {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }
}

impl From<&Option<Cow<'_, str>>> for StringRef {
    fn from(value: &Option<Cow<str>>) -> Self {
        if let Some(t) = value {
            t.as_ref().into()
        } else {
            Self {
                ptr: 0 as _,
                len: 0,
            }
        }
    }
}

#[repr(C)]
pub struct LocationRef {
    row: usize,
    col: usize,
    end_row: usize,
    end_col: usize,
}

impl From<&crate::Location> for LocationRef {
    fn from(value: &crate::Location) -> Self {
        Self {
            row: value.row,
            col: value.col,
            end_row: value.end_row,
            end_col: value.end_col,
        }
    }
}

#[repr(C)]
pub struct CallbackFunctions {
    pub begin_object: unsafe extern "C" fn(*const (), &LocationRef, &StringRef, &StringRef, usize, usize),
    pub begin_object_field: unsafe extern "C" fn(*const (), &LocationRef, &StringRef),
    pub begin_array: unsafe extern "C" fn (*const (), &LocationRef, usize),
    pub null: unsafe extern "C" fn (*const (), &LocationRef),
    pub bool: unsafe extern "C" fn (*const (), &LocationRef, i32),
    pub int: unsafe extern "C" fn (*const (), &LocationRef, i32, u64),
    pub float: unsafe extern "C" fn (*const (), &LocationRef, f64),
    pub string: unsafe extern "C" fn (*const (), &LocationRef, &StringRef),
    pub ident: unsafe extern "C" fn (*const (), &LocationRef, &StringRef),
    pub end: unsafe extern "C" fn(*const ()),
    pub err: unsafe extern "C" fn (*const (), usize, usize, &StringRef),
}


#[no_mangle]
pub unsafe extern "C" fn parse_object(content: *const u8, len: usize, row: usize, col: usize, obj: *const (), cb: &CallbackFunctions) {
    let content = std::str::from_utf8_unchecked(std::slice::from_raw_parts(content, len));
    match crate::parse(content, row, col, None) {
        Ok(root) => {
            root.accept(&mut V(obj, cb));
        }
        Err(e) => {
            (cb.err)(obj, e.row, e.col, &e.msg.as_ref().into());
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn parse_value(content: *const u8, len: usize, row: usize, col: usize, obj: *const (), cb: &CallbackFunctions) {
    let content = std::str::from_utf8_unchecked(std::slice::from_raw_parts(content, len));
    match crate::parse_value(content, row, col, None) {
        Ok(Some(value)) => {
            value.accept(&mut V(obj, cb));
        }
        Err(e) => {
            (cb.err)(obj, e.row, e.col, &e.msg.as_ref().into());
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn load_object(data: *const u8, len: usize, obj: *const (), cb: &CallbackFunctions) {
    let data = std::slice::from_raw_parts(data, len);
    match crate::Object::load(data) {
        Ok(root) => {
            root.accept(&mut V(obj, cb));
        }
        Err(Some(e)) => {
            (cb.err)(obj, e.row, e.col, &e.msg.as_ref().into());
        }
        Err(None) => {
            (cb.err)(obj, 0, 0, &"invalid binary data".into());
        }
    }
}

#[no_mangle]
pub extern "C" fn object_builder_create() -> Box<Builder<'static>> {
    Box::new(Builder::new())
}

#[no_mangle]
pub extern "C" fn object_builder_discard(obj: Box<Builder<'static>>) {
    let _ = obj;
}

#[no_mangle]
pub extern "C" fn object_builder_push_null(obj: &mut Builder<'static>) -> i32 {
    if obj.push_null() { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn object_builder_push_bool(obj: &mut Builder<'static>, value: i32) -> i32 {
    if obj.push_bool(value != 0) { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn object_builder_push_int(obj: &mut Builder<'static>, minus: i32, value: u64) -> i32 {
    if obj.push_int(minus != 0, value) { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn object_builder_push_float(obj: &mut Builder<'static>, value: f64) -> i32 {
    if obj.push_float(value) { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn object_builder_push_string(obj: &mut Builder<'static>, value: &StringRef) -> i32 {
    let value = std::str::from_raw_parts(value.ptr, value.len);
    if obj.push_string(value.to_owned().into()) { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn object_builder_push_ident_unchecked(obj: &mut Builder<'static>, value: &StringRef) -> i32 {
    let value = std::str::from_raw_parts(value.ptr, value.len);
    if obj.push_ident_unchecked(value.to_owned().into()) { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn object_builder_begin_array(obj: &mut Builder<'static>) {
    obj.begin_array()
}

#[no_mangle]
pub unsafe extern "C" fn object_builder_begin_object(obj: &mut Builder<'static>, name: &StringRef, ns: &StringRef) {
    let name = if name.ptr.is_null() || name.len == 0 {
        "".into()
    } else {
        std::str::from_raw_parts(name.ptr, name.len).to_owned().into()
    };
    let ns = if ns.ptr.is_null() || ns.len == 0 {
        None
    } else {
        Some(std::str::from_raw_parts(ns.ptr, ns.len).to_owned().into())
    };
    obj.begin_object(name, ns)
}

#[no_mangle]
pub unsafe extern "C" fn object_builder_begin_attribute(obj: &mut Builder<'static>, name: &StringRef) {
    let name = std::str::from_raw_parts(name.ptr, name.len).to_owned().into();
    obj.begin_attribute(name);
}

#[no_mangle]
pub extern "C" fn object_builder_end(obj: &mut Builder<'static>) -> i32 {
    if obj.end() { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn object_builder_build(builder: Box<Builder<'static>>, obj: *const (), cb: &CallbackFunctions) {
    let root = builder.build();
    root.accept(&mut V(obj, cb));
}

#[no_mangle]
pub unsafe extern "C" fn object_builder_save(builder: Box<Builder<'static>>, mode: i32, obj: *const (), cb: extern "C" fn(*const (), *const u8, usize)) {
    let root = builder.build();
    if mode == 1 {
        let mut fmt = crate::format::Text::new();
        fmt.format(&root);
        cb(obj, fmt.output.as_ptr(), fmt.output.len());
    } else {
        let mut buf = Vec::new();
        let _ = root.save_binary(&mut buf);
        cb(obj, buf.as_ptr(), buf.len());
    }
}

struct V<'a>(*const (), &'a CallbackFunctions);
impl<'a> Visitor<'a> for V<'_> {
    fn object(&mut self, val: &crate::Object<'a>) {
        unsafe {
            (self.1.begin_object)(self.0, &val.location().into(), &val.name.as_ref().into(), &(&val.ns).into(), val.attributes.len(), val.base.elements.len());
            for (name, val) in val.attributes.iter() {
                (self.1.begin_object_field)(self.0, &name.location().into(), &name.value.as_ref().into());
                val.accept(self);
                (self.1.end)(self.0);
            }
            for e in val.base.elements.iter() {
                e.accept(self);
            }
            (self.1.end)(self.0);
        }
    }

    fn array(&mut self, val: &crate::Array<'a>) {
        unsafe {
            (self.1.begin_array)(self.0, &val.location().into(), val.elements.len());
            for e in val.elements.iter() {
                e.accept(self);
            }
            (self.1.end)(self.0);
        }
    }

    fn null(&mut self, val: &crate::Null) {
        unsafe { 
            (self.1.null)(self.0, &val.location().into());
        }
    }

    fn bool(&mut self, val: &crate::Boolean) {
        unsafe {
            (self.1.bool)(self.0, &val.location().into(), if val.value { 1 } else { 0 });
        }
    }

    fn int(&mut self, val: &crate::Integer) {
        unsafe {
            (self.1.int)(self.0, &val.location().into(), if val.minus { 1 } else { 0 }, val.value);
        }
    }

    fn float(&mut self, val: &crate::Float) {
        unsafe {
            (self.1.float)(self.0, &val.location().into(), val.value);
        }
    }

    fn string(&mut self, val: &crate::String<'a>) {
        unsafe {
            (self.1.string)(self.0, &val.location().into(), &val.value.as_ref().into());
        }
    }

    fn ident(&mut self, val: &crate::Ident<'a>) {
        unsafe {
            (self.1.ident)(self.0, &val.location().into(), &val.value.as_ref().into());
        }
    }
}