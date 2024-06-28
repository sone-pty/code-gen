use std::{rc::{Rc, Weak}, cell::{Cell, RefCell}, collections::{BTreeSet, BTreeMap}, fmt, path::PathBuf};

use super::{syntax_info::{Production, Item, self, SyntaxInfo, Terminator}, node_ins::{NodeIns, NodeInsMgr}};



struct LRItem {
    production: Rc<Production>,
    ins: NodeIns,
    position: usize,
    flag: Cell<bool>
}

impl LRItem {
    fn new(production: Rc<Production>, ins: NodeIns) -> Self {
        Self { production, ins, position: 0, flag: Default::default() }
    }

    fn with_position(production: Rc<Production>, ins: NodeIns, position: usize) -> Self {
        Self { production, ins, position, flag: Default::default() }
    }

    fn item(&self) -> Option<&Item> {
        self.production.items.get(self.position)
    }
}

impl PartialEq for LRItem {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.production, &other.production) && self.ins == other.ins && self.position == other.position
    }
}

impl<'a> Eq for LRItem {}

impl<'a> PartialOrd for LRItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LRItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.production.id.cmp(&other.production.id).then_with(|| self.ins.cmp(&other.ins)).then_with(|| self.position.cmp(&other.position))
    }
}

struct LRNodeJump {
    node: Weak<syntax_info::LRNode>,
    target: Weak<LRState>,
}

struct LRTokenJump {
    token: Terminator,
    followed_target: Weak<LRState>,
    target: Weak<LRState>,
}

#[derive(Default)]
struct LRState {
    id: usize,
    items: Vec<Rc<LRItem>>,
    key_items: BTreeSet<Rc<LRItem>>,
    reduction: RefCell<Vec<Rc<Production>>>,
    node_jumps: RefCell<Vec<LRNodeJump>>,
    token_jumps: RefCell<Vec<LRTokenJump>>,
}

impl LRState {
    fn new() -> Self {
        Default::default()
    }

    fn with_id(id: usize) -> Self {
        LRState {
            id,
            items: Default::default(),
            key_items: Default::default(),
            reduction: Default::default(),
            node_jumps: Default::default(),
            token_jumps: Default::default(),
        }
    }

    fn add_item(&mut self, item: Rc<LRItem>) -> bool {
        if self.items.contains(&item) {
            false
        } else {
            if item.position > 0 {
                self.key_items.insert(item.clone());
            }
            self.items.push(item);
            true
        }
    }

    fn add_node_jump(&self, node: &Weak<syntax_info::LRNode>, target: Weak<LRState>) {
        let node_id = node.upgrade().unwrap().id.get();
        let mut node_jumps = self.node_jumps.borrow_mut();
        match node_jumps.binary_search_by(|probe| probe.node.upgrade().unwrap().id.get().cmp(&node_id)) {
            Err(index) => {
                node_jumps.insert(index, LRNodeJump {
                    node: node.clone(),
                    target,
                });
            }
            _ => (),
        }
    }

    fn add_token_jump(&self, token: Terminator, followed_target: Weak<LRState>, target: Weak<LRState>) {
        let (kind, id) = token.kind_and_id();
        let mut token_jumps = self.token_jumps.borrow_mut();
        if let Err(index) = token_jumps.binary_search_by(|probe| {
            let (probe_kind, probe_id) = probe.token.kind_and_id();
            probe_kind.cmp(&kind).then_with(|| probe_id.cmp(&id))
        }) {
            token_jumps.insert(index, LRTokenJump {
                token,
                followed_target,
                target,
            })
        }
    }

    fn build_closure(&mut self, mgr: &mut NodeInsMgr) {
        let mut i = 0;
        while i < self.items.len() {
            let lr_item = self.items[i].clone();
            if let Some(item) = lr_item.item() {
                if let Item::Node(node, builder) = item {
                    let node = node.upgrade().unwrap();
                    let ins = if let Some(builder) = builder {
                        builder.build(mgr, &lr_item.ins)
                    } else {
                        lr_item.ins.clone()
                    };
                    for p in node.productions.borrow().iter() {  
                        if let Some(ref cond) = p.cond {
                            if !cond.validate(&ins) {
                                continue;
                            }
                        }
                        self.add_item(Rc::new(LRItem::new(p.clone(), ins.clone())));
                    }
                }
            }
            i += 1;
        }
    }
}

