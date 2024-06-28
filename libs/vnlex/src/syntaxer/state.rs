

pub struct Reduction<T> {
    pub node_id: u32,
    pub state_count: usize,
    pub production: T,
}

impl<T> Reduction<T> {
    pub const fn new(node_id: u32, state_count: usize, production: T) -> Self {
        Reduction { node_id, state_count, production }
    }
}

pub struct NodeJump {
    pub id: u32,
    pub target: usize,
}

impl NodeJump {
    pub const fn new(id: u32, target: usize) -> Self {
        NodeJump { id, target, }
    }
}

pub struct TokenJump {
    pub kind: u32,
    pub id: Option<u32>,
    pub followed_target: Option<usize>,
    pub not_followed_target: Option<usize>,
}

impl TokenJump {
    pub const fn new(kind: u32, id: Option<u32>, followed_target: Option<usize>, not_followed_target: Option<usize>) -> Self {
        Self { kind, id, followed_target, not_followed_target }
    }
}

pub struct State<'a, T> {
    pub node_jumps: &'a [NodeJump],
    pub token_jumps: &'a [TokenJump],
    pub reduction: Option<&'a T>,
}

impl<'a, T> State<'a, T> {
    pub const fn new(node_jumps: &'a [NodeJump], token_jumps: &'a [TokenJump], reduction: Option<&'a T>) -> State<'a, T> {
        State { node_jumps, token_jumps, reduction }
    }
}