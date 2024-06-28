use std::{collections::BinaryHeap, ops::{Add, DerefMut}, cmp::Ordering, iter::FusedIterator, mem::forget, borrow::Borrow};



pub trait PathFindingNode<TArc: PathFindingArc<Self>>: Sized {
    fn with<R, F: FnOnce(&mut PathFindingNodeData<Self, TArc>) -> R>(&self, f: F) -> R;
}

pub trait PathFindingArc<TNode> {
    type Cost;
    type DataMut<'a>;
    fn cost(&self) -> Self::Cost;
    fn target(&self, data: &mut Self::DataMut<'_>) -> TNode;
}

pub trait PathFindingAdjacent<TNode, TArc: PathFindingArc<TNode>> {
    type AdjacentIter<'a>: Iterator<Item = TArc> where Self: 'a, TNode: 'a;
    fn adjacents<'a>(&'a mut self, node: &'a TNode) -> Self::AdjacentIter<'a>;
}

pub trait PathFindingEstimator<TCost, TNode> {
    fn estimate(&mut self, node: &TNode) -> TCost;
    fn compare(&mut self, old: TNode, new: &TNode) -> TNode;
}

impl<TCost: Default, TNode: Clone> PathFindingEstimator<TCost, TNode> for () {
    fn estimate(&mut self, _: &TNode) -> TCost {
        Default::default()
    }

    fn compare(&mut self, _: TNode, new: &TNode) -> TNode {
        new.clone()
    }
}

pub trait PathFindingMap<TNode, TArc: PathFindingArc<TNode>> {
    type NodeIter<'a>: Iterator where Self: 'a, TNode: 'a;
    type DataMut<'a>: DerefMut<Target = PathFindingMapData<TArc::Cost, TNode>> where Self: 'a;
    
    fn iter(&self) -> Self::NodeIter<'_>;
    fn data_mut(&self) -> Self::DataMut<'_>;
}

pub struct PathFindingMapData<TCost, TNode> {
    ticket: usize,
    heap: BinaryHeap<HeapNode<TCost, TNode>>,
}

impl<TCost: Ord, TNode> PathFindingMapData<TCost, TNode> {
    pub fn new() {
        Default::default()
    }
}

impl<TCost: Ord, TNode> Default for PathFindingMapData<TCost, TNode> {
    fn default() -> Self {
        Self {
            ticket: 0,
            heap: BinaryHeap::new(),
        }
    }
}

pub struct PathFindingNodeData<TNode, TArc: PathFindingArc<TNode>> {
    ticket: usize,
    open: bool,
    real_cost: TArc::Cost,
    estimated_cost: TArc::Cost,
    from: Option<(TNode, TArc)>,
    step: isize,
}

impl<TNode, TArc> Default for PathFindingNodeData<TNode, TArc>
where
    TArc: PathFindingArc<TNode>,
    TArc::Cost: Default,
{
    fn default() -> Self {
        Self {
            ticket: 0,
            open: false,
            real_cost: Default::default(),
            estimated_cost: Default::default(),
            from: None,
            step: 0,
        }
    }
}

impl<TNode, TArc: PathFindingArc<TNode>> PathFindingNodeData<TNode, TArc> {
    pub fn reset_ticket(&mut self) {
        self.ticket = 0;
    }
}

struct HeapNode<TCost, TNode> (TCost, TNode);

impl<TCost: Ord, TNode> PartialOrd for HeapNode<TCost, TNode> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<TCost: Ord, TNode> PartialEq for HeapNode<TCost, TNode> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<TCost: Ord, TNode> Eq for HeapNode<TCost, TNode> {}

impl<TCost: Ord, TNode> Ord for HeapNode<TCost, TNode> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

pub struct PathFinding<TCost, TNode, TEstimator, TAdjacent, TDataMut: DerefMut<Target = PathFindingMapData<TCost, TNode>>> {
    data: TDataMut,
    estimator: TEstimator,
    adjacent: TAdjacent,
    ticket: usize,
    closest: TNode,
}

impl<TCost, TNode, TEstimator, TAdjacent, TDataMut> PathFinding<TCost, TNode, TEstimator, TAdjacent, TDataMut>
where
    TDataMut: DerefMut<Target = PathFindingMapData<TCost, TNode>>,
{
    pub fn new<'a, TMap, TArc, I>(map: &'a TMap, mut estimator: TEstimator, adjacent: TAdjacent, starts: I) -> Option<Self>
    where
        TCost: Ord + Default + Add<Output = TCost> + Copy,
        TNode: PathFindingNode<TArc> + Clone + 'a,
        TArc: PathFindingArc<TNode, Cost = TCost, DataMut<'a> = TDataMut>,
        TMap: ?Sized + PathFindingMap<TNode, TArc, DataMut<'a> = TDataMut>,
        <TMap::NodeIter<'a> as Iterator>::Item: Borrow<TNode>,
        TEstimator: PathFindingEstimator<TCost, TNode>,
        I: IntoIterator<Item = TArc>,
    {
        let mut data = map.data_mut();
        let ticket = {
            let (ticket, overflow) = data.ticket.overflowing_add(1);
            if overflow {
                for node in map.iter() {
                    node.borrow().with(|data| data.ticket = 0);
                }
                data.ticket = 1;
                1
            } else {
                ticket
            }
        };
        for arc in starts.into_iter() {
            let node = arc.target(&mut data);
            let cost = node.with(|data| {
                data.ticket = ticket;
                data.real_cost = Default::default();
                data.estimated_cost = estimator.estimate(&node);
                data.from = None;
                data.open = true;
                data.step = 0;

                data.real_cost + data.estimated_cost
            });
            data.heap.push(HeapNode(cost, node));
        }
        if let Some(HeapNode(_, top)) = data.heap.peek() {
            let closest = top.clone();
            Some(Self { 
                data, estimator, adjacent, ticket,
                closest,
            })
        } else {
            None
        }
    }

