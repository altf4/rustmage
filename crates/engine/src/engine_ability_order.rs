//! Layer-six dependency order. This evaluates a frozen earlier-layer recipient
//! set; source abilities and resolution-created effects retain distinct origins.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GrantedAbilityKind {
    Template(u16),
    PlaneswalkerAttackLimit,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GrantedAbilityKey {
    pub source: ObjectRef,
    pub kind: GrantedAbilityKind,
    pub granting_card: Card,
    pub grantor_controller: Player,
    pub timestamp: u64,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GrantedManaAbility {
    pub source: ObjectRef,
    pub production: ManaProduction,
    pub sacrifice: bool,
    /// Intrinsic to a basic land subtype, removable by a later type replacement.
    pub subtype_derived: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LayerSixObject {
    pub printed: bool,
    pub keywords: Keywords,
    /// Acquired keywords retained when a type effect removes rules-text abilities.
    pub granted_keywords: Keywords,
    pub grants: Vec<GrantedAbilityKey>,
    /// Independently acquired self-owned static/activated ability instances.
    pub self_grants: Vec<u64>,
    pub mana: Vec<GrantedManaAbility>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug)]
pub(super) struct LayerSixGrant {
    pub recipient: CardId,
    pub keywords: Keywords,
    pub abilities: Vec<GrantedAbilityKey>,
    pub mana: Vec<GrantedManaAbility>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug)]
pub(super) enum LayerSixAction {
    Remove,
    /// Losing the land card type removes its subtype-derived abilities only.
    RemoveSubtypeMana,
    GrantSelf(u64),
    RemoveAndGrant(Vec<LayerSixGrant>),
    /// CR305.7 removes rules-text/copiable abilities, but not grants from other effects.
    RemovePrintedAndGrant(Vec<LayerSixGrant>),
    GrantEach(Vec<LayerSixGrant>),
    Grant {
        keywords: Keywords,
        abilities: Vec<GrantedAbilityKey>,
    },
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug)]
pub(super) struct LayerSixResult {
    pub objects: Vec<LayerSixObject>,
    /// A static effect that starts in layer6 keeps its later-layer fragments.
    pub started: Vec<bool>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl LayerSixEffect {
    fn active(&self, objects: &[LayerSixObject]) -> bool {
        self.started_earlier || self.provider.is_none_or(|id| objects[id as usize].printed)
    }
}

/// Find a dependency-free component, with timestamp order ONLY inside an actual
/// cycle. In particular an older dependent outside a cycle must still wait.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn apply_layer_six(
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
                        LayerSixAction::Remove
                            | LayerSixAction::RemoveAndGrant(_)
                            | LayerSixAction::RemovePrintedAndGrant(_)
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
        if let LayerSixAction::RemovePrintedAndGrant(_) = &effect.action {
            for &recipient in &effect.recipients {
                let object = &mut objects[recipient as usize];
                object.printed = false;
                object.keywords = object.granted_keywords;
                object.mana.retain(|ability| !ability.subtype_derived);
            }
        }
        if let LayerSixAction::RemoveAndGrant(_) = &effect.action {
            for &recipient in &effect.recipients {
                let object = &mut objects[recipient as usize];
                object.printed = false;
                object.keywords = Keywords::NONE;
                object.granted_keywords = Keywords::NONE;
                object.grants.clear();
                object.self_grants.clear();
                object.mana.clear();
            }
        }
        if let LayerSixAction::GrantEach(rows)
        | LayerSixAction::RemoveAndGrant(rows)
        | LayerSixAction::RemovePrintedAndGrant(rows) = &effect.action
        {
            for row in rows {
                let object = &mut objects[row.recipient as usize];
                object.keywords = object.keywords.union(row.keywords);
                object.granted_keywords = object.granted_keywords.union(row.keywords);
                object.grants.extend_from_slice(&row.abilities);
                object.mana.extend_from_slice(&row.mana);
            }
            continue;
        }
        for &recipient in &effect.recipients {
            let object = &mut objects[recipient as usize];
            match &effect.action {
                LayerSixAction::GrantEach(_)
                | LayerSixAction::RemoveAndGrant(_)
                | LayerSixAction::RemovePrintedAndGrant(_) => unreachable!(),
                LayerSixAction::RemoveSubtypeMana => {
                    object.mana.retain(|ability| !ability.subtype_derived);
                }
                LayerSixAction::Remove => {
                    object.printed = false;
                    object.keywords = Keywords::NONE;
                    object.granted_keywords = Keywords::NONE;
                    object.grants.clear();
                    object.self_grants.clear();
                    object.mana.clear();
                }
                LayerSixAction::GrantSelf(timestamp) => object.self_grants.push(*timestamp),
                LayerSixAction::Grant {
                    keywords,
                    abilities,
                } => {
                    object.keywords = object.keywords.union(*keywords);
                    object.granted_keywords = object.granted_keywords.union(*keywords);
                    object.grants.extend_from_slice(abilities);
                }
            }
        }
    }
    LayerSixResult { objects, started }
}
