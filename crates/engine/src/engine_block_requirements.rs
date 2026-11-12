//! Exact blocker and pair requirements add weights to legal combat assignments.
use super::*;

#[derive(Clone, Copy)]
struct AttackSlots {
    capacity: usize,
    required: usize,
    first_block: usize,
}

#[derive(Clone, Copy)]
struct BlockBundle {
    attacker: usize,
    minimum: usize,
    first_block: usize,
    company_only: bool,
}

struct BlockAssignments {
    // One row per available blocker. An absent edge is an illegal pair.
    edges: Vec<Vec<Option<usize>>>,
    slots: Vec<AttackSlots>,
    // Menace and similar restrictions make these slots optional as a group.
    bundles: Vec<BlockBundle>,
    fixed_blocks: usize,
}

impl BlockAssignments {
    fn maximize(
        &mut self,
        bundle: usize,
        score: usize,
        has_company_bundle: bool,
        best: &mut Option<usize>,
    ) {
        if self.slots.iter().map(|slots| slots.required).sum::<usize>() > self.edges.len() {
            return;
        }
        if let Some(&row) = self.bundles.get(bundle) {
            self.maximize(bundle + 1, score, has_company_bundle, best);
            if !row.company_only || !has_company_bundle {
                let saved = self.slots[row.attacker];
                self.slots[row.attacker].capacity = self.edges.len();
                self.slots[row.attacker].required = row.minimum;
                self.maximize(
                    bundle + 1,
                    score + row.first_block,
                    has_company_bundle || row.company_only,
                    best,
                );
                self.slots[row.attacker] = saved;
            }
            return;
        }

        let blockers = self.edges.len();
        let first_attacker = 1 + blockers;
        let sink = first_attacker + self.slots.len();
        let mut graph = vec![Vec::new(); sink + 1];
        for (blocker, edges) in self.edges.iter().enumerate() {
            let from = 1 + blocker;
            add_edge(&mut graph, 0, from, 1, (0, 0));
            for (attacker, &weight) in edges.iter().enumerate() {
                if let Some(weight) = weight
                    && self.slots[attacker].capacity != 0
                {
                    add_edge(
                        &mut graph,
                        from,
                        first_attacker + attacker,
                        1,
                        (0, weight as i128),
                    );
                }
            }
        }
        let mut required = 0;
        for (attacker, slots) in self.slots.iter().enumerate() {
            let node = first_attacker + attacker;
            required += slots.required;
            add_edge(&mut graph, node, sink, slots.required, (1, 0));
            let bonus = usize::from(slots.first_block != 0);
            add_edge(
                &mut graph,
                node,
                sink,
                bonus,
                (0, slots.first_block as i128),
            );
            add_edge(
                &mut graph,
                node,
                sink,
                slots.capacity - slots.required - bonus,
                (0, 0),
            );
        }

        let mut total = (0, 0);
        for count in 1..=blockers {
            let Some(delta) = augment(&mut graph, sink) else {
                break;
            };
            total = (total.0 + delta.0, total.1 + delta.1);
            if total.0 == required as i128 && self.fixed_blocks + count >= 2 {
                let candidate = score + usize::try_from(total.1).expect("nonnegative block score");
                *best = Some(best.map_or(candidate, |best| best.max(candidate)));
            }
        }
    }
}

// Prefer all mandatory slots before comparing requirements. After each
// augmenting path, the residual graph has no improving cycle, so a longest
// path gives the best score at the next cardinality. Keep every feasible size.
type Score = (i128, i128);

#[derive(Clone, Copy)]
struct Edge {
    to: usize,
    reverse: usize,
    capacity: usize,
    score: Score,
}

fn add_edge(graph: &mut [Vec<Edge>], from: usize, to: usize, capacity: usize, score: Score) {
    if capacity == 0 {
        return;
    }
    let forward = graph[from].len();
    let reverse = graph[to].len();
    graph[from].push(Edge {
        to,
        reverse,
        capacity,
        score,
    });
    graph[to].push(Edge {
        to: from,
        reverse: forward,
        capacity: 0,
        score: (-score.0, -score.1),
    });
}

