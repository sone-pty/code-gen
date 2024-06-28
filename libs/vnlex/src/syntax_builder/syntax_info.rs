use std::{cell::{Cell, RefCell}, rc::{Rc, Weak}, collections::{BTreeMap, BTreeSet}, borrow::Cow, path::{Path, PathBuf}, fs::File, io::Read, fmt, sync::Arc};


use crate::{ParseError, token::tokenizers::{KIND_IDENT, KIND_STRING, KIND_INTEGER, KIND_FLOAT, KIND_KEYWORD, KIND_SYMBOL, KIND_CODE_BLOCK}, cursor::Cursor};

use super::{states, node_ins::{NodeIns, NodeInsMgr, NodeInsModifier, NodeInsBuilder}};

pub struct Literal {
    pub content: String,
    pub id: Cell<u32>,
}

pub struct Symbol {
    pub ch: char,
    pub id: Cell<u32>,
}

pub struct LRNode {
    pub name: String,
    pub id: Cell<u32>,
    pub entry: Cell<bool>,
    pub productions: RefCell<Vec<Rc<Production>>>,
    pub reductions: RefCell<Vec<Reduction>>,
}

impl LRNode {

    pub fn new(name: String, entry: bool) -> Self {
        Self {
            name,
            id: Default::default(),
            entry: Cell::new(entry),
            productions: Default::default(),
            reductions: Default::default(),
        }
    }

    pub fn type_name(&self) -> TypeName {
        TypeName(self.name.as_str())
    }

    pub fn generate_production(&self, mut production: Production) -> Rc<Production> {
        'index: loop {
            let mut reductions = self.reductions.borrow_mut();
            'reduction: for reduction in reductions.iter() {
                if reduction.items.len() == production.real_count {
                    for i in 0..production.real_count {
                        let a = unsafe { reduction.items.get_unchecked(i) };
                        let b = unsafe { production.items.get_unchecked(i) };
                        if !a.is_same(b) {
                            continue 'reduction;
                        }
                    }
                    production.reduction_index = reduction.index;
                    break 'index;
                }
            }
            production.reduction_index = reductions.len();
            reductions.push(Reduction {
                index: production.reduction_index,
                items: production.items.iter().take(production.real_count).map(|t| t.clone()).collect(),
            });
            break;
        }
        let mut productions = self.productions.borrow_mut();
        production.index = productions.len();
        let production = Rc::new(production);
        productions.push(production.clone());
        production
    }

}

pub struct TypeName<'a> (&'a str);

impl fmt::Display for TypeName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            "as"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            => write!(f, "r#{}", self.0),
            _ => f.write_str(self.0),
        }
    }
}

pub struct Production {
    pub id: u32,
    pub index: usize,
    pub node: Weak<LRNode>,
    pub items: Vec<Item>,
    pub real_count: usize,
    pub cond: Option<Box<dyn ProductionCond>>,
    pub reduction_index: usize,
}

pub struct Reduction {
    pub index: usize,
    pub items: Vec<Item>,
}

pub trait ProductionCond {
    fn validate(&self, ins: &NodeIns) -> bool;
}

#[derive(Clone)]
pub enum Item {
    Node(Weak<LRNode>, Option<Rc<dyn NodeInsBuilder>>),
    Token(bool, Terminator),
}

impl Item {


    pub(crate) fn is_same_token(&self, token: &Terminator) -> Option<bool> {
        match self {
            Item::Token(followed, t) if token == t => Some(*followed),
            _ => None,
        }
    }

    pub(crate) fn is_same(&self, other: &Self) -> bool {
        match (self, other) {
            (Item::Node(a, _), Item::Node(b, _)) => {
                Weak::ptr_eq(a, b)
            }
            (Item::Token(_, a), Item::Token(_, b)) => {
                *a == *b
            }
            _ => false,
        }
    }

    fn with_keyword(followed: bool, kw: Rc<Literal>) -> Self {
        Self::Token(followed, Terminator::Keyword(kw))
    }

