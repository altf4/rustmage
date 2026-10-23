//! Layer-six dependency order. This evaluates a frozen earlier-layer recipient
//! set; source abilities and resolution-created effects retain distinct origins.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GrantedAbilityKey {
    pub source: ObjectRef,
    pub ability: u16,
    pub granting_card: Card,
    pub grantor_controller: Player,
    pub timestamp: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GrantedManaAbility {
    pub source: ObjectRef,
    pub production: ManaProduction,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LayerSixObject {
    pub printed: bool,
    pub hexproof_colors: u8,
    pub keywords: Keywords,
    pub grants: Vec<GrantedAbilityKey>,
    pub mana: Vec<GrantedManaAbility>,
}
#[derive(Clone, Debug)]
pub(super) struct LayerSixGrant {
    pub recipient: CardId,
    pub keywords: Keywords,
    pub abilities: Vec<GrantedAbilityKey>,
    pub mana: Vec<GrantedManaAbility>,
}
#[derive(Clone, Debug)]
pub(super) enum LayerSixAction {
    Remove,
    GrantHexproofColors(u8),
    RemoveAndGrant(Vec<LayerSixGrant>),
    GrantEach(Vec<LayerSixGrant>),
    Grant {
        keywords: Keywords,
        abilities: Vec<GrantedAbilityKey>,
    },
}
#[derive(Clone, Debug)]
pub(super) struct LayerSixEffect {
    /// None for resolution-created effects, which survive their source's loss.
    pub provider: Option<CardId>,
    /// CR613.6: an effect that already started in an earlier layer continues.
    pub started_earlier: bool,
    pub timestamp: u64,
    pub recipients: Vec<CardId>,
    pub action: LayerSixAction,
}
#[derive(Clone, Debug)]
pub(super) struct LayerSixResult {
    pub objects: Vec<LayerSixObject>,
    /// A static effect that starts in layer6 keeps its later-layer fragments.
    pub started: Vec<bool>,
}
impl LayerSixEffect {
    fn active(&self, objects: &[LayerSixObject]) -> bool {
        self.started_earlier || self.provider.is_none_or(|id| objects[id as usize].printed)
    }
}

/// Find a dependency-free component, with timestamp order ONLY inside an actual
/// cycle. In particular an older dependent outside a cycle must still wait.
fn next_effect(pending: &[bool], dependencies: &[Vec<usize>], times: &[u64]) -> usize {
    // Iterative Kosaraju avoids a call-stack limit on large legal effect sets.
    let n = pending.len();
    let mut seen = vec![false; n];
    let mut finished = Vec::new();
    for root in 0..n {
        if !pending[root] || seen[root] {
            continue;
        }
        seen[root] = true;
        let mut stack = vec![(root, 0)];
        while let Some((node, edge)) = stack.last_mut() {
            if let Some(&next) = dependencies[*node].get(*edge) {
                *edge += 1;
                if !seen[next] {
                    seen[next] = true;
                    stack.push((next, 0));
                }
            } else {
                finished.push(*node);
                stack.pop();
            }
        }
    }
    let mut reverse = vec![Vec::new(); n];
    for (from, edges) in dependencies.iter().enumerate() {
        for &to in edges {
            reverse[to].push(from);
        }
    }
    let mut components = vec![usize::MAX; n];
    let mut count = 0;
    for root in finished.into_iter().rev() {
        if components[root] != usize::MAX {
            continue;
        }
        components[root] = count;
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            for &next in &reverse[node] {
                if components[next] == usize::MAX {
                    components[next] = count;
                    stack.push(next);
                }
            }
        }
        count += 1;
    }
    let mut waiting = vec![false; count];
    for (from, edges) in dependencies.iter().enumerate() {
        for &to in edges {
            if components[from] != components[to] {
                waiting[components[from]] = true;
            }
        }
    }
    (0..n)
        .filter(|&i| pending[i] && !waiting[components[i]])
        .min_by_key(|&i| (times[i], i))
        .expect("nonempty dependency graph has a sink component")
}

/// Grants never change whether a provider has its printed abilities. Without
/// removal there are no dependency edges, so the existing timestamp/index order
/// is the complete layer-six order. Keep the dependency algorithm for removals.
pub(super) fn apply_layer_six(
    mut objects: Vec<LayerSixObject>,
    effects: &[LayerSixEffect],
) -> LayerSixResult {
    if effects.iter().any(|effect| {
        matches!(
            effect.action,
            LayerSixAction::Remove | LayerSixAction::RemoveAndGrant(_)
        )
    }) {
        return apply_layer_six_exhaustive(objects, effects);
    }
    let mut order: Vec<_> = (0..effects.len()).collect();
    order.sort_unstable_by_key(|&index| (effects[index].timestamp, index));
    let mut started = vec![false; effects.len()];
    for index in order {
        let effect = &effects[index];
        if effect.active(&objects) {
            started[index] = true;
            apply_effect(&mut objects, effect);
        }
    }
    LayerSixResult { objects, started }
}

