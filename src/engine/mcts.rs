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
    moves: Moves,
    children: Vec<NodeIdx>,
    n: u32,  // number of times visited this node
    value: Value,
    rn: u32,  // RAVE n
    rvalue: Value,  // RAVE value
}

impl TreeNode {
    fn new(pos: Position) -> TreeNode {
        TreeNode {
            position: pos,
            moves: Moves::new(),
            children: Vec::new(),
            n: 0,
            // anything but zero works, since this is overwritten the first time avg is computed
            value: 1.0, 
            rn: 0,
            rvalue: 1.0,
        }
    }

    // return a seeded TreeNode, i.e. with pre-computed values and n, rn as confidence.
    fn from_heuristic(pos: Position) -> TreeNode {
        TreeNode {
            position: pos,
            moves: Moves::new(),
            children: Vec::new(),
            n: 1,
            value: 0.5,
            rn: 1,
            rvalue: 0.5
        }
    }
}

/* TODO */
pub struct MCTSWorker<R: Rng> {
    all_nodes: Vec<TreeNode>,
    c: Value, // C parameter
    rng: R,
}

impl<R: Rng> MCTSWorker<R> {
    pub fn new(pos: Position, c: Value, rng: R) -> MCTSWorker<R> {
        let mut worker = MCTSWorker::<R> {
            all_nodes: Vec::new(),
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
        let mut explored_nodes = Vec::new();
        let mut explored_moves = Vec::new();
        loop {
            let len = self.all_nodes.len();
            let node = &self.all_nodes[idx as usize];
            let localpos = node.position;
            if node.children.len() == 0 {
                // is leaf node
                /* begin mut borrow of node */
                if localpos.is_won(localpos.to_move.other()) || localpos.is_drawn() {
                    // early end; since game is already over we can directly take its value
                    break;
                }
                explored_nodes.push(idx);

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
                    self.all_nodes.push(TreeNode::from_heuristic(newpos));
                }
                /* re-borrow nodes to set moves */
                let node = &mut self.all_nodes[idx as usize];
                node.moves = moves;

                // push first move
                explored_moves.push(moves.peek());
                break;
            } else {
                explored_nodes.push(idx);
                // find best child
                let best_idx = self.select_move(node, self.c);
                idx = node.children[best_idx] as NodeIdx;
                explored_moves.push(node.moves.nth_move(best_idx as u8));
            }
        }
        assert_eq!(explored_moves.len(), explored_nodes.len());
        let localpos = (&self.all_nodes[idx as usize]).position;
        let r = self.rollout(localpos, &mut explored_moves);
        self.backpropagate(r, explored_nodes, explored_moves);
    }

    // returns the best move index and the corresponding node index
    fn select_move(&self, node: &TreeNode, c: f32) -> usize {
        let mut best_eval: f32 = std::f32::NEG_INFINITY;
        let mut best_idx: usize = 300;

        let b = 0.1;
        
        for i in 0..node.children.len() {
            let child = &self.all_nodes[node.children[i] as usize];
            let n = child.n as f32;
            let rn = child.rn as f32;
            let beta = rn / (n + rn + 4.0 * n * rn * b * b);
            let mult = side_multiplier(child.position.to_move); // TODO move this outside the loop
            let eval = ((1.0 - beta) * child.value + beta * child.rvalue) * mult;
            debug_assert!(!eval.is_nan());
            if eval > best_eval {
                best_eval = eval;
                best_idx = i;
            }
        }
        debug_assert!(best_idx != 300);
        return best_idx;
    }

    /**
     * Note that explored_nodes contains nodes in treewalk() whereas explored_moves not only
     * contains moves explored in treewalk() but also those in rollout()
     */
    fn backpropagate(&mut self, r: Value, explored_nodes: Vec<u32>, explored_moves: Vec<Idx>) {
        debug_assert!(explored_nodes.len() <= explored_moves.len());
        for i in 0..explored_nodes.len() {
            let next_idx;
            let moves: Moves;
            {
                let node = &self.all_nodes[explored_nodes[i] as usize];
                moves = node.moves;
                // UCT
                let a = explored_moves[i];
                // the next node according to explored_moves
                debug_assert!(node.moves.size() != 0);
                next_idx = node.children[node.moves.move_number(a) as usize] as usize;
            }
            {
                let mut next = &mut self.all_nodes[next_idx];
                next.n += 1;
                next.value = next.value + (r - next.value) / (next.n as Value);
            }

            // RAVE
            let mut j = i + 2;
            while j < explored_moves.len() {
                let mov = explored_moves[j];
                if moves.contains(mov) {
                    // I encountered this move later in the tree; according to AMAF record its value
                    let next_idx;
                    {
                        let node = &self.all_nodes[explored_nodes[i] as usize];
                        next_idx = node.children[moves.move_number(mov) as usize] as usize;
                    }

                    let mut next = &mut self.all_nodes[next_idx];
                    next.rn += 1;
                    next.rvalue = next.rvalue + (r - next.rvalue) / (next.rn as Value)
                }
                j += 2;
            }
        }
    }

    fn rollout(&mut self, mut pos: Position, out_moves: &mut Vec<Idx>) -> Value {
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

            out_moves.push(mov);
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
        for mov in self.all_nodes[0].moves {
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
            for mov in node.moves {
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
