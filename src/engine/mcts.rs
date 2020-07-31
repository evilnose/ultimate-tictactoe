use crate::engine::utils::*;
use crate::moves::*;
use rand::Rng;
use std::time::Instant;

type Value = f32;
type NodeIdx = u32;
const NULL_NODE_IDX: NodeIdx = std::u32::MAX;

pub struct MCTSResult {
    pub best_move: Idx,
    pub value: Value,
}

// a Monte-Carlo Tree Node
struct TreeNode {
    position: Position,
    children: Vec<NodeIdx>,
    value: Value,
    n: u32, // number of times visited this node
}

impl TreeNode {
    fn new(pos: Position) -> TreeNode {
        TreeNode {
            position: pos,
            children: Vec::new(),
            value: 1.0, // anything but zero works
            n: 0,
        }
    }
}

/* TODO */
pub struct MCTSWorker<R: Rng> {
    all_nodes: Vec<TreeNode>,
    my_side: Side,
    my_side_mult: f32,
    c: Value, // C parameter
    rng: R,
}

impl<R: Rng> MCTSWorker<R> {
    pub fn new(pos: Position, c: Value, rng: R) -> MCTSWorker<R> {
        let mut worker = MCTSWorker::<R> {
            all_nodes: Vec::new(),
            my_side: pos.to_move,
            my_side_mult: side_multiplier(pos.to_move),
            c: c,
            rng: rng,
        };
        let root = TreeNode::new(pos);
        worker.all_nodes.push(root);
        return worker;
    }

    pub fn go(&mut self, millis: u64) -> MCTSResult {
        let now = Instant::now();
        let mut n_rollouts = 0;
        loop {
            if n_rollouts % 500 == 0 {
                if now.elapsed().as_millis() as u64 > millis - 10 {
                    eprintln!("{} rollouts", n_rollouts);
                    return self.get_best();
                }
            }
            self.treewalk(0);
            n_rollouts += 1;
        }
    }

    fn treewalk(&mut self, mut idx: NodeIdx) {
        let len = self.all_nodes.len();
        let mut explored_nodes = Vec::new();
        let r: Value;
        loop {
            explored_nodes.push(idx);
            let node = &self.all_nodes[idx as usize];
            let localpos = node.position;
            if node.children.len() == 0 {
                // is leaf node
                if node.n == 0 {
                    // never sampled before; rollout now
                    /* mutable borrow of self and node ends here */
                    r = self.rollout(localpos);
                    break;
                } else {
                    /* begin mut borrow of node */
                    if localpos.is_won(localpos.to_move.other()) ||
                        localpos.is_drawn() {
                        // early end; since game is already over we can directly take its value
                        r = node.value;
                        break;
                    }

                    debug_assert!(node.n == 1);
                    /* do mutable borrow here since need to modify children */
                    let node = &mut self.all_nodes[idx as usize];
                    // expand
                    let moves = localpos.legal_moves();
                    for i in 0..moves.size() {
                        node.children.push((len + i) as NodeIdx);
                    }
                    /* end mut borrow of node */

                    for mov in moves {
                        let mut newpos = localpos;
                        newpos.make_move(mov);
                        self.all_nodes.push(TreeNode::new(newpos));
                    }
                    idx = len as NodeIdx;
                }
            } else {
                // find best child
                let mut best_ucb: f32 = std::f32::NEG_INFINITY;
                let mut best_idx: u32 = NULL_NODE_IDX;

                let ln = natural_log(node.n);
                debug_assert!(ln != 0.0);
                //let c = 1.4 / 9.0 * node.children.len() as f32;
                for i in &node.children {
                    let child = &self.all_nodes[*i as usize];
                    let ucb = child.value * side_multiplier(node.position.to_move)
                        + self.c * (ln / (child.n as Value)).sqrt();
                    debug_assert!(!ucb.is_nan());
                    if ucb > best_ucb {
                        best_ucb = ucb;
                        best_idx = *i;
                    }
                }
                debug_assert!(best_idx != NULL_NODE_IDX);

                idx = best_idx as NodeIdx;
            }
        }
        self.backpropagate(r, explored_nodes);
    }

    fn backpropagate(&mut self, r: Value, explored_nodes: Vec<u32>) {
        for idx in explored_nodes {
            let mut node = &mut self.all_nodes[idx as usize];
            // UCT
            node.n += 1;
            node.value = node.value + (r - node.value) / (node.n as Value);
        }
    }

    fn rollout(&mut self, mut pos: Position) -> Value {
        loop {
            if pos.is_won(pos.to_move.other()) {
                return (pos.to_move != Side::X) as i32 as Value;
            } else if pos.is_drawn() {
                let sign = codingame_drawn(&pos);
                return 0.5 + 0.5 * sign;
            }
            let moves = pos.legal_moves();
            let n_moves = moves.size();
            let j = self.rng.gen_range(0, n_moves);
            let mov = moves.nth_move(j as u8);

            pos.make_move(mov);
        }
    }

    fn get_best(&self) -> MCTSResult {
        let mut best_move = NULL_IDX;
        /* NOTE score is for determining which node to select as best,
        while value is the supposed value of the node. One can have
        a different score and value */
        let mut best_score = std::f32::NEG_INFINITY;
        let mut best_value = std::f32::NEG_INFINITY;
        let mut i = 1;
        for mov in self.all_nodes[0].position.legal_moves() {
            let child = &self.all_nodes[i as usize];
            // TODO is this a good criterion
            let score = child.n as f32;
            if score > best_score {
                best_score = score;
                best_move = mov;
                best_value = child.value;
            }
            i += 1;
        }
        assert!(best_move != NULL_IDX);
        return MCTSResult {
            best_move: best_move,
            value: best_value,
        };
    }

    pub fn pv(&self) -> Vec<MCTSResult> {
        let mut cur = 0;
        let mut ret = Vec::new();
        while self.all_nodes[cur].children.len() != 0 {
            let node = &self.all_nodes[cur];

            let mut best_move = NULL_IDX;
            /* NOTE score is for determining which node to select as best,
            while value is the supposed value of the node. One can have
            a different score and value */
            let mut best_score = std::f32::NEG_INFINITY;
            let mut best_value = std::f32::NEG_INFINITY;
            let mut best_i = 0;
            let mut i = node.children[0];
            for mov in node.position.legal_moves() {
                let child = &self.all_nodes[i as usize];
                // TODO is this a good criterion
                let score = child.n as f32;
                if score > best_score {
                    best_score = score;
                    best_move = mov;
                    best_value = child.value;
                    best_i = i;
                }
                i += 1;
            }
            cur = best_i as usize;
            assert!(best_move != NULL_IDX);
            ret.push(MCTSResult {
                best_move: best_move,
                value: best_value,
            });
        }
        return ret;
    }
}
