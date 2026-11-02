//! Positive-kind public hand choices; no ordinary card admission is needed.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const TARGET_OPPONENT: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Player {
    opponent_only: true,
}));
const fn selection(program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TARGET_OPPONENT,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[
        selection(&[Instruction::RevealTargetHandChoose {
            kinds: 4 | 8,
            exile: false,
        }]),
        selection(&[Instruction::RevealTargetHandChoose {
            kinds: 1 | 16,
            exile: true,
        }]),
    ],
    ..CardRules::EMPTY
};
const DISCARD_THEN_DRAW: CardRules = CardRules {
    effect: Effect::Spell(
        TARGET_OPPONENT,
        &[
            Instruction::RevealTargetHandChoose {
                kinds: 1 | 16,
                exile: false,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
const REDIRECTED_ENTRY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    entry: EntryTapped::PayLife(2),
    extras: &CardExtras {
        zone_replacement: Some(ZoneReplacementRule::OpponentDiscardToBattlefield),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn wait_choice(g: &mut Game) {
    for _ in 0..12 {
        if matches!(g.state.pending, Pending::HandCardChoice { .. }) {
            return;
        }
        assert!(!g.state.stack.is_empty() || g.state.resolving.is_some());
        act(g, Action::Pass);
    }
    panic!("revealed hand choice missing");
}
fn activate(g: &mut Game, source: CardId, exile: bool) {
    let victim = g.state.active.other();
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: u16::from(exile),
            counter_payment: Counters::default(),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Player(victim),
        },
    );
    pay(g);
}
fn pick(g: &mut Game, card: CardId) {
    let object = g.object_ref(card);
    act(
        g,
        Action::ChooseHandCard {
            object: Some(object),
        },
    );
}
fn revealed_to_both(g: &Game, objects: &[ObjectRef]) {
    for viewer in [Player::P0, Player::P1] {
        let observation = g.observe(viewer);
        for object in objects {
            assert!(
                observation.known_hand.contains(object),
                "{viewer:?} misses {object:?}"
            );
            assert!(observation.objects.iter().any(|o| o.reference == *object));
        }
    }
}

#[test]
fn positive_unions_offer_each_artifact_creature_instant_and_sorcery_once() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    for p in [Player::P0, Player::P1] {
        for (exile, names) in [
            (false, &["Shock", "Divination"][..]),
            (true, &["Gilded Lotus", "Bear Cub", "Pactdoll Terror"][..]),
        ] {
            for chosen in 0..names.len() {
                let mut g = scenario(p);
                let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
                let candidates: Vec<_> = names
                    .iter()
                    .map(|name| put(&mut g, p.other(), name, Zone::Hand))
                    .collect();
                let wrong = put(
                    &mut g,
                    p.other(),
                    if exile { "Shock" } else { "Bear Cub" },
                    Zone::Hand,
                );
                let land = put(&mut g, p.other(), "Mountain", Zone::Hand);
                let own_hand = put(&mut g, p, names[0], Zone::Hand);
                assert!(!g.observe(p).objects.iter().any(|o| o.id == candidates[0]));
                activate(&mut g, source, exile);
                wait_choice(&mut g);
                let all: Vec<_> = candidates
                    .iter()
                    .chain([&wrong, &land])
                    .map(|&id| g.object_ref(id))
                    .collect();
                revealed_to_both(&g, &all);
                let observation = g.observe(p);
                let choice = observation.hand_choice.as_ref().unwrap();
                assert_eq!(choice.actor, p);
                assert_eq!(choice.victim, Some(p.other()));
                assert_eq!(choice.required, 1);
                assert!(!choice.optional);
                assert!(
                    choice.inspected_cards.is_none(),
                    "these are publicly revealed cards"
                );
                assert_eq!(
                    choice.candidates.len(),
                    candidates.len(),
                    "artifact creatures appear once"
                );
                for &id in &candidates {
                    assert!(choice.candidates.contains(&g.object_ref(id)));
                }
                assert!(
                    g.observe(p.other()).hand_choice.is_none(),
                    "only the choosing player owns the decision"
                );
                reject(&mut g, Action::ChooseHandCard { object: None });
                for id in [wrong, land, own_hand] {
                    let object = g.object_ref(id);
                    reject(
                        &mut g,
                        Action::ChooseHandCard {
                            object: Some(object),
                        },
                    );
                }
                let saved = g.snapshot();
                let revision = g.revision();
                assert_eq!(
                    g.apply(
                        p.other(),
                        Action::ChooseHandCard {
                            object: Some(g.object_ref(candidates[chosen]))
                        }
                    ),
                    Err(Error::WrongActor)
                );
                assert_eq!(g.snapshot(), saved);
                assert_eq!(g.revision(), revision);
                pick(&mut g, candidates[chosen]);
                settle(&mut g);
                for (index, &id) in candidates.iter().enumerate() {
                    assert_eq!(
                        g.object_view(id).zone,
                        if index != chosen {
                            Zone::Hand
                        } else if exile {
                            Zone::Exile
                        } else {
                            Zone::Graveyard
                        }
                    );
                }
                assert_eq!(
                    g.state.players[p.other().index()].cards_discarded_this_turn,
                    u64::from(!exile)
                );
                assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 0);
                assert_eq!(g.object_view(wrong).zone, Zone::Hand);
                assert_eq!(g.object_view(land).zone, Zone::Hand);
            }
        }
    }
}

#[test]
fn no_matching_card_still_reveals_the_whole_hand_without_an_optional_choice() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    for p in [Player::P0, Player::P1] {
        for exile in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let wrong = put(
                &mut g,
                p.other(),
                if exile { "Divination" } else { "Gilded Lotus" },
                Zone::Hand,
            );
            let land = put(&mut g, p.other(), "Mountain", Zone::Hand);
            let hand = [g.object_ref(wrong), g.object_ref(land)];
            activate(&mut g, source, exile);
            settle(&mut g);
            revealed_to_both(&g, &hand);
            assert!(g.observe(p).hand_choice.is_none());
            assert_eq!(g.object_view(wrong).zone, Zone::Hand);
            assert_eq!(g.object_view(land).zone, Zone::Hand);
            assert_eq!(
                g.state.players[p.other().index()].cards_discarded_this_turn,
                0
            );
        }
    }
}