impl PartialEq for LRState {
    fn eq(&self, other: &Self) -> bool {
        self.key_items == other.key_items
    }
}

fn add_state<'a>(mgr: &mut NodeInsMgr, states: &mut Vec<Rc<LRState>>, mut new_state: Rc<LRState>) -> Weak<LRState> {
    for state in states.iter() {
        if *state == new_state {
            return Rc::downgrade(state);
        }
    }
    Rc::get_mut(&mut new_state).unwrap().build_closure(mgr);
    let ret = Rc::downgrade(&new_state);
    states.push(new_state);
    ret
}

fn build_other_states<'a>(mgr: &mut NodeInsMgr, states: &mut Vec<Rc<LRState>>) {
    let mut i = 0;
    while i < states.len() {
        let state = states[i].clone();
        i += 1;

        let mut iter = state.items.iter();
        while let Some(ia) = iter.next() {
            if ia.flag.get() {
                continue;
            }
            ia.flag.set(true);
            if let Some(ca) = ia.item() {
                match ca {
                    Item::Node(na, _) => {
                        let mut new_state = LRState::with_id(states.len());
                        new_state.add_item(Rc::new(LRItem::with_position(ia.production.clone(), ia.ins.clone(), ia.position + 1)));
                        for ib in iter.clone() {
                            match ib.item() {
                                Some(Item::Node(nb, _)) if Weak::ptr_eq(na, nb) => {
                                    ib.flag.set(true);
                                    new_state.add_item(Rc::new(LRItem::with_position(ib.production.clone(), ib.ins.clone(), ib.position + 1)));
                                }
                                _ => (),
                            }
                        }
                        let target = add_state(mgr, states, Rc::new(new_state));
                        state.add_node_jump(na, target);
                    }
                    Item::Token(followed, token) => {
                        let mut new_state_followed = LRState::new();
                        let mut new_state = LRState::new();
                        if *followed {
                            new_state_followed.add_item(Rc::new(LRItem::with_position(ia.production.clone(), ia.ins.clone(), ia.position + 1)));
                        } else {
                            new_state.add_item(Rc::new(LRItem::with_position(ia.production.clone(), ia.ins.clone(), ia.position + 1)));
                        }
                        for ib in iter.clone() {
                            if let Some(cb) = ib.item() {
                                if let Some(followed) = cb.is_same_token(token) {
                                    ib.flag.set(true);
                                    if followed {
                                        new_state_followed.add_item(Rc::new(LRItem::with_position(ib.production.clone(), ib.ins.clone(), ib.position + 1)));
                                    } else {
                                        new_state.add_item(Rc::new(LRItem::with_position(ib.production.clone(), ib.ins.clone(), ib.position + 1)));
                                    }
                                }
                            }
                        }
                        
                        let followed_target = if new_state_followed.items.is_empty() {
                            Weak::new()
                        } else {
                            new_state_followed.id = states.len();
                            add_state(mgr, states, Rc::new(new_state_followed))
                        };
                        let target = if new_state.items.is_empty() {
                            Weak::new()
                        } else {
                            new_state.id = states.len();
                            add_state(mgr, states, Rc::new(new_state))
                        };
                        state.add_token_jump(token.clone(), followed_target, target);
                    }
                }

            } else {
                state.reduction.borrow_mut().push(ia.production.clone());
            }
        }
    }
}


pub struct SyntaxDesc {
    keywords: BTreeMap<String, Rc<syntax_info::Literal>>,
    symbols: BTreeMap<char, Rc<syntax_info::Symbol>>,
    //inputs: BTreeMap<Cow<'a, str>, Rc<syntax_info::Literal<'a>>>,
    nodes: BTreeMap<String, Rc<syntax_info::LRNode>>,
    entry_nodes: Vec<Rc<syntax_info::LRNode>>,
    states: Vec<Rc<LRState>>,
    mods: BTreeSet<PathBuf>,
    custom_type: String,
}