fn augment(graph: &mut [Vec<Edge>], sink: usize) -> Option<Score> {
    let mut scores = vec![None; graph.len()];
    let mut incoming = vec![None; graph.len()];
    scores[0] = Some((0, 0));
    for _ in 1..graph.len() {
        let mut changed = false;
        for (from, edges) in graph.iter().enumerate().take(sink) {
            let Some(score) = scores[from] else {
                continue;
            };
            for (index, edge) in edges.iter().enumerate() {
                if edge.capacity == 0 || edge.to == 0 {
                    continue;
                }
                let candidate = (score.0 + edge.score.0, score.1 + edge.score.1);
                if scores[edge.to].is_none_or(|previous| candidate > previous) {
                    scores[edge.to] = Some(candidate);
                    incoming[edge.to] = Some((from, index));
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    let best = scores[sink]?;
    let mut node = sink;
    while let Some((from, index)) = incoming[node] {
        let edge = &mut graph[from][index];
        edge.capacity -= 1;
        let reverse = edge.reverse;
        graph[node][reverse].capacity += 1;
        node = from;
    }
    Some(best)
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn has_creature_block_requirements(&self) -> bool {
        self.state.combat_rules.as_ref().is_some_and(|rules| {
            rules.creature_requirements.iter().any(|row| {
                let object = &self.state.objects[row.blocker.card as usize];
                object.zone == Zone::Battlefield
                    && object.controller == self.state.active.other()
                    && object.generation == row.blocker.generation
                    && row.attacker.is_none_or(|attacker| {
                        self.object_ref(attacker.card) == attacker
                            && self
                                .state
                                .attackers
                                .iter()
                                .any(|row| row.card == attacker.card)
                    })
            })
        })
    }

    pub(super) fn creature_block_requirement_weight(
        &self,
        blocker: CardId,
        attacker: CardId,
    ) -> usize {
        self.state.combat_rules.as_ref().map_or(0, |rules| {
            let blocker = self.object_ref(blocker);
            let attacker = self.object_ref(attacker);
            rules
                .creature_requirements
                .iter()
                .filter(|row| {
                    row.blocker == blocker
                        && row.attacker.is_none_or(|required| required == attacker)
                })
                .count()
        })
    }

    pub(super) fn maximum_weighted_block_requirements(
        &self,
        selected: &[(CardId, CardId)],
    ) -> Option<usize> {
        let available: Vec<_> = (0..self.state.objects.len())
            .map(|id| id as CardId)
            .filter(|&id| self.can_be_blocker(id) && !selected.iter().any(|&(b, _)| b == id))
            .collect();
        let attackers = &self.state.attackers;
        let edges: Vec<Vec<_>> = available
            .iter()
            .map(|&blocker| {
                attackers
                    .iter()
                    .map(|attacker| {
                        self.can_block_pair(blocker, attacker.card)
                            .then(|| self.creature_block_requirement_weight(blocker, attacker.card))
                    })
                    .collect()
            })
            .collect();
        let mut score = selected
            .iter()
            .map(|&(blocker, attacker)| self.creature_block_requirement_weight(blocker, attacker))
            .sum::<usize>();
        let mut slots = Vec::with_capacity(attackers.len());
        let mut bundles = Vec::new();
        let mut best_single = 0;
        let company: Vec<_> = available
            .iter()
            .map(|&blocker| self.combat_requires_company(blocker))
            .collect();
        let fixed_needs_company =
            matches!(selected, [(blocker, _)] if self.combat_requires_company(*blocker));
        let any_company_rule = fixed_needs_company || company.contains(&true);
        for (index, attacker) in attackers.iter().enumerate() {
            let count = selected
                .iter()
                .filter(|&&(_, id)| id == attacker.card)
                .count();
            let minimum = self.minimum_blockers(attacker.card);
            let weight = self.block_requirement_weight(attacker.card);
            let max_one = self
                .keywords(attacker.card)
                .contains(Keywords::MAX_ONE_BLOCKER);
            let eligible = edges.iter().filter(|row| row[index].is_some()).count();
            let capacity = if max_one {
                1usize.saturating_sub(count).min(eligible)
            } else {
                eligible
            };
            if count > 0 {
                let required = minimum.saturating_sub(count);
                if (max_one && count > 1) || required > capacity {
                    return None;
                }
                score += weight;
                slots.push(AttackSlots {
                    capacity,
                    required,
                    first_block: 0,
                });
            } else if minimum == 1 {
                let first_block = if capacity == 0 { 0 } else { weight };
                slots.push(AttackSlots {
                    capacity,
                    required: 0,
                    first_block,
                });
                if selected.is_empty() {
                    for (&requires_company, row) in company.iter().zip(&edges) {
                        if let Some(pair) = row[index]
                            && !requires_company
                        {
                            best_single = best_single.max(weight + pair);
                        }
                    }
                }
            } else {
                slots.push(AttackSlots {
                    capacity: 0,
                    required: 0,
                    first_block: 0,
                });
                // A zero-score bundle matters only as company for a sole
                // blocker. No declaration needs two such optional bundles.
                let company_only = weight == 0
                    && !edges
                        .iter()
                        .any(|row| row[index].is_some_and(|pair| pair > 0));
                if capacity >= minimum
                    && (!company_only || (any_company_rule && selected.len() < 2))
                {
                    bundles.push(BlockBundle {
                        attacker: index,
                        minimum,
                        first_block: weight,
                        company_only,
                    });
                }
            }
        }
        let mut best = if selected.is_empty() {
            Some(best_single)
        } else {
            (slots.iter().all(|slots| slots.required == 0) && !fixed_needs_company).then_some(score)
        };
        BlockAssignments {
            edges,
            slots,
            bundles,
            fixed_blocks: selected.len(),
        }
        .maximize(0, score, false, &mut best);
        best
    }
}