    fn with_symbol(followed: bool, sy: Rc<Symbol>) -> Self {
        Self::Token(followed, Terminator::Symbol(sy))
    }
    
    fn with_input(followed: bool, ip: Rc<Literal>) -> Self {
        Self::Token(followed, Terminator::Input(ip))
    }
}

#[derive(Clone)]
pub enum Terminator {
    Keyword(Rc<Literal>),
    Symbol(Rc<Symbol>),
    Input(Rc<Literal>),
}

impl PartialEq for Terminator {
    fn eq(&self, other: &Self) -> bool {
        use Terminator::*;
        match (self, other) {
            (Keyword(a), Keyword(b)) if Rc::ptr_eq(a, b) => true,
            (Symbol(a), Symbol(b)) if Rc::ptr_eq(a, b) => true,
            (Input(a), Input(b)) if Rc::ptr_eq(a, b) => true,
            _ => false,
        }
    }
}

impl Eq for Terminator {}

impl Terminator {
    pub(crate) fn kind_and_id(&self) -> (u32, Option<u32>) {
        use Terminator::*;
        match self {
            Keyword(a) => (KIND_KEYWORD, Some(a.id.get())),
            Symbol(a) => (KIND_SYMBOL, Some(a.id.get())),
            Input(a) => (a.id.get(), None),
        }
    }
}

pub struct SyntaxInfo {
    pub(crate) keywords: BTreeMap<String, Rc<Literal>>,
    pub(crate) symbols: BTreeMap<char, Rc<Symbol>>,
    pub(crate) inputs: BTreeMap<String, Rc<Literal>>,
    pub(crate) nodes: BTreeMap<String, Rc<LRNode>>,
    pub(crate) entry_nodes: Vec<Rc<LRNode>>,
    pub(crate) mods: BTreeSet<PathBuf>,
    pub(crate) ins_mgr: NodeInsMgr,
    last_id: u32, 
}

type Result<T> = std::result::Result<T, ParseError>;

impl SyntaxInfo {

    pub(crate) fn generate_id(&mut self) -> u32 {
        self.last_id += 1;
        self.last_id
    }

    pub(crate) fn set_input(&mut self, name: String, id: u32) {
        use std::collections::btree_map::Entry::*;
        match self.inputs.entry(name) {
            Vacant(e) => {
                let name = e.key().clone();
                e.insert(Rc::new(Literal {
                    content: name,
                    id: Cell::new(id),
                }));
            },
            Occupied(e) => e.get().id.set(id),
        }
    }

    pub(crate) fn get_node(&mut self, name: &str, entry: bool) -> Rc<LRNode> {
        if let Some(node) = self.nodes.get(name) {
            if entry {
                node.entry.set(true);
            }
            node.clone()
        } else {
            let node = Rc::new(LRNode::new(name.to_owned(), entry));
            self.nodes.insert(node.name.clone(), node.clone());
            node
        }
    }

    pub(crate) fn get_keyword(&mut self, content: &str) -> Rc<Literal> {
        if let Some(kw) = self.keywords.get(content) {
            kw.clone()
        } else {
            let kw = Rc::new(Literal {
                content: content.to_owned(),
                id: Cell::new(0)
            });
            self.keywords.insert(kw.content.clone(), kw.clone());
            kw
        }
    }

    pub(crate) fn get_symbol(&mut self, ch: char) -> Rc<Symbol> {
        self.symbols.entry(ch).or_insert_with(|| Rc::new(Symbol {
            ch,
            id: Cell::new(0),
        })).clone()
    }

    pub(crate) fn get_input(&mut self, name: &str) -> Rc<Literal> {
        if let Some(input) = self.inputs.get(name) {
            input.clone()
        } else {
            let input = Rc::new(Literal {
                content: name.to_owned(),
                id: Cell::new(0),
            });
            self.inputs.insert(input.content.clone(), input.clone());
            input
        }
    }

