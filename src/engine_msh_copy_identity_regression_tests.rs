//! Current paid copy-producer regressions. No synthetic card admission or runtime edits.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
use crate::cards::ReplacementSpec;

const TASK: &str = "Taskmaster, Mercenary Mimic";

fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "only admitted actual cards: {name}");
    let id = named(g, p, name, Zone::Hand);
    let (colored, generic) = card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle_keeping_new_copy(g);
    id
}
fn settle_keeping_new_copy(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..120 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        // A copied legendary Taskmaster/Monica requires the real public legend
        // choice. Keep the new token; do not fake away legendary or the source.
        let keep_copy = actions.iter().find(|action| {
            matches!(action, Action::KeepLegend { object }
                if g.object_view(object.card).is_token())
        });
        let action = keep_copy
            .or_else(|| actions.iter().find(|a| matches!(a, Action::Pass)))
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, action);
    }
    panic!("copy identity resolution stalled {:?}", g.state.pending);
}
fn next_main(g: &mut Game, p: Player) {
    let turn = g.state.turn;
    for _ in 0..240 {
        if g.state.turn > turn && g.state.active == p && g.state.step == Step::Main1 {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::Discard { .. }
                )
            })
            .unwrap();
        checked(g, action);
    }
    panic!("next first main did not arrive");
}
fn first_main_copy(g: &mut Game, source: CardId, blueprint: CardId) {
    g.prepare_triggers();
    let mut selected = false;
    for _ in 0..100 {
        if g.state.stack.iter().any(|s| {
            s.card == source && matches!(s.kind, StackKind::TriggeredAbility { ability: 0 })
        }) {
            settle(g);
            return;
        }
        let actions = g.legal_actions();
        let choose = Action::ChooseTarget {
            target: Target::Card(blueprint),
        };
        let action = if !selected && actions.contains(&choose) {
            selected = true;
            choose
        } else {
            actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::FinishTargets | Action::PutTrigger { .. } | Action::Pass
                    )
                })
                .unwrap()
        };
        checked(g, action);
    }
    panic!("paid Taskmaster first-main trigger did not resolve");
}
fn copy_tokens(g: &Game) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.is_copy_token())
        .map(|(id, _)| id as CardId)
        .collect()
}
fn token_count(g: &Game, definition: Card) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.is_token() && o.card == definition)
        .count()
}
fn replacement_case(replacer: &str) {
    let definition = Card::from_name(replacer).unwrap();
    let (replacement, additive) = match definition.replacement_effects()[0] {
        ReplacementSpec::AddArtifactToken(token) => (token, true),
        ReplacementSpec::ReplaceArtifactTokens(token) => (token, false),
        _ => panic!("expected real artifact-token replacement"),
    };
    for p in [Player::P0, Player::P1] {
        for blueprint in ["Three Tree Mascot", "Barkform Harvester"] {
            // Positive control: the same admitted artifact creature's direct
            // copy really IS an artifact and must receive the replacement.
            let mut positive = setup(p);
            paid(&mut positive, p, replacer, &[]);
            let original = paid(&mut positive, p, blueprint, &[]);
            assert_ne!(positive.current_kinds(original) & 16, 0);
            paid(
                &mut positive,
                p,
                "Self-Reflection",
                &[Target::Card(original)],
            );
            assert_eq!(token_count(&positive, replacement), 1);
            assert_eq!(copy_tokens(&positive).len(), usize::from(additive));

            // Taskmaster's printed exception replaces the copied card types
            // with Creature. The flat blueprint's printed Artifact must not
            // cause either artifact-only token replacement to match.
            let mut g = setup(p);
            paid(&mut g, p, replacer, &[]);
            let task = paid(&mut g, p, TASK, &[]);
            let original = paid(&mut g, p, blueprint, &[]);
            next_main(&mut g, p);
            first_main_copy(&mut g, task, original);
            assert_eq!(g.current_kinds(task) & 17, 1);
            assert_eq!(g.copiable_recipe(g.object_ref(task)).kinds() & 17, 1);
            paid(&mut g, p, "Self-Reflection", &[Target::Card(task)]);
            assert_eq!(
                token_count(&g, replacement),
                0,
                "{replacer} must classify the Creature-only recipe, not {blueprint}'s printed Artifact"
            );
            let copies = copy_tokens(&g);
            assert_eq!(copies.len(), 1);
            let copy = g.object_view(copies[0]);
            assert_eq!(copy.name(), TASK);
            assert_eq!(copy.kinds & 17, 1);
        }
    }
}
#[test]
fn msh_copy_identity_taskmaster_creature_only_does_not_add_worldwalker_map() {
    replacement_case("Worldwalker Helm");
}
#[test]
fn msh_copy_identity_taskmaster_creature_only_is_not_replaced_by_draconic_visitor() {
    replacement_case("Draconic Visitor");
}
fn paid_monica_transform(g: &mut Game, p: Player, source: CardId) {
    priority(g, p);
    g.state.players[p.index()].mana = [2, 0, 0, 1, 0, 2];
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    for _ in 0..10 {
        if g.state.announcing.is_none() {
            break;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, action);
    }
    assert!(g.state.announcing.is_none());
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle_keeping_new_copy(g);
}
#[test]
fn msh_copy_identity_token_of_single_faced_taskmaster_copying_monica_cannot_transform() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let task = paid(&mut g, p, TASK, &[]);
        let monica = paid(&mut g, p, "Monica Rambeau", &[]);
        next_main(&mut g, p);
        first_main_copy(&mut g, task, monica);
        assert!(g.object_view(task).face.is_none());
        paid(&mut g, p, "Molten Duplication", &[Target::Card(task)]);
        let copies = copy_tokens(&g);
        assert_eq!(copies.len(), 1);
        let copy = copies[0];
        let before = g.object_ref(copy);
        assert!(g.object_view(copy).is_token());
        assert_eq!(g.object_view(copy).name(), TASK);
        // Paying an inherited transform activation is legal. Its effect cannot
        // transform a token copied from a physically single-faced permanent.
        paid_monica_transform(&mut g, p, copy);
        let after = g.object_view(copy);
        assert_eq!(after.reference, before);
        assert!(
            after.face.is_none(),
            "CR707.8a does not give Taskmaster's token a back face"
        );
        assert_eq!((after.power, after.toughness), (3, 3));
        assert!(!after.keywords.contains(Keywords::HEXPROOF));
    }
}
#[test]
fn msh_copy_identity_token_of_actual_monica_retains_double_faces_and_transforms() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let monica = paid(&mut g, p, "Monica Rambeau", &[]);
        paid(&mut g, p, "Molten Duplication", &[Target::Card(monica)]);
        let copies = copy_tokens(&g);
        assert_eq!(copies.len(), 1);
        let copy = copies[0];
        assert!(!g.object_view(copy).face.unwrap().back);
        paid_monica_transform(&mut g, p, copy);
        let after = g.object_view(copy);
        assert!(after.face.unwrap().back);
        assert_eq!(after.name(), "Photon, Living Light");
        assert_eq!((after.power, after.toughness), (4, 4));
        assert!(after.keywords.contains(Keywords::HEXPROOF));
    }
}