pub(super) fn apply_layer_six_exhaustive(
    mut objects: Vec<LayerSixObject>,
    effects: &[LayerSixEffect],
) -> LayerSixResult {
    let mut pending = vec![true; effects.len()];
    let mut started = vec![false; effects.len()];
    let times: Vec<_> = effects.iter().map(|e| e.timestamp).collect();
    loop {
        // Abilities removed before their effect starts cannot supply an effect.
        // A source's earlier-layer effect or a resolving instruction is exempt.
        for (i, effect) in effects.iter().enumerate() {
            if pending[i] && !effect.active(&objects) {
                pending[i] = false;
            }
        }
        if !pending.iter().any(|&p| p) {
            break;
        }
        let mut dependencies = vec![Vec::new(); effects.len()];
        for (i, effect) in effects.iter().enumerate() {
            if !pending[i] || effect.started_earlier {
                continue;
            }
            let Some(source) = effect.provider else {
                continue;
            };
            for (j, other) in effects.iter().enumerate() {
                if i != j
                    && pending[j]
                    && matches!(
                        other.action,
                        LayerSixAction::Remove | LayerSixAction::RemoveAndGrant(_)
                    )
                    && other.recipients.contains(&source)
                {
                    dependencies[i].push(j);
                }
            }
        }
        let next = next_effect(&pending, &dependencies, &times);
        let effect = &effects[next];
        pending[next] = false;
        started[next] = true;
        apply_effect(&mut objects, effect);
    }
    LayerSixResult { objects, started }
}

fn apply_effect(objects: &mut [LayerSixObject], effect: &LayerSixEffect) {
    if let LayerSixAction::RemoveAndGrant(_) = &effect.action {
        for &recipient in &effect.recipients {
            let object = &mut objects[recipient as usize];
            object.printed = false;
            object.keywords = Keywords::NONE;
            object.hexproof_colors = 0;
            object.grants.clear();
            object.mana.clear();
        }
    }
    if let LayerSixAction::GrantEach(rows) | LayerSixAction::RemoveAndGrant(rows) = &effect.action {
        for row in rows {
            let object = &mut objects[row.recipient as usize];
            object.keywords = object.keywords.union(row.keywords);
            object.grants.extend_from_slice(&row.abilities);
            object.mana.extend_from_slice(&row.mana);
        }
        return;
    }
    for &recipient in &effect.recipients {
        let object = &mut objects[recipient as usize];
        match &effect.action {
            LayerSixAction::GrantEach(_) | LayerSixAction::RemoveAndGrant(_) => unreachable!(),
            LayerSixAction::Remove => {
                object.printed = false;
                object.keywords = Keywords::NONE;
                object.hexproof_colors = 0;
                object.grants.clear();
                object.mana.clear();
            }
            LayerSixAction::GrantHexproofColors(colors) => object.hexproof_colors |= colors,
            LayerSixAction::Grant {
                keywords,
                abilities,
            } => {
                object.keywords = object.keywords.union(*keywords);
                object.grants.extend_from_slice(abilities);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn objects(n: usize) -> Vec<LayerSixObject> {
        vec![
            LayerSixObject {
                printed: true,
                hexproof_colors: 0,
                keywords: Keywords::FLYING,
                grants: Vec::new(),
                mana: Vec::new()
            };
            n
        ]
    }
    fn effect(
        provider: Option<CardId>,
        timestamp: u64,
        target: CardId,
        action: LayerSixAction,
    ) -> LayerSixEffect {
        LayerSixEffect {
            provider,
            started_earlier: false,
            timestamp,
            recipients: vec![target],
            action,
        }
    }
    fn grant() -> LayerSixAction {
        LayerSixAction::Grant {
            keywords: Keywords::HASTE,
            abilities: Vec::new(),
        }
    }
    #[test]
    fn ability_order_dependency_beats_timestamps_but_earlier_started_job_continues() {
        for early in [false, true] {
            let mut job = effect(Some(0), 1, 2, grant());
            job.started_earlier = early;
            let loss = effect(Some(1), 5, 0, LayerSixAction::Remove);
            let result = apply_layer_six(objects(3), &[job, loss]);
            assert!(!result.objects[0].printed);
            assert_eq!(result.started[0], early);
            assert_eq!(result.objects[2].keywords.contains(Keywords::HASTE), early);
        }
    }
    #[test]
    fn ability_order_resolution_grant_and_nonkeyword_grant_follow_recipient_timestamp() {
        let key = GrantedAbilityKey {
            source: ObjectRef {
                card: 0,
                generation: 8,
            },
            ability: 3,
            granting_card: Card::GrizzlyBears,
            grantor_controller: Player::P0,
            timestamp: 9,
        };
        for late in [false, true] {
            let loss = effect(Some(1), 5, 2, LayerSixAction::Remove);
            let grant = effect(
                None,
                if late { 9 } else { 2 },
                2,
                LayerSixAction::Grant {
                    keywords: Keywords::MENACE.union(Keywords::CANT_BLOCK),
                    abilities: vec![key],
                },
            );
            let result = apply_layer_six(objects(3), &[loss, grant]);
            assert!(!result.objects[2].printed);
            assert_eq!(!result.objects[2].grants.is_empty(), late);
            assert_eq!(result.objects[2].keywords.contains(Keywords::MENACE), late);
            assert_eq!(
                result.objects[2].keywords.contains(Keywords::CANT_BLOCK),
                late
            );
        }
    }
    #[test]
    fn ability_order_real_dependency_cycle_does_not_release_older_outside_dependent() {
        // A and B remove one another's abilities. An older lord on A depends
        // on B but is not in their cycle, so timestamp must not rescue the lord.
        let lord = effect(Some(0), 0, 2, grant());
        let a = effect(Some(0), 3, 1, LayerSixAction::Remove);
        let b = effect(Some(1), 2, 0, LayerSixAction::Remove);
        let result = apply_layer_six(objects(3), &[lord, a, b]);
        assert_eq!(result.started, [false, false, true]);
        assert!(!result.objects[0].printed);
        assert!(result.objects[1].printed);
        assert!(!result.objects[2].keywords.contains(Keywords::HASTE));
    }
}