impl SyntaxDesc {
    pub(crate) fn new(mut info: SyntaxInfo, custom_type: String) -> Option<Self> {
        debug_assert!(!info.entry_nodes.is_empty());
        let mut states = Vec::new();
        for (index, entry) in info.entry_nodes.iter().enumerate() {
            let node = Rc::new(syntax_info::LRNode::new(format!("#{}", entry.name), false));
            let production = node.generate_production(Production {
                id: 0,
                index: 0,
                node: Rc::downgrade(&node),
                items: vec![Item::Node(Rc::downgrade(entry), None)],
                real_count: 1,
                cond: None,
                reduction_index: 0,
            });
            info.nodes.insert(node.name.clone(), node);
            let mut state = LRState::new();
            state.id = index;
            let item = Rc::new(LRItem::new(production, info.ins_mgr.empty()));
            state.items.push(item.clone());
            state.key_items.insert(item);
            state.build_closure(&mut info.ins_mgr);
            states.push(Rc::new(state));
        }

        let mut fatal = false;

        build_other_states(&mut info.ins_mgr, &mut states);

        for state in states.iter() {
            let reduction = state.reduction.borrow();
            if reduction.len() > 1 {
                fatal = true;
                eprintln!("multi-reductions in state #{}:", state.id);
                for r in reduction.iter() {
                    let node = r.node.upgrade().unwrap();
                    eprint!("  {} <-", node.name);
                    for item in r.items.iter() {
                        match item {
                            Item::Node(t, _) => eprint!(" {}", t.upgrade().unwrap().name),
                            Item::Token(followed, t) => {
                                if *followed {
                                    eprint!(" _");
                                }
                                match t {
                                    Terminator::Keyword(t) => {
                                        eprint!(" \'{}\'", t.content);
                                    },
                                    Terminator::Symbol(t) => {
                                        eprint!(" \'{}\'", t.ch);
                                    },
                                    Terminator::Input(t) => {
                                        eprint!(" @{}", t.content);
                                    },
                                }
                            }
                        }
                    }
                    eprintln!();
                }
            }
        }

        if fatal {
            /*
            for state in states {
                eprintln!("state #{}:", state.id);
                for item in state.items.iter() {
                    eprint!("  {}({:?}) <-", item.production.node.upgrade().unwrap().name, item.ins);
                    for (i, e) in item.production.items.iter().enumerate() {
                        if i == item.position {
                            eprint!(" *");
                        }
                        match e {
                            Item::Node(t, _) => eprint!(" {}", t.upgrade().unwrap().name),
                            Item::Token(followed, t) => {
                                if *followed {
                                    eprint!(" _");
                                }
                                match t {
                                    Terminator::Keyword(t) => {
                                        eprint!(" \'{}\'", t.content);
                                    },
                                    Terminator::Symbol(t) => {
                                        eprint!(" \'{}\'", t.ch);
                                    },
                                    Terminator::Input(t) => {
                                        eprint!(" @{}", t.content);
                                    },
                                }
                            }
                        }
                    }
                    eprintln!();
                }
            }
            */
            return None;
        }

        Some(Self {
            keywords: info.keywords,
            symbols: info.symbols,
            //inputs: info.inputs,
            nodes: info.nodes,
            entry_nodes: info.entry_nodes,
            states,
            mods: info.mods,
            custom_type,
        })
    }

    pub fn mods(&self) -> &BTreeSet<PathBuf> {
        &self.mods
    }
    
}