#[test]
fn captured_chooser_and_unlinked_exile_survive_source_departure_and_reentry() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let card = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        activate(&mut g, source, true);
        // The effect exists independently before its public hand decision starts.
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        g.state.objects[source as usize].controller = p.other();
        wait_choice(&mut g);
        assert_eq!(g.actor(), Some(p));
        pick(&mut g, card);
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Exile);
        assert_eq!(g.object_view(card).owner, p.other());
        g.move_object(source, Zone::Exile);
        assert_eq!(
            g.object_view(card).zone,
            Zone::Exile,
            "there is no leave-source return"
        );
        assert!(
            g.state
                .linked_choices
                .as_ref()
                .is_none_or(|c| c.links.is_empty() && c.null_cast_links.is_empty())
        );
    }
}

#[test]
fn frozen_candidate_set_rejects_a_reentered_hand_card_and_an_unseen_new_card() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let old = put(&mut g, Player::P1, "Gilded Lotus", Zone::Hand);
    let remaining = put(&mut g, Player::P1, "Bear Cub", Zone::Hand);
    activate(&mut g, source, true);
    wait_choice(&mut g);
    let old_ref = g.object_ref(old);
    g.move_object(old, Zone::Exile);
    g.move_object(old, Zone::Hand);
    let new_ref = g.object_ref(old);
    let unseen = put(&mut g, Player::P1, "Bear Cub", Zone::Hand);
    for object in [old_ref, new_ref, g.object_ref(unseen)] {
        reject(
            &mut g,
            Action::ChooseHandCard {
                object: Some(object),
            },
        );
    }
    assert!(!g.observe(Player::P0).objects.iter().any(|o| o.id == unseen));
    pick(&mut g, remaining);
    settle(&mut g);
    assert_eq!(g.object_view(old).zone, Zone::Hand);
    assert_eq!(g.object_view(unseen).zone, Zone::Hand);
    assert_eq!(g.object_view(remaining).zone, Zone::Exile);
}

#[test]
fn public_selection_is_owned_across_raw_index_parent_disposal_and_restore() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    for exile in [false, true] {
        let mut parent = scenario(Player::P1);
        let source = put(&mut parent, Player::P1, "Grizzly Bears", Zone::Battlefield);
        let names = if exile {
            ["Bear Cub", "Gilded Lotus"]
        } else {
            ["Shock", "Divination"]
        };
        let first = put(&mut parent, Player::P0, names[0], Zone::Hand);
        let second = put(&mut parent, Player::P0, names[1], Zone::Hand);
        activate(&mut parent, source, exile);
        wait_choice(&mut parent);
        let saved = parent.snapshot();
        let action = Action::ChooseHandCard {
            object: Some(parent.object_ref(first)),
        };
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let revision = indexed.revision();
        act(&mut raw, action);
        indexed.apply_index(Player::P1, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        drop(parent);
        pick(&mut indexed, second);
        settle(&mut raw);
        settle(&mut indexed);
        let moved = if exile { Zone::Exile } else { Zone::Graveyard };
        assert_eq!(raw.object_view(first).zone, moved);
        assert_eq!(raw.object_view(second).zone, Zone::Hand);
        assert_eq!(indexed.object_view(first).zone, Zone::Hand);
        assert_eq!(indexed.object_view(second).zone, moved);
    }
}

#[test]
fn discard_uses_the_opponent_cause_and_waits_for_owned_replacement_entry() {
    let _scope = fixture_rules::activate(&[
        (Card::Divination, &DISCARD_THEN_DRAW),
        (Card::from_name("Bear Cub").unwrap(), &REDIRECTED_ENTRY),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(p.other())));
        wait_choice(&mut g);
        pick(&mut g, card);
        assert!(matches!(g.state.pending, Pending::EntryLife { player } if player == p.other()));
        assert_eq!(g.object_view(card).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        let saved = g.snapshot();
        let mut branch = g.clone();
        act(&mut g, Action::ChooseEntryLife { pay: true });
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Battlefield);
        assert!(!g.object_view(card).tapped);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(
            g.state.players[p.other().index()].cards_discarded_this_turn,
            1
        );
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        branch.restore(&saved).unwrap();
        drop(g);
        act(&mut branch, Action::ChooseEntryLife { pay: false });
        settle(&mut branch);
        assert_eq!(branch.object_view(card).zone, Zone::Battlefield);
        assert!(branch.object_view(card).tapped);
        assert_eq!(branch.state.players[p.other().index()].life, 20);
        assert_eq!(branch.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}

#[test]
fn prevented_discard_or_exile_finishes_the_chosen_card_effect_without_reselection() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|r| r.set(None));
        }
    }
    let _reset = Reset;
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    for exile in [false, true] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let card = put(
            &mut g,
            Player::P1,
            if exile { "Bear Cub" } else { "Shock" },
            Zone::Hand,
        );
        activate(&mut g, source, exile);
        wait_choice(&mut g);
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|r| r.set(Some(g.object_ref(card))));
        pick(&mut g, card);
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Hand);
        assert_eq!(g.state.players[1].cards_discarded_this_turn, 0);
        assert!(g.observe(Player::P0).hand_choice.is_none());
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|r| r.set(None));
    }
}