    pub fn closest(&self) -> &TNode {
        &self.closest
    }

    pub fn estimator_mut(&mut self) -> &mut TEstimator {
        &mut self.estimator
    }

    pub fn advance<'a, TArc>(&mut self) -> bool
    where
        TCost: Ord + Default + Add<Output = TCost> + Copy,
        TNode: PathFindingNode<TArc> + Clone + Eq,
        TArc: PathFindingArc<TNode, Cost = TCost, DataMut<'a> = TDataMut>,
        TEstimator: PathFindingEstimator<TCost, TNode>,
        TAdjacent: PathFindingAdjacent<TNode, TArc>,
    {
        let data = &mut self.data;
        if let Some(HeapNode(_, node)) = data.heap.pop() {
            node.with(|node_data| {
                if node_data.open {
                    node_data.open = false;
                    let step = node_data.step + 1;

                    for arc in self.adjacent.adjacents(&node) {
                        let next = arc.target(data);
                        next.with(|next_data| {
                            if next_data.ticket != self.ticket {
                                next_data.ticket = self.ticket;
                                next_data.real_cost = node_data.real_cost + arc.cost();
                                next_data.estimated_cost = self.estimator.estimate(&next);
                                next_data.from = Some((node.clone(), arc));
                                next_data.step = step;
                                next_data.open = true;
                                data.heap.push(HeapNode(next_data.real_cost + next_data.estimated_cost, next.clone()));
                                
                            } else if next_data.open {
                                let new_cost = node_data.real_cost + arc.cost();
                                if new_cost < next_data.real_cost {
                                    next_data.real_cost = new_cost;
                                    next_data.from = Some((node.clone(), arc));
                                    next_data.step = step;

                                    data.heap.retain(|t| t.1 != next);
                                    data.heap.push(HeapNode(new_cost + next_data.estimated_cost, next.clone()));
                                }
                            }
                        });
                    }
                }
            });
            if let Some(HeapNode(_, top)) = data.heap.peek() {
                self.closest = self.estimator.compare(node, top);
                return true;
            }
        }
        false
    }

    pub fn into_reversed_path<TArc>(self) -> ReversedPath<TDataMut, TNode, TArc>
    where
        TNode: PathFindingNode<TArc>,
        TArc: PathFindingArc<TNode, Cost = TCost>
    {
        unsafe {
            use std::ptr::read;
            let data = read(&self.data);
            let _ = read(&self.estimator);
            let _ = read(&self.adjacent);
            let closest = read(&self.closest);
            forget(self);

            let item = closest.with(|data| data.from.take());
            ReversedPath { data, item }
        }
    }

}

impl<TCost, TNode, TEstimator, TAdjacent, TDataMut: DerefMut<Target = PathFindingMapData<TCost, TNode>>> Drop for PathFinding<TCost, TNode, TEstimator, TAdjacent, TDataMut> {
    fn drop(&mut self) {
        self.data.heap.clear();
    }
}


pub struct ReversedPath<TDataMut: DerefMut<Target = PathFindingMapData<TArc::Cost, TNode>>, TNode, TArc: PathFindingArc<TNode>> {
    data: TDataMut,
    item: Option<(TNode, TArc)>,
}

impl<TDataMut, TNode, TArc> Iterator for ReversedPath<TDataMut, TNode, TArc>
where
    TDataMut: DerefMut<Target = PathFindingMapData<TArc::Cost, TNode>>,
    TArc: PathFindingArc<TNode>,
    TNode: PathFindingNode<TArc> + Clone,
{
    type Item = (TNode, TArc);
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.item.take();
        if let Some((ref node, _)) = ret {
            self.item = node.with(|data| data.from.take());
        }
        ret
    }
}

impl<TDataMut, TNode, TArc> FusedIterator for ReversedPath<TDataMut, TNode, TArc>
where
    TDataMut: DerefMut<Target = PathFindingMapData<TArc::Cost, TNode>>,
    TArc: PathFindingArc<TNode>,
    TNode: PathFindingNode<TArc> + Clone,
{
}

impl<TDataMut: DerefMut<Target = PathFindingMapData<TArc::Cost, TNode>>, TNode, TArc: PathFindingArc<TNode>> Drop for ReversedPath<TDataMut, TNode, TArc> {
    fn drop(&mut self) {
        self.data.heap.clear();
    }
}