    pub(crate) fn parse<'a>(cursor: &mut Cursor<'a>, mod_root: &Path) -> Result<Self> {
        let mut info = Self {
            keywords: Default::default(),
            symbols: Default::default(),
            inputs: Default::default(),
            nodes: Default::default(),
            entry_nodes: Default::default(),
            mods: Default::default(),
            ins_mgr: Default::default(),
            last_id: 0,
        };

        let root = super::parse_root(cursor)?;

        if let Some(root) = root {
            info.set_input("ident".to_owned(), KIND_IDENT);
            info.set_input("string".to_owned(), KIND_STRING);
            info.set_input("integer".to_owned(), KIND_INTEGER);
            info.set_input("float".to_owned(), KIND_FLOAT);
            info.set_input("code_block".to_owned(), KIND_CODE_BLOCK);
            info.parse_root(root, mod_root)?;

            let mut fatal = false;
            let mut id = 0;
            for kw in info.keywords.values() {
                id += 1;
                kw.id.set(id);
            }
            id = 0;
            for sym in info.symbols.values() {
                id += 1;
                sym.id.set(id);
            }
            id = 0;
            for node in info.nodes.values() {
                id += 1;
                node.id.set(id);
                if node.productions.borrow().len() == 0 {
                    fatal = true;
                    eprintln!("undefined node `{}`", node.name);
                }
            }
            
            for input in info.inputs.values() {
                if input.id.get() == 0 {
                    fatal = true;
                    eprintln!("undefined token `{}`", input.content);
                }
            }
    
            if fatal {
                return Err(ParseError::with_cursor(cursor, "fatal error(s) occurred"));
            }
    
            for node in info.nodes.values().filter(|t| t.entry.get()) {
                info.entry_nodes.push(node.clone());
            }
        }



        if info.entry_nodes.is_empty() {
            if let Some(t) = info.nodes.get("script") {
                info.entry_nodes.push(t.clone());
            } else {
                return Err(ParseError::with_cursor(cursor, "entry node(s) or `script` node not found"));
            }
        }

        Ok(info)
    }

    fn parse_root(&mut self, root: Box<states::nodes::script>, mod_root: &Path) -> Result<()> {
        match *root {
            states::nodes::script::p0(node) => self.parse_script_item(node, mod_root),
            states::nodes::script::p1(n1, n2) => {
                self.parse_root(n1, mod_root)?;
                self.parse_script_item(n2, mod_root)
            }
        }
    }

    fn parse_script_item(&mut self, node: Box<states::nodes::script_item>, mod_root: &Path) -> Result<()> {
        match *node {
            states::nodes::script_item::p0(node) => self.parse_token(node),
            states::nodes::script_item::p1(node) => self.parse_production(node),
            states::nodes::script_item::p2(node) => self.parse_import(node, mod_root),
        }
    }

    fn parse_import(&mut self, node: Box<states::nodes::import>, mod_root: &Path) -> Result<()> {
        match *node {
            states::nodes::import::p0(_, t, _) => {
                let name_token = &t.0;
                let name = name_token.data.get_string().unwrap().as_ref();
                let mod_root = mod_root.join(name);
                let mut file_path = mod_root.with_extension("lex");
                let mut file = if let Ok(file) = File::open(&file_path) {
                    file
                } else {
                    file_path = mod_root.join("mod.lex");
                    if let Ok(file) = File::open(&file_path) {
                        file
                    } else {
                        return Err(ParseError::with_location(
                            &name_token.location,
                            format!("import `{}` failed", name),
                        ));
                    }
                };
                if !self.mods.contains(&file_path) {
                    self.mods.insert(file_path.clone());
                    let mut content = String::new();
                    if let Err(_) = file.read_to_string(&mut content) {
                        return Err(ParseError::with_location(
                            &name_token.location,
                            format!("import `{}` failed", name),
                        ));
                    }
                    let mut cursor = Cursor::new(content.as_str(), 0, 0, Arc::new(file_path).into());
                    super::parse_root(&mut cursor)
                        .and_then(|t| { 
                            if let Some(t) = t {
                                self.parse_root(t, &mod_root)
                            } else {
                                Ok(())
                            }
                        })?
                }
                Ok(())
            }
        }
    }

    fn parse_token(&mut self, node: Box<states::nodes::token>) -> Result<()> {
        match *node {
            states::nodes::token::p0(_, name, _, id, _) => {
                let name = name.0.data.get_string().unwrap().as_ref();
                let id = id.0.data.get_integer().unwrap() as u32;
                self.set_input(name.to_owned(), id);
            }
        }
        Ok(())
    }

    fn parse_production(&mut self, node: Box<states::nodes::production>) -> Result<()> {
        match *node {
            states::nodes::production::p0(name, _, stmt, _) => {
                let node = self.get_node(name.0.data.get_string().unwrap(), false);
                self.parse_statement_list(stmt, &node)
            }
            states::nodes::production::p1(_, name, _, stmt, _) => {
                let node = self.get_node(name.0.data.get_string().unwrap(), true);
                self.parse_statement_list(stmt, &node)
            }
        }
    }

    fn parse_statement_list(&mut self, node: Box<states::nodes::statement_list>, lr_node: &Rc<LRNode>) -> Result<()> {
        match *node {
            states::nodes::statement_list::p0(node) => self.parse_statement_with_cond(node, lr_node),
            states::nodes::statement_list::p1(list, _, node) => {
                self.parse_statement_list(list, lr_node)?;
                self.parse_statement_with_cond(node, lr_node)
            }
        }
    }

    fn parse_statement_with_cond(&mut self, node: Box<states::nodes::statement_with_cond>, lr_node: &Rc<LRNode>) -> Result<()> {
        match *node {
            states::nodes::statement_with_cond::p0(node) => self.parse_statement(node, lr_node, None),
            states::nodes::statement_with_cond::p1(node, _, cond) => {
                struct Cond (String);
                impl ProductionCond for Cond {
                    fn validate(&self, ins: &NodeIns) -> bool {
                        ins.contains(self.0.as_str())
                    }
                }
                self.parse_statement(node, lr_node, Some(Box::new(Cond(cond.0.data.into_string().unwrap().into_owned()))))
            }
            states::nodes::statement_with_cond::p2(node, _, _, cond) => {
                struct Cond (String);
                impl ProductionCond for Cond {
                    fn validate(&self, ins: &NodeIns) -> bool {
                        !ins.contains(self.0.as_str())
                    }
                }
                self.parse_statement(node, lr_node, Some(Box::new(Cond(cond.0.data.into_string().unwrap().into_owned()))))
            }
        }
    }

    fn parse_statement(&mut self, node: Box<states::nodes::statement>, lr_node: &Rc<LRNode>, cond: Option<Box<dyn ProductionCond>>) -> Result<()> {
        let mut production = Production {
            id: self.generate_id(),
            index: 0,
            node: Rc::downgrade(lr_node),
            items: Vec::new(),
            cond,
            real_count: 0,
            reduction_index: 0,
        };
        match *node {
            states::nodes::statement::p0(item) => self.parse_item(item, &mut production)?,
            states::nodes::statement::p1(item, _, not_followed_item_list) => {
                self.parse_item(item, &mut production)?;
                self.parse_not_followed_item_list(not_followed_item_list, &mut production)?;
            }
            states::nodes::statement::p2(item, followed_item_list) => {
                self.parse_item(item, &mut production)?;
                self.parse_followed_item_list(followed_item_list, &mut production)?;
            }
            states::nodes::statement::p3(item, followed_item_list, _, not_followed_item_list) => {
                self.parse_item(item, &mut production)?;
                self.parse_followed_item_list(followed_item_list, &mut production)?;
                self.parse_not_followed_item_list(not_followed_item_list, &mut production)?;
            }
        }
        lr_node.generate_production(production);
        Ok(())
    }

    fn parse_item(&mut self, node: Box<states::nodes::item>, production: &mut Production) -> Result<()> {
        match *node {
            states::nodes::item::p0(name) => {
                let node = self.get_node(name.0.data.get_string().unwrap(), false);
                production.items.push(Item::Node(Rc::downgrade(&node), None));
                production.real_count += 1;
            }
            states::nodes::item::p1(name) => {
                let kw = self.get_keyword(name.0.data.get_string().unwrap());
                production.items.push(Item::with_keyword(false, kw));
                production.real_count += 1;
            }
            states::nodes::item::p2(name) => {
                let content = name.0.data.get_string().unwrap();
                let mut chars = content.chars();
                if let Some(ch) = chars.next() {
                    production.items.push(Item::with_symbol(false, self.get_symbol(ch)));
                    production.real_count += 1;
                    while let Some(ch) = chars.next() {
                        production.items.push(Item::with_symbol(true, self.get_symbol(ch)));
                        production.real_count += 1;
                    }
                }
            }
            states::nodes::item::p3(_, name) => {
                let input = self.get_input(name.0.data.get_string().unwrap());
                production.items.push(Item::with_input(false, input));
                production.real_count += 1;
            }
            states::nodes::item::p4(name, _, _) => {
                let node = self.get_node(name.0.data.get_string().unwrap(), false);
                production.items.push(Item::Node(Rc::downgrade(&node), Some(Rc::new(self.ins_mgr.empty()))));
                production.real_count += 1;
            }
            states::nodes::item::p5(name, _, param_list, _) |
            states::nodes::item::p6(name, _, param_list, _, _) => {
                let node = self.get_node(name.0.data.get_string().unwrap(), false);
                let mut params = Vec::new();
                self.parse_node_param_list(param_list, &mut params);
                production.items.push(Item::Node(Rc::downgrade(&node), Some(Rc::new(self.ins_mgr.get(params.into_iter())))));
                production.real_count += 1;
            }
            states::nodes::item::p7(name, _, _, modifier_list, _) |
            states::nodes::item::p8(name, _, _, modifier_list, _, _) => {
                let node = self.get_node(name.0.data.get_string().unwrap(), false);
                let mut modifier = NodeInsModifier::new();
                self.parse_node_modifier_param_list(modifier_list, &mut modifier);
                production.items.push(Item::Node(Rc::downgrade(&node), Some(Rc::new(modifier))));
                production.real_count += 1;
            }
            
        }
        Ok(())
    }

    fn parse_node_param_list<'a>(&mut self, node: Box<states::nodes::node_param_list<'a>>, params: &mut Vec<Cow<'a, str>>) {
        match *node {
            states::nodes::node_param_list::p0(p) => {
                params.push(p.0.data.into_string().unwrap());
            },
            states::nodes::node_param_list::p1(node, _, p) => {
                self.parse_node_param_list(node, params);
                params.push(p.0.data.into_string().unwrap());
            }
        }
    }

    fn parse_node_modifier_param_list(&mut self, node: Box<states::nodes::node_modifier_param_list>, modifier: &mut NodeInsModifier) {
        match *node {
            states::nodes::node_modifier_param_list::p0(node) => self.parse_node_modifier_param(node, modifier),
            states::nodes::node_modifier_param_list::p1(list, _, node) => {
                self.parse_node_modifier_param_list(list, modifier);
                self.parse_node_modifier_param(node, modifier);
            }
        }
    }

    fn parse_node_modifier_param(&mut self, node: Box<states::nodes::node_modifier_param>, modifier: &mut NodeInsModifier) {
        let name;
        let method;
        match *node {
            states::nodes::node_modifier_param::p0(token) => {
                name = token.0.data.get_string().unwrap().to_string();
                method = true;
            }
            states::nodes::node_modifier_param::p1(_, token) => {
                name = token.0.data.get_string().unwrap().to_string();
                method = false;
            }
        }
        modifier.add(name, method);
    }

    fn parse_followed_item_list(&mut self, node: Box<states::nodes::followed_item_list>, production: &mut Production) -> Result<()> {
        match *node {
            states::nodes::followed_item_list::p0(item) => self.parse_followed_item(item, production),
            states::nodes::followed_item_list::p1(list, item) => {
                self.parse_followed_item_list(list, production)?;
                self.parse_followed_item(item, production)
            }
        }
    }

    fn parse_followed_item(&mut self, node: Box<states::nodes::followed_item>, production: &mut Production) -> Result<()> {
        match *node {
            states::nodes::followed_item::p0(item) => self.parse_item(item, production)?,
            states::nodes::followed_item::p1(_, kw) => {
                let kw = self.get_keyword(kw.0.data.get_string().unwrap());
                production.items.push(Item::with_keyword(true, kw));
                production.real_count += 1;
            }
            states::nodes::followed_item::p2(_, sy) => {
                let content = sy.0.data.get_string().unwrap();
                let mut chars = content.chars();
                if let Some(ch) = chars.next() {
                    production.items.push(Item::with_symbol(true, self.get_symbol(ch)));
                    production.real_count += 1;
                    while let Some(ch) = chars.next() {
                        production.items.push(Item::with_symbol(true, self.get_symbol(ch)));
                        production.real_count += 1;
                    }
                }
            }
            states::nodes::followed_item::p3(_, _, name) => {
                let input = self.get_input(name.0.data.get_string().unwrap());
                production.items.push(Item::with_input(true, input));
                production.real_count += 1;
            }
        }
        Ok(())
    }

    fn parse_not_followed_item_list(&mut self, node: Box<states::nodes::not_followed_item_list>, production: &mut Production) -> Result<()> {
        match *node {
            states::nodes::not_followed_item_list::p0(item) => self.parse_not_followed_item(item, production),
            states::nodes::not_followed_item_list::p1(list, item) => {
                self.parse_not_followed_item_list(list, production)?;
                self.parse_not_followed_item(item, production)
            }
        }
    }

    fn parse_not_followed_item(&mut self, node: Box<states::nodes::not_followed_item>, production: &mut Production) -> Result<()> {
        match *node {
            states::nodes::not_followed_item::p0(kw) => {
                let kw = self.get_keyword(kw.0.data.get_string().unwrap());
                production.items.push(Item::with_keyword(false, kw));
            }
            states::nodes::not_followed_item::p1(sy) => {
                let content = sy.0.data.get_string().unwrap();
                let mut chars = content.chars();
                if let Some(ch) = chars.next() {
                    production.items.push(Item::with_symbol(false, self.get_symbol(ch)));
                    while let Some(ch) = chars.next() {
                        production.items.push(Item::with_symbol(true, self.get_symbol(ch)));
                    }
                }   
            }
            states::nodes::not_followed_item::p2(_, name) => {
                let input = self.get_input(name.0.data.get_string().unwrap());
                production.items.push(Item::with_input(false, input));
            }
            states::nodes::not_followed_item::p3(_, kw) => {
                let kw = self.get_keyword(kw.0.data.get_string().unwrap());
                production.items.push(Item::with_keyword(true, kw));
            }
            states::nodes::not_followed_item::p4(_, sy) => {
                let content = sy.0.data.get_string().unwrap();
                let mut chars = content.chars();
                if let Some(ch) = chars.next() {
                    production.items.push(Item::with_symbol(true, self.get_symbol(ch)));
                    while let Some(ch) = chars.next() {
                        production.items.push(Item::with_symbol(true, self.get_symbol(ch)));
                    }
                }   
            }
            states::nodes::not_followed_item::p5(_, _, name) => {
                let input = self.get_input(name.0.data.get_string().unwrap());
                production.items.push(Item::with_input(true, input));
            }
        }
        Ok(())
    }
}