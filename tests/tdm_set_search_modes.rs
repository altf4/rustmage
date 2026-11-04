//! Focused F08/F12 acceptance: actual Siege and Lotuslight definitions.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

// Engine compatibility only: no actual TDM definition is overlaid. Opt supplies
// its ordinary printed U/instant header to a narrow generic ability copier.
const ABILITY_COPIER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::CopyTargetAbility { group: 0 }],
    ),
    ..CardRules::EMPTY
};

fn frame(g: &Game, p: Player) -> serde_json::Value {
    serde_json::to_value(g.observe(p)).unwrap()
}
fn frames(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| frame(g, p))
}
fn ready(g: &mut Game, done: impl Fn(&Game) -> bool) {
    for _ in 0..128 {
        if done(g) {
            return;
        }
        match g.state.pending {
            Pending::Priority => act(g, Action::Pass),
            Pending::OrderTriggers { .. } => choose(g, |a| matches!(a, Action::PutTrigger { .. })),
            Pending::Retarget { .. } => act(g, Action::FinishStackTargets),
            _ => panic!("explicit set choice required: {:?}", g.state.pending),
        }
    }
    panic!("128-action focused set scenario budget exhausted");
}
fn drain(g: &mut Game) {
    ready(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}
fn normalized_restored_frame(
    g: &Game,
    p: Player,
    expected: &serde_json::Value,
) -> serde_json::Value {
    let mut restored = frame(g, p);
    restored["revision"] = expected["revision"].clone();
    restored
}

fn lotus_world(p: Player, names: [&str; 4]) -> (Game, [CardId; 4], CardId, CardId) {
    let mut g = scenario(p);
    let cards = names.map(|name| put(&mut g, p, name, Zone::Library));
    let source = put(&mut g, p, "Lotuslight Dancers", Zone::Hand);
    let foreign = put(&mut g, p.other(), "Duress", Zone::Library);
    g.state.players[p.index()].mana = [0, 1, 1, 0, 1, 2];
    (g, cards, source, foreign)
}
fn pay_lotus(g: &mut Game, p: Player, source: CardId) {
    cast(g, source, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    ready(g, |g| {
        matches!(g.state.pending, Pending::EffectSearch { .. })
    });
    assert_eq!(stats(g, source), (3, 6));
    assert!(g.keywords(source).contains(Keywords::LIFELINK));
    assert!(g.observe(p).search_choice.is_some());
    assert!(g.observe(p.other()).search_choice.is_none());
}

#[test]
fn f12_actual_paid_lotus_search_paired_hidden_qualities_remain_private_and_owned() {
    for p in [Player::P0, Player::P1] {
        let (mut parent, cards, source, foreign) = lotus_world(
            p,
            [
                "Duress",
                "Llanowar Elves",
                "Divination",
                "Auroral Procession",
            ],
        );
        let (mut hidden, other_cards, other_source, other_foreign) =
            lotus_world(p, ["Opt", "Duress", "Giant Growth", "Shock"]);
        assert_eq!(cards, other_cards);
        assert_eq!((source, foreign), (other_source, other_foreign));
        assert_eq!(
            frames(&parent),
            frames(&hidden),
            "unsearched libraries are hidden from both seats"
        );
        pay_lotus(&mut parent, p, source);
        pay_lotus(&mut hidden, p, other_source);
        assert_eq!(frame(&parent, p.other()), frame(&hidden, p.other()));
        assert_ne!(
            frame(&parent, p),
            frame(&hidden, p),
            "the search controller sees different private qualities"
        );
        let multi = Action::ChooseSearch { card: cards[3] };
        assert!(parent.legal_actions().contains(&multi));
        assert!(!hidden.legal_actions().contains(&multi));
        let before_hidden = frames(&hidden);
        reject(&mut hidden, multi);
        assert_eq!(frames(&hidden), before_hidden);
        for world in [&mut parent, &mut hidden] {
            let before = frames(world);
            reject(world, Action::ChooseSearch { card: foreign });
            assert_eq!(frames(world), before);
        }
        assert_eq!(frame(&parent, p.other()), frame(&hidden, p.other()));
        let saved = parent.snapshot();
        let saved_frames = frames(&parent);
        let revision = parent.revision();
        let first = Action::ChooseSearch { card: cards[0] };
        let index = parent
            .legal_actions()
            .iter()
            .position(|a| *a == first)
            .unwrap();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let mut sibling = parent.clone();
        drop(parent);
        assert_eq!(raw.apply(p.other(), first.clone()), Err(Error::WrongActor));
        assert_eq!(raw.snapshot(), saved);
        assert_eq!(frames(&raw), saved_frames);
        act(&mut raw, first.clone());
        indexed.apply_index(p, revision, index).unwrap();
        act(&mut hidden, first.clone());
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(frames(&raw), frames(&indexed));
        assert_eq!(frame(&raw, p.other()), frame(&hidden, p.other()));
        for world in [&mut raw, &mut hidden] {
            let before = frames(world);
            reject(world, first.clone());
            assert_eq!(frames(world), before);
        }
        for &card in &cards[1..3] {
            let action = Action::ChooseSearch { card };
            act(&mut raw, action.clone());
            act(&mut hidden, action);
            assert_eq!(frame(&raw, p.other()), frame(&hidden, p.other()));
            assert_eq!(raw.object_view(card).zone, Zone::Library);
        }
        // Even the maximum selection remains private until explicit commitment.
        assert!(raw.legal_actions().contains(&Action::FinishSearch));
        assert_eq!(frame(&raw, p.other()), frame(&hidden, p.other()));
        for &card in &cards[1..3] {
            act(&mut indexed, Action::ChooseSearch { card });
        }
        let rng = raw.state.rng;
        for child in [&mut raw, &mut indexed, &mut hidden, &mut sibling] {
            act(child, Action::FinishSearch);
            drain(child);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_ne!(raw.state.rng, rng);
        for &card in &cards[..3] {
            assert_eq!(raw.object_view(card).zone, Zone::Graveyard);
            assert_eq!(hidden.object_view(card).zone, Zone::Graveyard);
            assert_eq!(sibling.object_view(card).zone, Zone::Library);
        }
        assert_eq!(raw.object_view(cards[3]).zone, Zone::Library);
        assert_eq!(raw.object_view(foreign).zone, Zone::Library);
        assert_eq!(raw.library_card_count(p), 9);
        assert_eq!(sibling.library_card_count(p), 12);
        assert_ne!(
            frame(&raw, p.other()),
            frame(&hidden, p.other()),
            "committed graveyard identities are public"
        );
        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        for (seat, expected) in [Player::P0, Player::P1].into_iter().zip(&saved_frames) {
            assert_eq!(
                normalized_restored_frame(&indexed, seat, expected),
                *expected
            );
        }
        for &card in &cards[..3] {
            act(&mut indexed, Action::ChooseSearch { card });
        }
        act(&mut indexed, Action::FinishSearch);
        drain(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        for seat in [Player::P0, Player::P1] {
            let expected = frame(&raw, seat);
            assert_eq!(
                normalized_restored_frame(&indexed, seat, &expected),
                expected
            );
        }
    }
}

fn siege(g: &mut Game, p: Player, mode: u16) -> CardId {
    g.state.players[p.index()].mana = [0, 1, 0, 1, 0, 1];
    let id = put(g, p, "Frostcliff Siege", Zone::Hand);
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    ready(g, |g| matches!(g.state.pending, Pending::EntryValue { .. }));
    act(
        g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::NamedMode(mode),
        },
    );
    drain(g);
    id
}
fn hits(g: &mut Game, sources: &[CardId], target: Player) {
    let batch = sources
        .iter()
        .map(|&id| DamageEvent {
            source: g.damage_source(id),
            target: Target::Player(target),
            amount: 1,
            combat: true,
            preventable: true,
        })
        .collect::<Vec<_>>();
    g.apply_damage_batch(&batch);
    g.give_priority();
}

#[test]
fn f08_actual_siege_trigger_copy_keeps_frozen_mode_event_and_each_controller_after_source_changes()
{
    let _compatibility =
        fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &ABILITY_COPIER)]);
    for p in [Player::P0, Player::P1] {
        for change in 0..3 {
            let mut g = scenario(p);
            let source = siege(&mut g, p, 0);
            let original = g.object_ref(source);
            let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            hits(&mut g, &[a, b], p.other());
            ready(&mut g, |g| {
                g.state.pending == Pending::Priority && g.state.stack.len() == 1
            });
            let trigger = g.state.stack[0].id;
            assert!(matches!(
                g.state.stack[0].kind,
                StackKind::TriggeredAbility { .. }
            ));
            assert_eq!(g.state.stack[0].controller, p);
            assert_eq!(g.state.stack[0].source.unwrap().object, original);
            if g.actor() != Some(p.other()) {
                act(&mut g, Action::Pass);
            }
            let copy_spell = put(&mut g, p.other(), "Opt", Zone::Hand);
            g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 0];
            cast(&mut g, copy_spell, Some(Target::Stack(trigger)));
            assert_eq!(g.state.players[p.other().index()].mana, [0; 6]);
            ready(&mut g, |g| {
                g.state.pending == Pending::Priority && g.state.stack.iter().any(|s| s.is_copy)
            });
            assert_eq!(g.state.stack.len(), 2);
            let copy = g.state.stack.iter().find(|s| s.is_copy).unwrap();
            assert_eq!(copy.controller, p.other());
            assert_eq!(copy.source.unwrap().object, original);
            // A real source-control operation and normal copy/entry machinery
            // change future availability; the two existing packets remain owned.
            g.record_control(original, p.other(), None);
            match change {
                0 => (),
                1 => g.apply_temporary_copy(
                    original,
                    CopyRecipe::new(Card::from_name("Short Bow").unwrap()),
                    CopyDeadline::EndOfTurn,
                ),
                _ => {
                    g.move_object(source, Zone::Exile);
                    let mut movement = g.entry_move(source, Zone::Battlefield, false);
                    movement.controller = p.other();
                    g.stage_entries(vec![movement], None);
                    assert!(g.advance_entries());
                    assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
                    act(
                        &mut g,
                        Action::ChooseEntryValue {
                            value: EntryChoiceValue::NamedMode(1),
                        },
                    );
                    assert_ne!(g.object_ref(source), original);
                    assert_eq!(stats(&g, enemy), (3, 2));
                    assert!(
                        g.keywords(enemy)
                            .contains(Keywords::TRAMPLE.union(Keywords::HASTE))
                    );
                }
            }
            drain(&mut g);
            assert_eq!(
                g.hand_count(p),
                1,
                "original keeps its trigger-time controller"
            );
            assert_eq!(
                g.hand_count(p.other()),
                1,
                "copy draws for its own controller despite old source facts"
            );
            hits(&mut g, &[a], p.other());
            drain(&mut g);
            assert_eq!(g.hand_count(p), 1);
            assert_eq!(g.hand_count(p.other()), 1);
            hits(&mut g, &[enemy], p);
            drain(&mut g);
            assert_eq!(g.hand_count(p), 1);
            assert_eq!(
                g.hand_count(p.other()),
                if change == 0 { 2 } else { 1 },
                "future events use current controller/form/new Temur mode"
            );
        }
    }
}

#[test]
fn f08_actual_temur_siege_never_creates_an_ineligible_jeskai_packet_to_copy() {
    let _compatibility =
        fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &ABILITY_COPIER)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        siege(&mut g, p, 1);
        let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, creature), (3, 2));
        hits(&mut g, &[creature], p.other());
        drain(&mut g);
        assert_eq!(g.hand_count(p), 0);
        assert!(g.state.stack.is_empty());
        let copier = put(&mut g, p, "Opt", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        let before = frames(&g);
        reject(&mut g, Action::AnnounceCast { card: copier });
        assert_eq!(frames(&g), before);
    }
}
