use std::{rc::Rc, collections::HashSet, hash::Hash, cmp::Ordering, borrow::Cow};



#[derive(Debug, Clone)]
pub struct NodeIns (Rc<StringSet>);

impl PartialEq for NodeIns {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for NodeIns {}

impl PartialOrd for NodeIns {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeIns {
    fn cmp(&self, other: &Self) -> Ordering {
        Rc::as_ptr(&self.0).cmp(&Rc::as_ptr(&other.0))
    }
}

impl NodeIns {
    pub fn contains(&self, param: &str) -> bool {
        self.0.as_ref().0.contains(param)
    }
}

pub struct NodeInsMgr {
    empty: NodeIns,
    mgr: HashSet<Rc<StringSet>>,
}

impl NodeInsMgr {
    pub fn new() -> Self {
        let empty = NodeIns (Default::default());
        let mut mgr = HashSet::new();
        mgr.insert(empty.0.clone());
        Self { empty, mgr }
    }

    pub fn empty(&self) -> NodeIns {
        self.empty.clone()
    }

    pub fn get<'a, I: Iterator<Item = Cow<'a, str>>>(&mut self, ss: I) -> NodeIns {
        let ss = Rc::new(StringSet (ss.map(|t| t.into_owned()).collect()));
        NodeIns(self.mgr.get_or_insert(ss).clone())
    }
}

impl Default for NodeInsMgr {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug, Default, PartialEq, Eq)]
struct StringSet (HashSet<String>);

impl Hash for StringSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.0.len());
        for s in self.0.iter() {
            s.hash(state);
        }
    }
}

pub trait NodeInsBuilder {
    fn build(&self, mgr: &mut NodeInsMgr, from: &NodeIns) -> NodeIns;
}

impl NodeInsBuilder for NodeIns {
    fn build(&self, _mgr: &mut NodeInsMgr, _from: &NodeIns) -> NodeIns {
        self.clone()
    }
}

pub struct NodeInsModifier (Vec<(String, bool)>);

impl NodeInsModifier {
    pub fn new() -> Self {
        Self (Vec::new())
    }

    pub fn add(&mut self, name: String, method: bool) {
        self.0.push((name, method));
    }
}

impl NodeInsBuilder for NodeInsModifier {
    fn build(&self, mgr: &mut NodeInsMgr, from: &NodeIns) -> NodeIns {
        if self.0.is_empty() {
            from.clone()
        } else {
            let mut ss = from.0.as_ref().0.clone();
            for (s, m) in self.0.iter() {
                if *m {
                    ss.insert(s.clone());
                } else {
                    ss.remove(s);
                }
            }
            NodeIns(mgr.mgr.get_or_insert(Rc::new(StringSet(ss))).clone())
        }
    }
}