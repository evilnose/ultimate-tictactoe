use std::time::Instant;
use crate::moves::*;
use crate::engine::utils::*;
use rand::Rng;

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
    n: u32,  // number of times visited this node
}

impl TreeNode {
    fn new(pos: Position) -> TreeNode {
        TreeNode {
            position: pos,
            children: Vec::new(),
            value: 1.0,  // anything but zero works
            n: 0,
        }
    }
}

/* TODO */
pub struct MCTSWorker<R: Rng> {
    all_nodes: Vec<TreeNode>,
    my_side: Side,
    my_side_mult: f32,
    c: Value,  // C parameter
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

    fn treewalk(&mut self, idx: NodeIdx) -> Value {
        let len = self.all_nodes.len();
        let r: Value;
        // leaf node
        if self.all_nodes[idx as usize].children.len() == 0 {
            if self.all_nodes[idx as usize].n == 0 {
                // never sampled before; rollout immediately
                r = self.rollout(self.all_nodes[idx as usize].position);

                // manually set n and value since this is the first time
                let node = &mut self.all_nodes[idx as usize];
                node.n = 1;
                node.value = r;
                return r;
            } else {
                /* begin mut borrow of node */
                let node = &mut self.all_nodes[idx as usize];
                let pos = node.position;

                if node.position.is_won(node.position.to_move.other()) || node.position.is_drawn() {
                    // early end
                    node.n += 1;
                    return node.value;
                }

                debug_assert!(node.n == 1);
                // expand
                let moves = node.position.legal_moves();
                for i in 0..moves.size() {
                    node.children.push((len + i) as u32);
                }
                /* end mut borrow of node */

                for mov in moves {
                    let mut newpos = pos;
                    newpos.make_move(mov);
                    self.all_nodes.push(TreeNode::new(newpos));
                }
                r = self.treewalk(len as u32);
            }
        } else {
            let node = &self.all_nodes[idx as usize];

            // find best child
            let mut best_ucb: f32 = std::f32::NEG_INFINITY;
            let mut best_idx: u32 = NULL_NODE_IDX;

            let ln = natural_log(node.n);
            debug_assert!(ln != 0.0);
            //let c = 1.4 / 9.0 * node.children.len() as f32;
            for i in &node.children {
                let child = &self.all_nodes[*i as usize];
                let ucb = child.value * side_multiplier(node.position.to_move) + self.c * (ln / (child.n as Value)).sqrt();
                debug_assert!(!ucb.is_nan());
                if ucb > best_ucb {
                    best_ucb = ucb;
                    best_idx = *i;
                }
            }
            debug_assert!(best_idx != NULL_NODE_IDX);

            r = self.treewalk(best_idx);
        }

        /* last mut borrow of node */
        let mut node = &mut self.all_nodes[idx as usize];
        node.n += 1;
        /* propagate; trust me, the algebra worked out */
        node.value = node.value + (r - node.value) / (node.n as Value);
        return r;
    }

    fn rollout(&mut self, mut pos: Position) -> Value {
        loop {
            if pos.is_won(pos.to_move.other()) {
                /*
                if pos.to_move == self.my_side {
                    return 0.0;
                } else {
                    return 1.0;
                }
                */
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