impl fmt::Display for SyntaxDesc {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "use vnlex::{{syntaxer::{{Node, NonToken, RootNode, NodeDestructor, state::{{NodeJump, TokenJump, Reduction, State}}}}, token::Token}};")?;
        writeln!(f, "use std::ops::ControlFlow;")?;
        write!(f, "pub const DEF_KEYWORDS: &[(&str, u32)] = &[")?;
        if !self.keywords.is_empty() {
            writeln!(f)?;
            for kw in self.keywords.values() {
                writeln!(f, "    (\"{}\", {}),", kw.content.escape_default(), kw.id.get())?;
            }
        }
        writeln!(f, "];")?;
        write!(f, "pub const DEF_SYMBOLS: &[(char, u32)] = &[")?;
        if !self.symbols.is_empty() {
            writeln!(f)?;
            for sym in self.symbols.values() {
                writeln!(f, "    ('{}', {}),", sym.ch.escape_default(), sym.id.get())?;
            }
        }
        writeln!(f, "];")?;

        writeln!(f, "pub type ReductionType = Reduction<for<'a> fn(&mut Vec<Box<dyn Node<'a, {0}> + 'a>>, &mut Vec<Box<(Token<'a, {0}>, bool)>>) -> Box<dyn Node<'a, {0}> + 'a>>;", self.custom_type)?;

        for state in self.states.iter() {
            let node_jumps = state.node_jumps.borrow();
            write!(f, "pub const DEF_STATE_{}_NODE_JUMPS: &[NodeJump] = &[", state.id)?;
            if !node_jumps.is_empty() {
                writeln!(f)?;
                for jump in node_jumps.iter() {
                    writeln!(f, "    NodeJump::new({}, {}),", jump.node.upgrade().unwrap().id.get(), jump.target.upgrade().unwrap().id)?;
                }
            }
            writeln!(f, "];")?;
            let token_jumps = state.token_jumps.borrow();
            write!(f, "pub const DEF_STATE_{}_TOKEN_JUMPS: &[TokenJump] = &[", state.id)?;
            if !token_jumps.is_empty() {
                writeln!(f)?;
                for jump in token_jumps.iter() {
                    let (kind, id) = jump.token.kind_and_id();
                    let followed_target = jump.followed_target.upgrade().map(|t| t.id);
                    let target = jump.target.upgrade().map(|t| t.id);
                    writeln!(f, "    TokenJump::new({}, {:?}, {:?}, {:?}),", kind, id, followed_target, target)?;
                }
            }
            writeln!(f, "];")?;

            if let Some(reduction) = state.reduction.borrow().get(0) {
                let node = reduction.node.upgrade().unwrap();
                write!(f, "pub const DEF_STATE_{}_REDUCTION: ReductionType = Reduction::new({}, {}, |",
                    state.id, node.id.get(), reduction.items.len())?;

                if reduction.id == 0 {
                    writeln!(f, "_, _| unreachable!()")?;
                } else {
                    if reduction.items.len() == reduction.real_count {
                        write!(f, "nodes, _")?;
                    } else {
                        write!(f, "nodes, tokens")?;
                    }
                    writeln!(f, "| unsafe {{")?;
                    writeln!(f, "    let mut iter = nodes.drain(nodes.len() - {} ..);", reduction.items.len())?;
                    for _ in reduction.real_count .. reduction.items.len() {
                        writeln!(f, "    tokens.push(iter.next_back().unwrap_unchecked().downcast_unchecked());")?;
                    }
                    writeln!(f, "    Box::new(nodes::{}::p{}(", node.type_name(), reduction.reduction_index)?;
                    for _ in 0..reduction.real_count {
                        writeln!(f, "        iter.next().unwrap_unchecked().downcast_unchecked(),")?;
                    }
                    writeln!(f, "    ))")?;
                    write!(f, "}}")?;
                }

                writeln!(f, ");")?;
            }
        }

        writeln!(f, "pub type StateType<'r> = State<'r, ReductionType>;")?;

        writeln!(f, "pub const DEF_STATES: &[StateType] = &[")?;
        for state in self.states.iter() {
            write!(f, "    State::new(DEF_STATE_{0}_NODE_JUMPS, DEF_STATE_{0}_TOKEN_JUMPS, ", state.id)?;
            if state.reduction.borrow().is_empty() == false {
                writeln!(f, "Some(&DEF_STATE_{}_REDUCTION)),", state.id)?;
            } else {
                writeln!(f, "None),")?;
            }
        }
        writeln!(f, "];")?;

        writeln!(f, "#[allow(non_camel_case_types)]")?;
        writeln!(f, "pub mod nodes {{")?;
        
        for node in self.nodes.values() {
            if node.id.get() == 0 {
                continue;
            }
            let node_type_name = node.type_name();
            writeln!(f, "    pub enum {}<'a> {{", node_type_name)?;
            for reduction in node.reductions.borrow().iter() {
                write!(f, "        /// {} ->", node.name)?;
                for item in reduction.items.iter() {
                    match item {
                        Item::Node(t, _) => write!(f, " {}", t.upgrade().unwrap().name)?,
                        Item::Token(_, token) => {
                            match token {
                                Terminator::Keyword(t) => {
                                    write!(f, " \'{}\'", t.content)?
                                },
                                Terminator::Symbol(t) => {
                                    write!(f, " \'{}\'", t.ch)?
                                },
                                Terminator::Input(t) => {
                                    write!(f, " @{}", t.content)?
                                },
                            }
                        } 
                    }
                }
                writeln!(f)?;
                let mut iter = reduction.items.iter();
                write!(f, "        p{}(", reduction.index)?;
                match iter.next().unwrap() {
                    Item::Node(t, _) => write!(f, "Box<{}<'a>>", t.upgrade().unwrap().type_name())?,
                    Item::Token(..) => write!(f, "Box<(super::Token<'a, {}>, bool)>", self.custom_type)?,
                }
                while let Some(item) = iter.next() {
                    match item {
                        Item::Node(t, _) => write!(f, ", Box<{}<'a>>", t.upgrade().unwrap().type_name())?,
                        Item::Token(..) => write!(f, ", Box<(super::Token<'a, {}>, bool)>", self.custom_type)?,
                    }
                }
                writeln!(f, "),")?;
            }
            writeln!(f, "    }}")?;
            writeln!(f, "    impl<'a> super::Node<'a, {}> for {}<'a> {{", self.custom_type, node_type_name)?;
            writeln!(f, "        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, {0}>, bool)>, Box<dyn super::Node<'a, {0}> + 'a>> {{", self.custom_type)?;
            writeln!(f, "            Err(self)")?;
            writeln!(f, "        }}")?;
            writeln!(f, "        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, {}>) -> super::ControlFlow<()> {{", self.custom_type)?;
            writeln!(f, "            match *self {{")?;
            for reduction in node.reductions.borrow().iter() {
                write!(f, "                Self::p{}(i0", reduction.index)?;
                for i in 1..reduction.items.len() {
                    write!(f, ", i{}", i)?;
                }
                writeln!(f, ") => {{")?;
                for (index, item) in reduction.items.iter().enumerate() {
                    write!(f, "                    destructor.")?;
                    match item {
                        Item::Node(_, _) => write!(f, "non_token"),
                        Item::Token(_, _) => write!(f, "token"),
                    }?;
                    writeln!(f, "(i{})?;", index)?;
                }
                writeln!(f, "                }}")?;
            }
            writeln!(f, "            }}")?;
            writeln!(f, "            super::ControlFlow::Continue(())")?;
            writeln!(f, "        }}")?;
            writeln!(f, "    }}")?;

            writeln!(f, "    impl<'a> super::NonToken<'a, {}> for {}<'a> {{", self.custom_type, node.type_name())?;
            writeln!(f, "        fn name(&self) -> &str {{ \"{}\" }}", node.name)?;
            writeln!(f, "        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, {0}> + 'a>, Box<dyn super::NonToken<'a, {0}> + 'a>> {{", self.custom_type)?;
            writeln!(f, "            match *self {{")?;
            let mut rest = false;
            for reduction in node.reductions.borrow().iter() {
                if reduction.items.len() == 1 {
                    writeln!(f, "                Self::p{}(t) => Ok(t),", reduction.index)?;
                } else {
                    rest = true;
                }
            }
            if rest {
                writeln!(f, "                _ => Err(self)")?;
            }
            writeln!(f, "            }}")?;
            writeln!(f, "        }}")?;
            writeln!(f, "    }}")?;
        }

        for (index, entry) in self.entry_nodes.iter().enumerate() {
            writeln!(f, "    impl<'a> super::RootNode<'a, {}> for {}<'a> {{", self.custom_type, entry.type_name())?;
            writeln!(f, "        fn entry_state<'r, F>(states: &'r [super::State<'r, F>]) -> &'r super::State<'r, F> {{")?;
            writeln!(f, "            unsafe {{ states.get_unchecked({}) }}", index)?;
            writeln!(f, "        }}")?;
            writeln!(f, "    }}")?;
        }

        writeln!(f, "}}")?;

 
        Ok(())
    }
}