use std::borrow::Cow;

use vnlex::Location;

use crate::Object;



trait Container<'a> {

    fn push(&mut self, val: Box<dyn crate::Value<'a> + 'a>) -> bool;

    fn push_attribute(&mut self, name: Cow<'a, str>, value: Box<dyn crate::Value<'a> + 'a>) -> bool;

    fn end(self: Box<Self>, parent: &mut dyn Container<'a>) -> bool;
}

impl<'a> Container<'a> for crate::Array<'a> {
    
    fn push(&mut self, val: Box<dyn crate::Value<'a> + 'a>) -> bool {
        self.elements.push(val);
        true
    }

    fn push_attribute(&mut self, _: Cow<'a, str>, _: Box<dyn crate::Value<'a> + 'a>) -> bool {
        false
    }

    fn end(self: Box<Self>, parent: &mut dyn Container<'a>) -> bool {
        parent.push(self)
    }
}

impl<'a> Container<'a> for crate::Object<'a> {

    fn push(&mut self, val: Box<dyn crate::Value<'a> + 'a>) -> bool {
        self.base.elements.push(val);
        true
    }

    fn push_attribute(&mut self, name: Cow<'a, str>, value: Box<dyn crate::Value<'a> + 'a>) -> bool {
        let name = crate::Ident { location: Location::DEFAULT, value: name };
        self.attributes.insert(name, value)
    }

    fn end(self: Box<Self>, parent: &mut dyn Container<'a>) -> bool {
        parent.push(self)
    }
}

struct Attribute<'a> {
    name: Cow<'a, str>,
    value: Option<Box<dyn crate::Value<'a> + 'a>>,
}

impl<'a> Container<'a> for Attribute<'a> {
    fn push(&mut self, val: Box<dyn crate::Value<'a> + 'a>) -> bool {
        if self.value.is_some() {
            false
        } else {
            self.value = Some(val);
            true
        }
    }

    fn push_attribute(&mut self, _: Cow<'a, str>, _: Box<dyn crate::Value<'a> + 'a>) -> bool {
        false
    }

    fn end(self: Box<Self>, parent: &mut dyn Container<'a>) -> bool {
        let Self { name, value } = *self;
        if let Some(value) = value {
            parent.push_attribute(name, value)
        } else {
            false
        }
    }
}

pub struct Builder<'a> {
    root: Option<Object<'a>>,
    stack: Vec<Box<dyn Container<'a> + 'a>>,
}

impl<'a> Builder<'a> {
    pub fn new() -> Self {
        Self {
            root: Some(Object {
                base: crate::Array { location: Location::DEFAULT, elements: Vec::new() },
                ns: None,
                name: "".into(),
                attributes: crate::attributes::Attributes::new(),
            }),
            stack: Vec::new(),
        }
    }

    fn top(&mut self) -> &mut (dyn Container<'a> + 'a) {
        if let Some(t) = self.stack.last_mut() {
            t.as_mut()
        } else {
            unsafe { self.root.as_mut().unwrap_unchecked() }
        }
    }

    pub fn push_null(&mut self) -> bool {
        self.top().push(Box::new(crate::Null(Location::DEFAULT)))
    }

    pub fn push_bool(&mut self, value: bool) -> bool {
        self.top().push(Box::new(crate::Boolean {
            location: Location::DEFAULT,
            value,
        }))
    }

    pub fn push_int(&mut self, minus: bool, value: u64) -> bool {
        self.top().push(Box::new(crate::Integer {
            location: Location::DEFAULT,
            minus,
            value,
        }))
    }

    pub fn push_float(&mut self, value: f64) -> bool {
        self.top().push(Box::new(crate::Float {
            location: Location::DEFAULT,
            value,
        }))
    }

    pub fn push_string(&mut self, value: Cow<'a, str>) -> bool {
        self.top().push(Box::new(crate::String {
            location: Location::DEFAULT,
            value,
        }))
    }

    pub unsafe fn push_ident_unchecked(&mut self, value: Cow<'a, str>) -> bool {
        self.top().push(Box::new(crate::Ident {
            location: Location::DEFAULT,
            value,
        }))
    }

    pub fn begin_array(&mut self) {
        self.stack.push(Box::new(crate::Array {
            location: Location::DEFAULT,
            elements: Vec::new(),
        }));
    }

    pub fn begin_object(&mut self, name: Cow<'a, str>, ns: Option<Cow<'a, str>>) {
        self.stack.push(Box::new(Object {
            base: crate::Array {
                location: Location::DEFAULT,
                elements: Vec::new(),
            },
            ns,
            name,
            attributes: crate::attributes::Attributes::new(),
        }))
    }

    pub fn begin_attribute(&mut self, name: Cow<'a, str>) {
        self.stack.push(Box::new(Attribute {
            name,
            value: None,
        }))
    }

    pub fn end(&mut self) -> bool {
        if let Some(t) = self.stack.pop() {
            t.end(self.top())
        } else {
            false
        }
    }

    pub fn build(mut self) -> Object<'a> {
        while let Some(t) = self.stack.pop() {
            t.end(self.top());
        }
        unsafe { self.root.take().unwrap_unchecked() }
    }
}