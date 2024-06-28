use crate::{syntaxer::{Node, NonToken, RootNode, NodeDestructor, state::{NodeJump, TokenJump, Reduction, State}}, token::Token};
use std::ops::ControlFlow;
pub const DEF_KEYWORDS: &[(&str, u32)] = &[
    ("_", 1),
    ("mod", 2),
];
pub const DEF_SYMBOLS: &[(char, u32)] = &[
    ('!', 1),
    ('#', 2),
    (',', 3),
    (':', 4),
    (';', 5),
    ('<', 6),
    ('=', 7),
    ('>', 8),
    ('@', 9),
    ('^', 10),
    ('|', 11),
];
pub type ReductionType = Reduction<for<'a> fn(&mut Vec<Box<dyn Node<'a, ()> + 'a>>, &mut Vec<Box<(Token<'a, ()>, bool)>>) -> Box<dyn Node<'a, ()> + 'a>>;
pub const DEF_STATE_0_NODE_JUMPS: &[NodeJump] = &[
    NodeJump::new(3, 5),
    NodeJump::new(10, 4),
    NodeJump::new(11, 1),
    NodeJump::new(12, 2),
    NodeJump::new(16, 3),
];
pub const DEF_STATE_0_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(7)),
    TokenJump::new(2, Some(2), None, Some(9)),
    TokenJump::new(3, Some(2), None, Some(8)),
    TokenJump::new(3, Some(9), None, Some(6)),
];
pub const DEF_STATE_1_NODE_JUMPS: &[NodeJump] = &[
    NodeJump::new(3, 5),
    NodeJump::new(10, 4),
    NodeJump::new(12, 10),
    NodeJump::new(16, 3),
];
pub const DEF_STATE_1_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(7)),
    TokenJump::new(2, Some(2), None, Some(9)),
    TokenJump::new(3, Some(2), None, Some(8)),
    TokenJump::new(3, Some(9), None, Some(6)),
];
pub const DEF_STATE_1_REDUCTION: ReductionType = Reduction::new(0, 1, |_, _| unreachable!()
);
pub const DEF_STATE_2_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_2_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_2_REDUCTION: ReductionType = Reduction::new(11, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::script::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_3_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_3_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_3_REDUCTION: ReductionType = Reduction::new(12, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::script_item::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_4_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_4_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_4_REDUCTION: ReductionType = Reduction::new(12, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::script_item::p1(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_5_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_5_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_5_REDUCTION: ReductionType = Reduction::new(12, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::script_item::p2(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_6_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_6_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(11)),
];
pub const DEF_STATE_7_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_7_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(3, Some(4), None, Some(12)),
];
pub const DEF_STATE_8_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_8_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(13)),
];
pub const DEF_STATE_9_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_9_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(14)),
];
pub const DEF_STATE_10_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_10_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_10_REDUCTION: ReductionType = Reduction::new(11, 2, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 2 ..);
    Box::new(nodes::script::p1(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_11_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_11_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(3, Some(7), None, Some(15)),
];
pub const DEF_STATE_12_NODE_JUMPS: &[NodeJump] = &[
    NodeJump::new(4, 19),
    NodeJump::new(13, 18),
    NodeJump::new(14, 16),
    NodeJump::new(15, 17),
];
pub const DEF_STATE_12_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(20)),
    TokenJump::new(3, Some(9), None, Some(23)),
    TokenJump::new(101, None, None, Some(21)),
    TokenJump::new(102, None, None, Some(22)),
];
pub const DEF_STATE_13_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_13_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(3, Some(4), None, Some(24)),
];
pub const DEF_STATE_14_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_14_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(3, Some(5), None, Some(25)),
];
pub const DEF_STATE_15_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_15_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(5, None, None, Some(26)),
];
pub const DEF_STATE_16_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_16_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(3, Some(5), None, Some(27)),
    TokenJump::new(3, Some(11), None, Some(28)),
];
pub const DEF_STATE_17_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_17_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_17_REDUCTION: ReductionType = Reduction::new(14, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::statement_list::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_18_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_18_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(3, Some(10), None, Some(29)),
];
pub const DEF_STATE_18_REDUCTION: ReductionType = Reduction::new(15, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::statement_with_cond::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_19_NODE_JUMPS: &[NodeJump] = &[
    NodeJump::new(1, 32),
    NodeJump::new(2, 31),
    NodeJump::new(4, 33),
];
pub const DEF_STATE_19_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(20)),
    TokenJump::new(2, Some(1), None, Some(34)),
    TokenJump::new(3, Some(1), None, Some(30)),
    TokenJump::new(3, Some(9), None, Some(23)),
    TokenJump::new(101, None, None, Some(21)),
    TokenJump::new(102, None, None, Some(22)),
];
pub const DEF_STATE_19_REDUCTION: ReductionType = Reduction::new(13, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::statement::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_20_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_20_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(3, Some(6), None, Some(35)),
];
pub const DEF_STATE_20_REDUCTION: ReductionType = Reduction::new(4, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::item::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_21_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_21_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_21_REDUCTION: ReductionType = Reduction::new(4, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::item::p1(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_22_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_22_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_22_REDUCTION: ReductionType = Reduction::new(4, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::item::p2(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_23_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_23_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(36)),
];
pub const DEF_STATE_24_NODE_JUMPS: &[NodeJump] = &[
    NodeJump::new(4, 19),
    NodeJump::new(13, 18),
    NodeJump::new(14, 37),
    NodeJump::new(15, 17),
];
pub const DEF_STATE_24_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(20)),
    TokenJump::new(3, Some(9), None, Some(23)),
    TokenJump::new(101, None, None, Some(21)),
    TokenJump::new(102, None, None, Some(22)),
];
pub const DEF_STATE_25_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_25_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_25_REDUCTION: ReductionType = Reduction::new(3, 3, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 3 ..);
    Box::new(nodes::import::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_26_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_26_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(3, Some(5), None, Some(38)),
];
pub const DEF_STATE_27_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_27_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_27_REDUCTION: ReductionType = Reduction::new(10, 4, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 4 ..);
    Box::new(nodes::production::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_28_NODE_JUMPS: &[NodeJump] = &[
    NodeJump::new(4, 19),
    NodeJump::new(13, 18),
    NodeJump::new(15, 39),
];
pub const DEF_STATE_28_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(20)),
    TokenJump::new(3, Some(9), None, Some(23)),
    TokenJump::new(101, None, None, Some(21)),
    TokenJump::new(102, None, None, Some(22)),
];
pub const DEF_STATE_29_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_29_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(40)),
    TokenJump::new(3, Some(1), None, Some(41)),
];
pub const DEF_STATE_30_NODE_JUMPS: &[NodeJump] = &[
    NodeJump::new(8, 43),
    NodeJump::new(9, 42),
];
pub const DEF_STATE_30_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(2, Some(1), None, Some(47)),
    TokenJump::new(3, Some(9), None, Some(46)),
    TokenJump::new(101, None, None, Some(44)),
    TokenJump::new(102, None, None, Some(45)),
];
pub const DEF_STATE_31_NODE_JUMPS: &[NodeJump] = &[
    NodeJump::new(1, 49),
    NodeJump::new(4, 33),
];
pub const DEF_STATE_31_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(20)),
    TokenJump::new(2, Some(1), None, Some(34)),
    TokenJump::new(3, Some(1), None, Some(48)),
    TokenJump::new(3, Some(9), None, Some(23)),
    TokenJump::new(101, None, None, Some(21)),
    TokenJump::new(102, None, None, Some(22)),
];
pub const DEF_STATE_31_REDUCTION: ReductionType = Reduction::new(13, 2, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 2 ..);
    Box::new(nodes::statement::p2(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_32_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_32_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_32_REDUCTION: ReductionType = Reduction::new(2, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::followed_item_list::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_33_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_33_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_33_REDUCTION: ReductionType = Reduction::new(1, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::followed_item::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_34_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_34_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(3, Some(9), None, Some(52)),
    TokenJump::new(101, None, None, Some(50)),
    TokenJump::new(102, None, None, Some(51)),
];
pub const DEF_STATE_35_NODE_JUMPS: &[NodeJump] = &[
    NodeJump::new(7, 54),
];
pub const DEF_STATE_35_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(56)),
    TokenJump::new(3, Some(8), None, Some(53)),
    TokenJump::new(3, Some(9), None, Some(55)),
];
pub const DEF_STATE_36_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_36_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_36_REDUCTION: ReductionType = Reduction::new(4, 2, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 2 ..);
    Box::new(nodes::item::p3(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_37_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_37_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(3, Some(5), None, Some(57)),
    TokenJump::new(3, Some(11), None, Some(28)),
];
pub const DEF_STATE_38_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_38_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_38_REDUCTION: ReductionType = Reduction::new(16, 5, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 5 ..);
    Box::new(nodes::token::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_39_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_39_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_39_REDUCTION: ReductionType = Reduction::new(14, 3, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 3 ..);
    Box::new(nodes::statement_list::p1(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_40_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_40_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_40_REDUCTION: ReductionType = Reduction::new(15, 3, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 3 ..);
    Box::new(nodes::statement_with_cond::p1(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_41_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_41_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(58)),
];
pub const DEF_STATE_42_NODE_JUMPS: &[NodeJump] = &[
    NodeJump::new(8, 59),
];
pub const DEF_STATE_42_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(2, Some(1), None, Some(47)),
    TokenJump::new(3, Some(9), None, Some(46)),
    TokenJump::new(101, None, None, Some(44)),
    TokenJump::new(102, None, None, Some(45)),
];
pub const DEF_STATE_42_REDUCTION: ReductionType = Reduction::new(13, 3, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 3 ..);
    Box::new(nodes::statement::p1(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_43_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_43_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_43_REDUCTION: ReductionType = Reduction::new(9, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::not_followed_item_list::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_44_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_44_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_44_REDUCTION: ReductionType = Reduction::new(8, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::not_followed_item::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_45_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_45_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_45_REDUCTION: ReductionType = Reduction::new(8, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::not_followed_item::p1(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_46_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_46_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(60)),
];
pub const DEF_STATE_47_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_47_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(3, Some(9), None, Some(63)),
    TokenJump::new(101, None, None, Some(61)),
    TokenJump::new(102, None, None, Some(62)),
];
pub const DEF_STATE_48_NODE_JUMPS: &[NodeJump] = &[
    NodeJump::new(8, 43),
    NodeJump::new(9, 64),
];
pub const DEF_STATE_48_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(2, Some(1), None, Some(47)),
    TokenJump::new(3, Some(9), None, Some(46)),
    TokenJump::new(101, None, None, Some(44)),
    TokenJump::new(102, None, None, Some(45)),
];
pub const DEF_STATE_49_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_49_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_49_REDUCTION: ReductionType = Reduction::new(2, 2, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 2 ..);
    Box::new(nodes::followed_item_list::p1(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_50_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_50_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_50_REDUCTION: ReductionType = Reduction::new(1, 2, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 2 ..);
    Box::new(nodes::followed_item::p1(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_51_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_51_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_51_REDUCTION: ReductionType = Reduction::new(1, 2, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 2 ..);
    Box::new(nodes::followed_item::p2(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_52_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_52_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(65)),
];
pub const DEF_STATE_53_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_53_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_53_REDUCTION: ReductionType = Reduction::new(4, 3, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 3 ..);
    Box::new(nodes::item::p4(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_54_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_54_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(3, Some(3), None, Some(67)),
    TokenJump::new(3, Some(8), None, Some(66)),
];
pub const DEF_STATE_55_NODE_JUMPS: &[NodeJump] = &[
    NodeJump::new(5, 69),
    NodeJump::new(6, 68),
];
pub const DEF_STATE_55_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(70)),
    TokenJump::new(3, Some(1), None, Some(71)),
];
pub const DEF_STATE_56_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_56_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_56_REDUCTION: ReductionType = Reduction::new(7, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::node_param_list::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_57_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_57_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_57_REDUCTION: ReductionType = Reduction::new(10, 5, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 5 ..);
    Box::new(nodes::production::p1(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_58_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_58_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_58_REDUCTION: ReductionType = Reduction::new(15, 4, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 4 ..);
    Box::new(nodes::statement_with_cond::p2(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_59_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_59_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_59_REDUCTION: ReductionType = Reduction::new(9, 2, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 2 ..);
    Box::new(nodes::not_followed_item_list::p1(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_60_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_60_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_60_REDUCTION: ReductionType = Reduction::new(8, 2, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 2 ..);
    Box::new(nodes::not_followed_item::p2(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_61_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_61_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_61_REDUCTION: ReductionType = Reduction::new(8, 2, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 2 ..);
    Box::new(nodes::not_followed_item::p3(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_62_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_62_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_62_REDUCTION: ReductionType = Reduction::new(8, 2, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 2 ..);
    Box::new(nodes::not_followed_item::p4(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_63_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_63_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(72)),
];
pub const DEF_STATE_64_NODE_JUMPS: &[NodeJump] = &[
    NodeJump::new(8, 59),
];
pub const DEF_STATE_64_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(2, Some(1), None, Some(47)),
    TokenJump::new(3, Some(9), None, Some(46)),
    TokenJump::new(101, None, None, Some(44)),
    TokenJump::new(102, None, None, Some(45)),
];
pub const DEF_STATE_64_REDUCTION: ReductionType = Reduction::new(13, 4, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 4 ..);
    Box::new(nodes::statement::p3(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_65_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_65_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_65_REDUCTION: ReductionType = Reduction::new(1, 3, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 3 ..);
    Box::new(nodes::followed_item::p3(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_66_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_66_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_66_REDUCTION: ReductionType = Reduction::new(4, 4, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 4 ..);
    Box::new(nodes::item::p5(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_67_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_67_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(74)),
    TokenJump::new(3, Some(8), None, Some(73)),
];
pub const DEF_STATE_68_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_68_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(3, Some(3), None, Some(76)),
    TokenJump::new(3, Some(8), None, Some(75)),
];
pub const DEF_STATE_69_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_69_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_69_REDUCTION: ReductionType = Reduction::new(6, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::node_modifier_param_list::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_70_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_70_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_70_REDUCTION: ReductionType = Reduction::new(5, 1, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 1 ..);
    Box::new(nodes::node_modifier_param::p0(
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_71_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_71_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(77)),
];
pub const DEF_STATE_72_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_72_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_72_REDUCTION: ReductionType = Reduction::new(8, 3, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 3 ..);
    Box::new(nodes::not_followed_item::p5(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_73_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_73_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_73_REDUCTION: ReductionType = Reduction::new(4, 5, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 5 ..);
    Box::new(nodes::item::p6(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_74_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_74_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_74_REDUCTION: ReductionType = Reduction::new(7, 3, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 3 ..);
    Box::new(nodes::node_param_list::p1(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_75_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_75_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_75_REDUCTION: ReductionType = Reduction::new(4, 5, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 5 ..);
    Box::new(nodes::item::p7(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_76_NODE_JUMPS: &[NodeJump] = &[
    NodeJump::new(5, 79),
];
pub const DEF_STATE_76_TOKEN_JUMPS: &[TokenJump] = &[
    TokenJump::new(1, None, None, Some(70)),
    TokenJump::new(3, Some(1), None, Some(71)),
    TokenJump::new(3, Some(8), None, Some(78)),
];
pub const DEF_STATE_77_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_77_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_77_REDUCTION: ReductionType = Reduction::new(5, 2, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 2 ..);
    Box::new(nodes::node_modifier_param::p1(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_78_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_78_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_78_REDUCTION: ReductionType = Reduction::new(4, 6, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 6 ..);
    Box::new(nodes::item::p8(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub const DEF_STATE_79_NODE_JUMPS: &[NodeJump] = &[];
pub const DEF_STATE_79_TOKEN_JUMPS: &[TokenJump] = &[];
pub const DEF_STATE_79_REDUCTION: ReductionType = Reduction::new(6, 3, |nodes, _| unsafe {
    let mut iter = nodes.drain(nodes.len() - 3 ..);
    Box::new(nodes::node_modifier_param_list::p1(
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
        iter.next().unwrap_unchecked().downcast_unchecked(),
    ))
});
pub type StateType<'r> = State<'r, ReductionType>;
pub const DEF_STATES: &[StateType] = &[
    State::new(DEF_STATE_0_NODE_JUMPS, DEF_STATE_0_TOKEN_JUMPS, None),
    State::new(DEF_STATE_1_NODE_JUMPS, DEF_STATE_1_TOKEN_JUMPS, Some(&DEF_STATE_1_REDUCTION)),
    State::new(DEF_STATE_2_NODE_JUMPS, DEF_STATE_2_TOKEN_JUMPS, Some(&DEF_STATE_2_REDUCTION)),
    State::new(DEF_STATE_3_NODE_JUMPS, DEF_STATE_3_TOKEN_JUMPS, Some(&DEF_STATE_3_REDUCTION)),
    State::new(DEF_STATE_4_NODE_JUMPS, DEF_STATE_4_TOKEN_JUMPS, Some(&DEF_STATE_4_REDUCTION)),
    State::new(DEF_STATE_5_NODE_JUMPS, DEF_STATE_5_TOKEN_JUMPS, Some(&DEF_STATE_5_REDUCTION)),
    State::new(DEF_STATE_6_NODE_JUMPS, DEF_STATE_6_TOKEN_JUMPS, None),
    State::new(DEF_STATE_7_NODE_JUMPS, DEF_STATE_7_TOKEN_JUMPS, None),
    State::new(DEF_STATE_8_NODE_JUMPS, DEF_STATE_8_TOKEN_JUMPS, None),
    State::new(DEF_STATE_9_NODE_JUMPS, DEF_STATE_9_TOKEN_JUMPS, None),
    State::new(DEF_STATE_10_NODE_JUMPS, DEF_STATE_10_TOKEN_JUMPS, Some(&DEF_STATE_10_REDUCTION)),
    State::new(DEF_STATE_11_NODE_JUMPS, DEF_STATE_11_TOKEN_JUMPS, None),
    State::new(DEF_STATE_12_NODE_JUMPS, DEF_STATE_12_TOKEN_JUMPS, None),
    State::new(DEF_STATE_13_NODE_JUMPS, DEF_STATE_13_TOKEN_JUMPS, None),
    State::new(DEF_STATE_14_NODE_JUMPS, DEF_STATE_14_TOKEN_JUMPS, None),
    State::new(DEF_STATE_15_NODE_JUMPS, DEF_STATE_15_TOKEN_JUMPS, None),
    State::new(DEF_STATE_16_NODE_JUMPS, DEF_STATE_16_TOKEN_JUMPS, None),
    State::new(DEF_STATE_17_NODE_JUMPS, DEF_STATE_17_TOKEN_JUMPS, Some(&DEF_STATE_17_REDUCTION)),
    State::new(DEF_STATE_18_NODE_JUMPS, DEF_STATE_18_TOKEN_JUMPS, Some(&DEF_STATE_18_REDUCTION)),
    State::new(DEF_STATE_19_NODE_JUMPS, DEF_STATE_19_TOKEN_JUMPS, Some(&DEF_STATE_19_REDUCTION)),
    State::new(DEF_STATE_20_NODE_JUMPS, DEF_STATE_20_TOKEN_JUMPS, Some(&DEF_STATE_20_REDUCTION)),
    State::new(DEF_STATE_21_NODE_JUMPS, DEF_STATE_21_TOKEN_JUMPS, Some(&DEF_STATE_21_REDUCTION)),
    State::new(DEF_STATE_22_NODE_JUMPS, DEF_STATE_22_TOKEN_JUMPS, Some(&DEF_STATE_22_REDUCTION)),
    State::new(DEF_STATE_23_NODE_JUMPS, DEF_STATE_23_TOKEN_JUMPS, None),
    State::new(DEF_STATE_24_NODE_JUMPS, DEF_STATE_24_TOKEN_JUMPS, None),
    State::new(DEF_STATE_25_NODE_JUMPS, DEF_STATE_25_TOKEN_JUMPS, Some(&DEF_STATE_25_REDUCTION)),
    State::new(DEF_STATE_26_NODE_JUMPS, DEF_STATE_26_TOKEN_JUMPS, None),
    State::new(DEF_STATE_27_NODE_JUMPS, DEF_STATE_27_TOKEN_JUMPS, Some(&DEF_STATE_27_REDUCTION)),
    State::new(DEF_STATE_28_NODE_JUMPS, DEF_STATE_28_TOKEN_JUMPS, None),
    State::new(DEF_STATE_29_NODE_JUMPS, DEF_STATE_29_TOKEN_JUMPS, None),
    State::new(DEF_STATE_30_NODE_JUMPS, DEF_STATE_30_TOKEN_JUMPS, None),
    State::new(DEF_STATE_31_NODE_JUMPS, DEF_STATE_31_TOKEN_JUMPS, Some(&DEF_STATE_31_REDUCTION)),
    State::new(DEF_STATE_32_NODE_JUMPS, DEF_STATE_32_TOKEN_JUMPS, Some(&DEF_STATE_32_REDUCTION)),
    State::new(DEF_STATE_33_NODE_JUMPS, DEF_STATE_33_TOKEN_JUMPS, Some(&DEF_STATE_33_REDUCTION)),
    State::new(DEF_STATE_34_NODE_JUMPS, DEF_STATE_34_TOKEN_JUMPS, None),
    State::new(DEF_STATE_35_NODE_JUMPS, DEF_STATE_35_TOKEN_JUMPS, None),
    State::new(DEF_STATE_36_NODE_JUMPS, DEF_STATE_36_TOKEN_JUMPS, Some(&DEF_STATE_36_REDUCTION)),
    State::new(DEF_STATE_37_NODE_JUMPS, DEF_STATE_37_TOKEN_JUMPS, None),
    State::new(DEF_STATE_38_NODE_JUMPS, DEF_STATE_38_TOKEN_JUMPS, Some(&DEF_STATE_38_REDUCTION)),
    State::new(DEF_STATE_39_NODE_JUMPS, DEF_STATE_39_TOKEN_JUMPS, Some(&DEF_STATE_39_REDUCTION)),
    State::new(DEF_STATE_40_NODE_JUMPS, DEF_STATE_40_TOKEN_JUMPS, Some(&DEF_STATE_40_REDUCTION)),
    State::new(DEF_STATE_41_NODE_JUMPS, DEF_STATE_41_TOKEN_JUMPS, None),
    State::new(DEF_STATE_42_NODE_JUMPS, DEF_STATE_42_TOKEN_JUMPS, Some(&DEF_STATE_42_REDUCTION)),
    State::new(DEF_STATE_43_NODE_JUMPS, DEF_STATE_43_TOKEN_JUMPS, Some(&DEF_STATE_43_REDUCTION)),
    State::new(DEF_STATE_44_NODE_JUMPS, DEF_STATE_44_TOKEN_JUMPS, Some(&DEF_STATE_44_REDUCTION)),
    State::new(DEF_STATE_45_NODE_JUMPS, DEF_STATE_45_TOKEN_JUMPS, Some(&DEF_STATE_45_REDUCTION)),
    State::new(DEF_STATE_46_NODE_JUMPS, DEF_STATE_46_TOKEN_JUMPS, None),
    State::new(DEF_STATE_47_NODE_JUMPS, DEF_STATE_47_TOKEN_JUMPS, None),
    State::new(DEF_STATE_48_NODE_JUMPS, DEF_STATE_48_TOKEN_JUMPS, None),
    State::new(DEF_STATE_49_NODE_JUMPS, DEF_STATE_49_TOKEN_JUMPS, Some(&DEF_STATE_49_REDUCTION)),
    State::new(DEF_STATE_50_NODE_JUMPS, DEF_STATE_50_TOKEN_JUMPS, Some(&DEF_STATE_50_REDUCTION)),
    State::new(DEF_STATE_51_NODE_JUMPS, DEF_STATE_51_TOKEN_JUMPS, Some(&DEF_STATE_51_REDUCTION)),
    State::new(DEF_STATE_52_NODE_JUMPS, DEF_STATE_52_TOKEN_JUMPS, None),
    State::new(DEF_STATE_53_NODE_JUMPS, DEF_STATE_53_TOKEN_JUMPS, Some(&DEF_STATE_53_REDUCTION)),
    State::new(DEF_STATE_54_NODE_JUMPS, DEF_STATE_54_TOKEN_JUMPS, None),
    State::new(DEF_STATE_55_NODE_JUMPS, DEF_STATE_55_TOKEN_JUMPS, None),
    State::new(DEF_STATE_56_NODE_JUMPS, DEF_STATE_56_TOKEN_JUMPS, Some(&DEF_STATE_56_REDUCTION)),
    State::new(DEF_STATE_57_NODE_JUMPS, DEF_STATE_57_TOKEN_JUMPS, Some(&DEF_STATE_57_REDUCTION)),
    State::new(DEF_STATE_58_NODE_JUMPS, DEF_STATE_58_TOKEN_JUMPS, Some(&DEF_STATE_58_REDUCTION)),
    State::new(DEF_STATE_59_NODE_JUMPS, DEF_STATE_59_TOKEN_JUMPS, Some(&DEF_STATE_59_REDUCTION)),
    State::new(DEF_STATE_60_NODE_JUMPS, DEF_STATE_60_TOKEN_JUMPS, Some(&DEF_STATE_60_REDUCTION)),
    State::new(DEF_STATE_61_NODE_JUMPS, DEF_STATE_61_TOKEN_JUMPS, Some(&DEF_STATE_61_REDUCTION)),
    State::new(DEF_STATE_62_NODE_JUMPS, DEF_STATE_62_TOKEN_JUMPS, Some(&DEF_STATE_62_REDUCTION)),
    State::new(DEF_STATE_63_NODE_JUMPS, DEF_STATE_63_TOKEN_JUMPS, None),
    State::new(DEF_STATE_64_NODE_JUMPS, DEF_STATE_64_TOKEN_JUMPS, Some(&DEF_STATE_64_REDUCTION)),
    State::new(DEF_STATE_65_NODE_JUMPS, DEF_STATE_65_TOKEN_JUMPS, Some(&DEF_STATE_65_REDUCTION)),
    State::new(DEF_STATE_66_NODE_JUMPS, DEF_STATE_66_TOKEN_JUMPS, Some(&DEF_STATE_66_REDUCTION)),
    State::new(DEF_STATE_67_NODE_JUMPS, DEF_STATE_67_TOKEN_JUMPS, None),
    State::new(DEF_STATE_68_NODE_JUMPS, DEF_STATE_68_TOKEN_JUMPS, None),
    State::new(DEF_STATE_69_NODE_JUMPS, DEF_STATE_69_TOKEN_JUMPS, Some(&DEF_STATE_69_REDUCTION)),
    State::new(DEF_STATE_70_NODE_JUMPS, DEF_STATE_70_TOKEN_JUMPS, Some(&DEF_STATE_70_REDUCTION)),
    State::new(DEF_STATE_71_NODE_JUMPS, DEF_STATE_71_TOKEN_JUMPS, None),
    State::new(DEF_STATE_72_NODE_JUMPS, DEF_STATE_72_TOKEN_JUMPS, Some(&DEF_STATE_72_REDUCTION)),
    State::new(DEF_STATE_73_NODE_JUMPS, DEF_STATE_73_TOKEN_JUMPS, Some(&DEF_STATE_73_REDUCTION)),
    State::new(DEF_STATE_74_NODE_JUMPS, DEF_STATE_74_TOKEN_JUMPS, Some(&DEF_STATE_74_REDUCTION)),
    State::new(DEF_STATE_75_NODE_JUMPS, DEF_STATE_75_TOKEN_JUMPS, Some(&DEF_STATE_75_REDUCTION)),
    State::new(DEF_STATE_76_NODE_JUMPS, DEF_STATE_76_TOKEN_JUMPS, None),
    State::new(DEF_STATE_77_NODE_JUMPS, DEF_STATE_77_TOKEN_JUMPS, Some(&DEF_STATE_77_REDUCTION)),
    State::new(DEF_STATE_78_NODE_JUMPS, DEF_STATE_78_TOKEN_JUMPS, Some(&DEF_STATE_78_REDUCTION)),
    State::new(DEF_STATE_79_NODE_JUMPS, DEF_STATE_79_TOKEN_JUMPS, Some(&DEF_STATE_79_REDUCTION)),
];
#[allow(non_camel_case_types)]
pub mod nodes {
    pub enum followed_item<'a> {
        /// followed_item -> item
        p0(Box<item<'a>>),
        /// followed_item -> '_' @keyword_literal
        p1(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
        /// followed_item -> '_' @symbol_literal
        p2(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
        /// followed_item -> '_' '@' @ident
        p3(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
    }
    impl<'a> super::Node<'a, ()> for followed_item<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0) => {
                    destructor.non_token(i0)?;
                }
                Self::p1(i0, i1) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                }
                Self::p2(i0, i1) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                }
                Self::p3(i0, i1, i2) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                    destructor.token(i2)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for followed_item<'a> {
        fn name(&self) -> &str { "followed_item" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                Self::p0(t) => Ok(t),
                _ => Err(self)
            }
        }
    }
    pub enum followed_item_list<'a> {
        /// followed_item_list -> followed_item
        p0(Box<followed_item<'a>>),
        /// followed_item_list -> followed_item_list followed_item
        p1(Box<followed_item_list<'a>>, Box<followed_item<'a>>),
    }
    impl<'a> super::Node<'a, ()> for followed_item_list<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0) => {
                    destructor.non_token(i0)?;
                }
                Self::p1(i0, i1) => {
                    destructor.non_token(i0)?;
                    destructor.non_token(i1)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for followed_item_list<'a> {
        fn name(&self) -> &str { "followed_item_list" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                Self::p0(t) => Ok(t),
                _ => Err(self)
            }
        }
    }
    pub enum import<'a> {
        /// import -> 'mod' @ident ';'
        p0(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
    }
    impl<'a> super::Node<'a, ()> for import<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0, i1, i2) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                    destructor.token(i2)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for import<'a> {
        fn name(&self) -> &str { "import" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                _ => Err(self)
            }
        }
    }
    pub enum item<'a> {
        /// item -> @ident
        p0(Box<(super::Token<'a, ()>, bool)>),
        /// item -> @keyword_literal
        p1(Box<(super::Token<'a, ()>, bool)>),
        /// item -> @symbol_literal
        p2(Box<(super::Token<'a, ()>, bool)>),
        /// item -> '@' @ident
        p3(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
        /// item -> @ident '<' '>'
        p4(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
        /// item -> @ident '<' node_param_list '>'
        p5(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<node_param_list<'a>>, Box<(super::Token<'a, ()>, bool)>),
        /// item -> @ident '<' node_param_list ',' '>'
        p6(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<node_param_list<'a>>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
        /// item -> @ident '<' '@' node_modifier_param_list '>'
        p7(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<node_modifier_param_list<'a>>, Box<(super::Token<'a, ()>, bool)>),
        /// item -> @ident '<' '@' node_modifier_param_list ',' '>'
        p8(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<node_modifier_param_list<'a>>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
    }
    impl<'a> super::Node<'a, ()> for item<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0) => {
                    destructor.token(i0)?;
                }
                Self::p1(i0) => {
                    destructor.token(i0)?;
                }
                Self::p2(i0) => {
                    destructor.token(i0)?;
                }
                Self::p3(i0, i1) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                }
                Self::p4(i0, i1, i2) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                    destructor.token(i2)?;
                }
                Self::p5(i0, i1, i2, i3) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                    destructor.non_token(i2)?;
                    destructor.token(i3)?;
                }
                Self::p6(i0, i1, i2, i3, i4) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                    destructor.non_token(i2)?;
                    destructor.token(i3)?;
                    destructor.token(i4)?;
                }
                Self::p7(i0, i1, i2, i3, i4) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                    destructor.token(i2)?;
                    destructor.non_token(i3)?;
                    destructor.token(i4)?;
                }
                Self::p8(i0, i1, i2, i3, i4, i5) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                    destructor.token(i2)?;
                    destructor.non_token(i3)?;
                    destructor.token(i4)?;
                    destructor.token(i5)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for item<'a> {
        fn name(&self) -> &str { "item" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                Self::p0(t) => Ok(t),
                Self::p1(t) => Ok(t),
                Self::p2(t) => Ok(t),
                _ => Err(self)
            }
        }
    }
    pub enum node_modifier_param<'a> {
        /// node_modifier_param -> @ident
        p0(Box<(super::Token<'a, ()>, bool)>),
        /// node_modifier_param -> '!' @ident
        p1(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
    }
    impl<'a> super::Node<'a, ()> for node_modifier_param<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0) => {
                    destructor.token(i0)?;
                }
                Self::p1(i0, i1) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for node_modifier_param<'a> {
        fn name(&self) -> &str { "node_modifier_param" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                Self::p0(t) => Ok(t),
                _ => Err(self)
            }
        }
    }
    pub enum node_modifier_param_list<'a> {
        /// node_modifier_param_list -> node_modifier_param
        p0(Box<node_modifier_param<'a>>),
        /// node_modifier_param_list -> node_modifier_param_list ',' node_modifier_param
        p1(Box<node_modifier_param_list<'a>>, Box<(super::Token<'a, ()>, bool)>, Box<node_modifier_param<'a>>),
    }
    impl<'a> super::Node<'a, ()> for node_modifier_param_list<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0) => {
                    destructor.non_token(i0)?;
                }
                Self::p1(i0, i1, i2) => {
                    destructor.non_token(i0)?;
                    destructor.token(i1)?;
                    destructor.non_token(i2)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for node_modifier_param_list<'a> {
        fn name(&self) -> &str { "node_modifier_param_list" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                Self::p0(t) => Ok(t),
                _ => Err(self)
            }
        }
    }
    pub enum node_param_list<'a> {
        /// node_param_list -> @ident
        p0(Box<(super::Token<'a, ()>, bool)>),
        /// node_param_list -> node_param_list ',' @ident
        p1(Box<node_param_list<'a>>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
    }
    impl<'a> super::Node<'a, ()> for node_param_list<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0) => {
                    destructor.token(i0)?;
                }
                Self::p1(i0, i1, i2) => {
                    destructor.non_token(i0)?;
                    destructor.token(i1)?;
                    destructor.token(i2)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for node_param_list<'a> {
        fn name(&self) -> &str { "node_param_list" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                Self::p0(t) => Ok(t),
                _ => Err(self)
            }
        }
    }
    pub enum not_followed_item<'a> {
        /// not_followed_item -> @keyword_literal
        p0(Box<(super::Token<'a, ()>, bool)>),
        /// not_followed_item -> @symbol_literal
        p1(Box<(super::Token<'a, ()>, bool)>),
        /// not_followed_item -> '@' @ident
        p2(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
        /// not_followed_item -> '_' @keyword_literal
        p3(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
        /// not_followed_item -> '_' @symbol_literal
        p4(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
        /// not_followed_item -> '_' '@' @ident
        p5(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
    }
    impl<'a> super::Node<'a, ()> for not_followed_item<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0) => {
                    destructor.token(i0)?;
                }
                Self::p1(i0) => {
                    destructor.token(i0)?;
                }
                Self::p2(i0, i1) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                }
                Self::p3(i0, i1) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                }
                Self::p4(i0, i1) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                }
                Self::p5(i0, i1, i2) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                    destructor.token(i2)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for not_followed_item<'a> {
        fn name(&self) -> &str { "not_followed_item" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                Self::p0(t) => Ok(t),
                Self::p1(t) => Ok(t),
                _ => Err(self)
            }
        }
    }
    pub enum not_followed_item_list<'a> {
        /// not_followed_item_list -> not_followed_item
        p0(Box<not_followed_item<'a>>),
        /// not_followed_item_list -> not_followed_item_list not_followed_item
        p1(Box<not_followed_item_list<'a>>, Box<not_followed_item<'a>>),
    }
    impl<'a> super::Node<'a, ()> for not_followed_item_list<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0) => {
                    destructor.non_token(i0)?;
                }
                Self::p1(i0, i1) => {
                    destructor.non_token(i0)?;
                    destructor.non_token(i1)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for not_followed_item_list<'a> {
        fn name(&self) -> &str { "not_followed_item_list" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                Self::p0(t) => Ok(t),
                _ => Err(self)
            }
        }
    }
    pub enum production<'a> {
        /// production -> @ident ':' statement_list ';'
        p0(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<statement_list<'a>>, Box<(super::Token<'a, ()>, bool)>),
        /// production -> '#' @ident ':' statement_list ';'
        p1(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<statement_list<'a>>, Box<(super::Token<'a, ()>, bool)>),
    }
    impl<'a> super::Node<'a, ()> for production<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0, i1, i2, i3) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                    destructor.non_token(i2)?;
                    destructor.token(i3)?;
                }
                Self::p1(i0, i1, i2, i3, i4) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                    destructor.token(i2)?;
                    destructor.non_token(i3)?;
                    destructor.token(i4)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for production<'a> {
        fn name(&self) -> &str { "production" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                _ => Err(self)
            }
        }
    }
    pub enum script<'a> {
        /// script -> script_item
        p0(Box<script_item<'a>>),
        /// script -> script script_item
        p1(Box<script<'a>>, Box<script_item<'a>>),
    }
    impl<'a> super::Node<'a, ()> for script<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0) => {
                    destructor.non_token(i0)?;
                }
                Self::p1(i0, i1) => {
                    destructor.non_token(i0)?;
                    destructor.non_token(i1)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for script<'a> {
        fn name(&self) -> &str { "script" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                Self::p0(t) => Ok(t),
                _ => Err(self)
            }
        }
    }
    pub enum script_item<'a> {
        /// script_item -> token
        p0(Box<token<'a>>),
        /// script_item -> production
        p1(Box<production<'a>>),
        /// script_item -> import
        p2(Box<import<'a>>),
    }
    impl<'a> super::Node<'a, ()> for script_item<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0) => {
                    destructor.non_token(i0)?;
                }
                Self::p1(i0) => {
                    destructor.non_token(i0)?;
                }
                Self::p2(i0) => {
                    destructor.non_token(i0)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for script_item<'a> {
        fn name(&self) -> &str { "script_item" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                Self::p0(t) => Ok(t),
                Self::p1(t) => Ok(t),
                Self::p2(t) => Ok(t),
            }
        }
    }
    pub enum statement<'a> {
        /// statement -> item
        p0(Box<item<'a>>),
        /// statement -> item '!' not_followed_item_list
        p1(Box<item<'a>>, Box<(super::Token<'a, ()>, bool)>, Box<not_followed_item_list<'a>>),
        /// statement -> item followed_item_list
        p2(Box<item<'a>>, Box<followed_item_list<'a>>),
        /// statement -> item followed_item_list '!' not_followed_item_list
        p3(Box<item<'a>>, Box<followed_item_list<'a>>, Box<(super::Token<'a, ()>, bool)>, Box<not_followed_item_list<'a>>),
    }
    impl<'a> super::Node<'a, ()> for statement<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0) => {
                    destructor.non_token(i0)?;
                }
                Self::p1(i0, i1, i2) => {
                    destructor.non_token(i0)?;
                    destructor.token(i1)?;
                    destructor.non_token(i2)?;
                }
                Self::p2(i0, i1) => {
                    destructor.non_token(i0)?;
                    destructor.non_token(i1)?;
                }
                Self::p3(i0, i1, i2, i3) => {
                    destructor.non_token(i0)?;
                    destructor.non_token(i1)?;
                    destructor.token(i2)?;
                    destructor.non_token(i3)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for statement<'a> {
        fn name(&self) -> &str { "statement" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                Self::p0(t) => Ok(t),
                _ => Err(self)
            }
        }
    }
    pub enum statement_list<'a> {
        /// statement_list -> statement_with_cond
        p0(Box<statement_with_cond<'a>>),
        /// statement_list -> statement_list '|' statement_with_cond
        p1(Box<statement_list<'a>>, Box<(super::Token<'a, ()>, bool)>, Box<statement_with_cond<'a>>),
    }
    impl<'a> super::Node<'a, ()> for statement_list<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0) => {
                    destructor.non_token(i0)?;
                }
                Self::p1(i0, i1, i2) => {
                    destructor.non_token(i0)?;
                    destructor.token(i1)?;
                    destructor.non_token(i2)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for statement_list<'a> {
        fn name(&self) -> &str { "statement_list" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                Self::p0(t) => Ok(t),
                _ => Err(self)
            }
        }
    }
    pub enum statement_with_cond<'a> {
        /// statement_with_cond -> statement
        p0(Box<statement<'a>>),
        /// statement_with_cond -> statement '^' @ident
        p1(Box<statement<'a>>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
        /// statement_with_cond -> statement '^' '!' @ident
        p2(Box<statement<'a>>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
    }
    impl<'a> super::Node<'a, ()> for statement_with_cond<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0) => {
                    destructor.non_token(i0)?;
                }
                Self::p1(i0, i1, i2) => {
                    destructor.non_token(i0)?;
                    destructor.token(i1)?;
                    destructor.token(i2)?;
                }
                Self::p2(i0, i1, i2, i3) => {
                    destructor.non_token(i0)?;
                    destructor.token(i1)?;
                    destructor.token(i2)?;
                    destructor.token(i3)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for statement_with_cond<'a> {
        fn name(&self) -> &str { "statement_with_cond" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                Self::p0(t) => Ok(t),
                _ => Err(self)
            }
        }
    }
    pub enum token<'a> {
        /// token -> '@' @ident '=' @integer ';'
        p0(Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>, Box<(super::Token<'a, ()>, bool)>),
    }
    impl<'a> super::Node<'a, ()> for token<'a> {
        fn into_token(self: Box<Self>) -> Result<Box<(super::Token<'a, ()>, bool)>, Box<dyn super::Node<'a, ()> + 'a>> {
            Err(self)
        }
        fn destruct(self: Box<Self>, destructor: &mut dyn super::NodeDestructor<'a, ()>) -> super::ControlFlow<()> {
            match *self {
                Self::p0(i0, i1, i2, i3, i4) => {
                    destructor.token(i0)?;
                    destructor.token(i1)?;
                    destructor.token(i2)?;
                    destructor.token(i3)?;
                    destructor.token(i4)?;
                }
            }
            super::ControlFlow::Continue(())
        }
    }
    impl<'a> super::NonToken<'a, ()> for token<'a> {
        fn name(&self) -> &str { "token" }
        fn into_one(self: Box<Self>) -> Result<Box<dyn super::Node<'a, ()> + 'a>, Box<dyn super::NonToken<'a, ()> + 'a>> {
            match *self {
                _ => Err(self)
            }
        }
    }
    impl<'a> super::RootNode<'a, ()> for script<'a> {
        fn entry_state<'r, F>(states: &'r [super::State<'r, F>]) -> &'r super::State<'r, F> {
            unsafe { states.get_unchecked(0) }
        }
    }
}

