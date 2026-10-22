//! Paid ECL cards, with owned branches, observation equality, replay and rejection atomicity.
use super::tests::{scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    g.state.players[p.index()].mana = [30; 6];
    g
}
fn checked(g: &mut Game, a: Action) {
    let p = g.actor().unwrap();
    let revision = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|v| *v == a)
        .unwrap_or_else(|| panic!("illegal {a:?}; {:?}", g.legal_actions()));
    let before = g.snapshot();
    assert_eq!(g.apply(p.other(), a.clone()), Err(Error::WrongActor));
    assert_eq!(g.snapshot(), before);
    let mut child = {
        let parent = g.clone();
        parent.clone()
    };
    child.apply_index(p, revision, index).unwrap();
    g.apply(p, a.clone()).unwrap();
    assert_eq!(g.snapshot(), child.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap(),
            serde_json::to_value(child.observe(viewer)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(g.apply_index(p, revision, index), Err(Error::StaleRevision));
    g.apply(p, a).unwrap();
    assert_eq!(g.snapshot(), after);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), before);
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets)))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        checked(g, a);
    }
    panic!("unsettled {:?}", g.state.pending);
}
fn cast(g: &mut Game, id: CardId) {
    checked(g, Action::AnnounceCast { card: id });
    settle(g);
}
#[test]
fn boulder_dash_requires_distinct_targets_and_resolves_legal_half() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let spell = named(&mut g, p, "Boulder Dash", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        // A departed second target does not invalidate the first.
        g.move_object(victim, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.objects[victim as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn dawnhand_eulogist_mills_before_checking_any_elf_card_and_has_menace() {
    for p in [Player::P0, Player::P1] {
        for elf in [false, true] {
            let mut g = setup(p);
            let top = named(
                &mut g,
                p,
                if elf { "Llanowar Elves" } else { "Forest" },
                Zone::Library,
            );
            let id = named(&mut g, p, "Dawnhand Eulogist", Zone::Hand);
            let library = g.state.players[p.index()].library.len();
            cast(&mut g, id);
            assert_eq!(g.state.players[p.index()].library.len(), library - 3);
            assert_eq!(g.state.objects[top as usize].zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].life, if elf { 22 } else { 20 });
            assert_eq!(
                g.state.players[p.other().index()].life,
                if elf { 18 } else { 20 }
            );
            assert!(g.keywords(id).contains(Keywords::MENACE));
        }
    }
}
#[test]
fn heirloom_auntie_enters_shrunken_and_surveys_each_other_owned_death() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Heirloom Auntie", Zone::Hand);
        cast(&mut g, id);
        assert_eq!(g.counter_count(id, CounterKind::MinusOne), 2);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.move_object(enemy, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::MinusOne), 2);
        for expected in [1, 0, 0] {
            let friend = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            g.move_object(friend, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(g.counter_count(id, CounterKind::MinusOne), expected);
        }
    }
}
fn advance(g: &mut Game) {
    let actions = g.legal_actions();
    let a = if g.state.pending == Pending::Priority {
        Action::Pass
    } else {
        actions
            .iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets)))
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone()
    };
    checked(g, a);
}
fn until_blight(g: &mut Game) {
    for _ in 0..100 {
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Blight { .. }))
        {
            return;
        }
        advance(g);
    }
    panic!("missing blight choice");
}
#[test]
fn four_resolution_blight_cards_pay_decline_and_preserve_private_opponent_hand() {
    for p in [Player::P0, Player::P1] {
        for (name, amount) in [
            ("Blighted Blackthorn", 2),
            ("Dream Seizer", 1),
            ("Sourbread Auntie", 2),
            ("Scuzzback Scrounger", 1),
        ] {
            for pay in [false, true] {
                let mut g = setup(p);
                let hidden = named(&mut g, p.other(), "Lightning Bolt", Zone::Hand);
                let id = named(&mut g, p, name, Zone::Hand);
                checked(&mut g, Action::AnnounceCast { card: id });
                if name == "Scuzzback Scrounger" {
                    settle(&mut g);
                    g.begin_step(Step::Main1);
                    g.prepare_triggers();
                }
                until_blight(&mut g);
                let other = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
                let invalid = g.object_ref(other);
                reject(&mut g, Action::Blight { object: invalid });
                assert!(!g.observe(p).objects.iter().any(|o| o.id == hidden));
                let hand_before = g.observe(p).players[p.index()].hand_count;
                if pay {
                    let object = g.object_ref(id);
                    checked(&mut g, Action::Blight { object });
                } else {
                    checked(&mut g, Action::DeclineEffectPayment);
                }
                settle(&mut g);
                assert_eq!(
                    g.counter_count(id, CounterKind::MinusOne),
                    if pay { amount } else { 0 },
                    "{name}"
                );
                let tokens = g
                    .observe(p)
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
                    .count();
                match name {
                    "Blighted Blackthorn" => {
                        assert_eq!(g.state.players[p.index()].life, if pay { 19 } else { 20 });
                        assert_eq!(
                            g.observe(p).players[p.index()].hand_count,
                            hand_before + usize::from(pay)
                        );
                    }
                    "Dream Seizer" => {
                        assert_eq!(
                            g.state.objects[hidden as usize].zone,
                            if pay { Zone::Graveyard } else { Zone::Hand }
                        );
                        assert!(g.keywords(id).contains(Keywords::FLYING));
                    }
                    "Sourbread Auntie" => {
                        assert_eq!(tokens, if pay { 2 } else { 0 });
                        for token in g.observe(p).objects.iter().filter(|o| o.is_token()) {
                            assert!(token.has_subtype("Goblin"));
                            assert_eq!((token.power, token.toughness), (1, 1));
                            assert_eq!(token.card.colors(), 12);
                        }
                    }
                    "Scuzzback Scrounger" => assert_eq!(tokens, usize::from(pay)),
                    _ => unreachable!(),
                }
            }
        }
    }
}
fn targeted(g: &mut Game, id: CardId, mode: Option<u16>, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card: id });
    if let Some(mode) = mode {
        checked(g, Action::ChooseMode { mode });
    }
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    settle(g);
}
#[test]
fn unbury_modes_restrict_owner_number_and_shared_type_including_changeling() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
        let changeling = named(&mut g, p, "Graveshifter", Zone::Graveyard);
        let enemy = named(&mut g, p.other(), "Llanowar Elves", Zone::Graveyard);
        let id = named(&mut g, p, "Unbury", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: id });
        checked(&mut g, Action::ChooseMode { mode: 1 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(elf),
            },
        );
        reject(&mut g, Action::FinishTargets);
        for invalid in [bear, enemy] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(invalid),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(changeling),
            },
        );
        settle(&mut g);
        assert_eq!(g.state.objects[elf as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[changeling as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
        g.state.players[p.index()].mana = [30; 6];
        let id = named(&mut g, p, "Unbury", Zone::Hand);
        targeted(&mut g, id, Some(0), &[Target::Card(bear)]);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Hand);
    }
}
#[test]
fn nameless_inversion_has_changeling_and_erases_types_until_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Nameless Inversion", Zone::Hand);
        assert!(g.state.objects[id as usize].card.has_subtype("Elf"));
        let victim = named(&mut g, p.other(), "Graveshifter", Zone::Battlefield);
        // Keep the target alive so both subtype and cleanup behavior are observable.
        g.state.objects[victim as usize].counters.plus_one = 3;
        targeted(&mut g, id, None, &[Target::Card(victim)]);
        assert_eq!(
            g.stats(victim),
            Stats {
                power: 8,
                toughness: 2
            }
        );
        assert!(!g.has_subtype(victim, "Elf"));
        assert!(!g.has_subtype(victim, "Shapeshifter"));
        g.finish_cleanup();
        assert_eq!(
            g.stats(victim),
            Stats {
                power: 5,
                toughness: 5
            }
        );
        assert!(g.has_subtype(victim, "Elf"));
    }
}
#[test]
fn goatnap_untaps_grants_haste_and_only_goats_receive_extra_power() {
    for p in [Player::P0, Player::P1] {
        for goat in [false, true] {
            let mut g = setup(p);
            let victim = named(
                &mut g,
                p.other(),
                if goat {
                    "Graveshifter"
                } else {
                    "Grizzly Bears"
                },
                Zone::Battlefield,
            );
            g.state.objects[victim as usize].tapped = true;
            let id = named(&mut g, p, "Goatnap", Zone::Hand);
            targeted(&mut g, id, None, &[Target::Card(victim)]);
            assert_eq!(g.state.objects[victim as usize].controller, p);
            assert!(!g.state.objects[victim as usize].tapped);
            assert!(g.keywords(victim).contains(Keywords::HASTE));
            assert_eq!(g.stats(victim).power, if goat { 5 } else { 2 });
            g.finish_cleanup();
            assert_eq!(g.state.objects[victim as usize].controller, p.other());
            assert!(!g.keywords(victim).contains(Keywords::HASTE));
            assert_eq!(g.stats(victim).power, 2);
        }
    }
}
#[test]
fn creakwood_safewright_end_step_rechecks_both_intervening_conditions() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Creakwood Safewright", Zone::Hand);
        cast(&mut g, id);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        g.begin_step(Step::End);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::MinusOne), 3);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        g.begin_step(Step::End);
        g.prepare_triggers();
        assert!(!g.state.stack.is_empty() || !g.state.pending_triggers.is_empty());
        g.move_object(elf, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::MinusOne), 3);
        g.move_object(elf, Zone::Graveyard);
        g.begin_step(Step::End);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::MinusOne), 2);
        g.begin_step(Step::End);
        g.prepare_triggers();
        g.state.objects[id as usize].counters.minus_one = 0;
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::MinusOne), 0);
    }
}
#[test]
fn blighted_blackthorn_attack_trigger_can_blight_itself_again() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, "Blighted Blackthorn", Zone::Hand);
    checked(&mut g, Action::AnnounceCast { card: id });
    until_blight(&mut g);
    let object = g.object_ref(id);
    checked(&mut g, Action::Blight { object });
    settle(&mut g);
    g.state.objects[id as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(&mut g, Action::Attack { card: id });
    checked(&mut g, Action::FinishAttackers);
    until_blight(&mut g);
    let hand = g.observe(Player::P0).players[0].hand_count;
    checked(&mut g, Action::Blight { object });
    settle(&mut g);
    assert_eq!(g.counter_count(id, CounterKind::MinusOne), 4);
    assert_eq!(g.state.players[0].life, 18);
    assert_eq!(g.observe(Player::P0).players[0].hand_count, hand + 1);
}
#[test]
fn temporary_type_replacement_expires_to_an_older_permanent_replacement() {
    let mut g = setup(Player::P0);
    let victim = named(&mut g, Player::P1, "Graveshifter", Zone::Battlefield);
    g.state.objects[victim as usize].counters.plus_one = 3;
    let id = named(&mut g, Player::P0, "Nameless Inversion", Zone::Hand);
    checked(&mut g, Action::AnnounceCast { card: id });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    let packet = g.state.announcing.as_ref().unwrap().item.clone();
    g.set_creature_types(&packet, Subject::Targets, &["Werewolf"]);
    assert!(g.has_subtype(victim, "Werewolf"));
    assert!(!g.has_subtype(victim, "Elf"));
    settle(&mut g);
    assert!(!g.has_subtype(victim, "Werewolf"));
    g.finish_cleanup();
    assert!(g.has_subtype(victim, "Werewolf"));
    assert!(!g.has_subtype(victim, "Elf"));
}
fn activate_with_blight(g: &mut Game, id: CardId, witness: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    assert!(
        g.observe(g.actor().unwrap().other())
            .additional_cost
            .is_none()
    );
    let object = g.object_ref(witness);
    checked(g, Action::ChooseAdditionalCost { object });
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, pay);
}
#[test]
fn gristle_glutton_mandatory_blight_pays_before_forced_discard_and_draw() {
    for p in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Gristle Glutton", Zone::Hand);
            cast(&mut g, id);
            let activation = Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            };
            assert!(
                !g.legal_actions().contains(&activation),
                "summoning sickness"
            );
            g.state.objects[id as usize].summoning_sick = false;
            let discarded = discard.then(|| named(&mut g, p, "Forest", Zone::Hand));
            let library = g.state.players[p.index()].library.len();
            activate_with_blight(&mut g, id, id);
            assert!(g.state.objects[id as usize].tapped);
            assert_eq!(g.counter_count(id, CounterKind::MinusOne), 1);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                library - usize::from(discard)
            );
            if let Some(card) = discarded {
                assert_eq!(g.state.objects[card as usize].zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn sting_slinger_can_blight_its_tapped_source_and_costs_are_not_doubled_by_effect_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Sting-Slinger", Zone::Hand);
        cast(&mut g, id);
        g.state.objects[id as usize].summoning_sick = false;
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        activate_with_blight(&mut g, id, id);
        assert!(g.state.objects[id as usize].tapped);
        assert_eq!(g.counter_count(id, CounterKind::MinusOne), 1);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn sting_slinger_blight_witness_can_fund_mana_then_die_with_ability_on_stack() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Sting-Slinger", Zone::Hand);
        cast(&mut g, id);
        g.state.objects[id as usize].summoning_sick = false;
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let mountain = named(&mut g, p, "Mountain", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let invalid = g.object_ref(enemy);
        reject(&mut g, Action::ChooseAdditionalCost { object: invalid });
        let witness = g.object_ref(elf);
        checked(&mut g, Action::ChooseAdditionalCost { object: witness });
        checked(&mut g, Action::TapMana { card: elf });
        checked(&mut g, Action::TapMana { card: mountain });
        assert_eq!(
            g.counter_count(elf, CounterKind::MinusOne),
            0,
            "cost is not placed during announcement"
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        assert_eq!(g.state.objects[elf as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.stack.len(), 1);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn optional_resolution_blight_cost_is_not_doubled_by_doubling_season() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, "Doubling Season", Zone::Battlefield);
    let id = named(&mut g, Player::P0, "Blighted Blackthorn", Zone::Hand);
    checked(&mut g, Action::AnnounceCast { card: id });
    until_blight(&mut g);
    let object = g.object_ref(id);
    checked(&mut g, Action::Blight { object });
    settle(&mut g);
    assert_eq!(g.counter_count(id, CounterKind::MinusOne), 2);
    assert_eq!(g.state.players[0].life, 19);
}
#[test]
fn aunties_sentence_reveals_full_hand_but_discards_only_nonland_permanents() {
    for p in [Player::P0, Player::P1] {
        for has_permanent in [false, true] {
            let mut g = setup(p);
            let land = named(&mut g, p.other(), "Forest", Zone::Hand);
            let instant = named(&mut g, p.other(), "Shock", Zone::Hand);
            let creature =
                has_permanent.then(|| named(&mut g, p.other(), "Grizzly Bears", Zone::Hand));
            let id = named(&mut g, p, "Auntie's Sentence", Zone::Hand);
            assert!(!g.observe(p).objects.iter().any(|o| o.id == land));
            checked(&mut g, Action::AnnounceCast { card: id });
            checked(&mut g, Action::ChooseMode { mode: 0 });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            if let Some(creature) = creature {
                for _ in 0..100 {
                    if matches!(g.state.pending, Pending::RevealedDiscard { .. }) {
                        break;
                    }
                    advance(&mut g);
                }
                for excluded in [land, instant] {
                    let object = g.object_ref(excluded);
                    reject(&mut g, Action::ChooseRevealedDiscard { object });
                    assert!(g.observe(p).objects.iter().any(|o| o.id == excluded));
                }
                let object = g.object_ref(creature);
                checked(&mut g, Action::ChooseRevealedDiscard { object });
                settle(&mut g);
                assert_eq!(g.state.objects[creature as usize].zone, Zone::Graveyard);
            } else {
                settle(&mut g);
            }
            assert_eq!(g.state.objects[land as usize].zone, Zone::Hand);
            assert_eq!(g.state.objects[instant as usize].zone, Zone::Hand);
        }
    }
}
#[test]
fn aunties_sentence_second_mode_shrinks_any_creature_until_cleanup() {
    let mut g = setup(Player::P0);
    let creature = named(&mut g, Player::P1, "Heirloom Auntie", Zone::Battlefield);
    let id = named(&mut g, Player::P0, "Auntie's Sentence", Zone::Hand);
    targeted(&mut g, id, Some(1), &[Target::Card(creature)]);
    assert_eq!(
        g.stats(creature),
        Stats {
            power: 2,
            toughness: 2
        }
    );
    g.finish_cleanup();
    assert_eq!(
        g.stats(creature),
        Stats {
            power: 4,
            toughness: 4
        }
    );
}
#[test]
fn gloom_ripper_counts_own_elves_in_both_zones_with_optional_opponent_target() {
    for p in [Player::P0, Player::P1] {
        for optional in [false, true] {
            let mut g = setup(p);
            let own = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            named(&mut g, p, "Graveshifter", Zone::Graveyard);
            named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Heirloom Auntie", Zone::Battlefield);
            let id = named(&mut g, p, "Gloom Ripper", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: id });
            for _ in 0..100 {
                if matches!(g.state.pending, Pending::Targets { .. }) {
                    break;
                }
                advance(&mut g);
            }
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foe),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            if optional {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(foe),
                    },
                );
            } else {
                checked(&mut g, Action::FinishTargets);
            }
            settle(&mut g);
            assert_eq!(
                g.stats(own),
                Stats {
                    power: 4,
                    toughness: 1
                }
            );
            assert_eq!(g.stats(foe).toughness, if optional { 1 } else { 4 });
            g.finish_cleanup();
            assert_eq!(
                g.stats(own),
                Stats {
                    power: 1,
                    toughness: 1
                }
            );
            assert_eq!(g.stats(foe).toughness, 4);
        }
    }
}
#[test]
fn boggart_mischief_optional_blight_creates_goblins_and_drains_for_owned_goblin_deaths() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let witness = named(&mut g, p, "Goblin Piker", Zone::Battlefield);
            let id = named(&mut g, p, "Boggart Mischief", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: id });
            until_blight(&mut g);
            if pay {
                let object = g.object_ref(witness);
                checked(&mut g, Action::Blight { object });
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(
                g.observe(p)
                    .objects
                    .iter()
                    .filter(|o| o.controller == p && o.zone == Zone::Battlefield && o.is_token())
                    .count(),
                if pay { 2 } else { 0 }
            );
            assert_eq!(g.state.players[p.index()].life, if pay { 21 } else { 20 });
            assert_eq!(
                g.state.players[p.other().index()].life,
                if pay { 19 } else { 20 }
            );
            let opponent = named(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
            g.move_object(opponent, Zone::Graveyard);
            let non_goblin = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            g.move_object(non_goblin, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, if pay { 21 } else { 20 });
            let goblin = named(&mut g, p, "Goblin Piker", Zone::Battlefield);
            g.move_object(goblin, Zone::Graveyard);
            g.prepare_triggers();
            g.move_object(id, Zone::Exile);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, if pay { 22 } else { 21 });
            assert_eq!(
                g.state.players[p.other().index()].life,
                if pay { 18 } else { 19 }
            );
        }
        let mut g = setup(p);
        let id = named(&mut g, p, "Boggart Mischief", Zone::Hand);
        cast(&mut g, id);
        assert!(!g.observe(p).objects.iter().any(|o| o.is_token()));
    }
}
#[test]
fn gutsplitter_gang_can_pay_decline_or_lose_source_before_its_first_main_trigger() {
    for p in [Player::P0, Player::P1] {
        for branch in 0..3 {
            let mut g = setup(p);
            let id = named(&mut g, p, "Gutsplitter Gang", Zone::Hand);
            cast(&mut g, id);
            g.begin_step(Step::Main1);
            g.prepare_triggers();
            if branch == 2 {
                g.move_object(id, Zone::Exile);
            } else {
                until_blight(&mut g);
                if branch == 0 {
                    let object = g.object_ref(id);
                    checked(&mut g, Action::Blight { object });
                } else {
                    checked(&mut g, Action::DeclineEffectPayment);
                }
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if branch == 0 { 20 } else { 17 }
            );
            assert_eq!(
                g.counter_count(id, CounterKind::MinusOne),
                if branch == 0 { 2 } else { 0 }
            );
            g.begin_step(Step::Main2);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if branch == 0 { 20 } else { 17 }
            );
        }
    }
}
fn bloodletter(g: &mut Game, p: Player, creature: CardId) -> CardId {
    let id = named(g, p, "Barbed Bloodletter", Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    for _ in 0..100 {
        if matches!(g.state.pending, Pending::Targets { .. }) {
            break;
        }
        advance(g);
    }
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    settle(g);
    id
}
#[test]
fn barbed_bloodletter_flashes_attaches_and_grants_only_temporary_wither_then_equips() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.active = p.other();
        g.state.step = Step::BeginCombat;
        let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let second = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let equipment = bloodletter(&mut g, p, own);
        assert_eq!(
            g.state.objects[equipment as usize].attached_to,
            Some(g.object_ref(own))
        );
        assert_eq!(
            g.stats(own),
            Stats {
                power: 3,
                toughness: 4
            }
        );
        assert!(g.keywords(own).contains(Keywords::WITHER));
        let activation = Action::AnnounceActivate {
            card: equipment,
            ability: 0,
            counter_payment: Counters::default(),
        };
        assert!(
            !g.legal_actions().contains(&activation),
            "equip only as a sorcery"
        );
        g.finish_cleanup();
        assert!(!g.keywords(own).contains(Keywords::WITHER));
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        g.state.pending = Pending::Priority;
        g.state.players[p.index()].mana = [30; 6];
        checked(&mut g, activation);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(second),
            },
        );
        settle(&mut g);
        assert_eq!(
            g.stats(own),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert_eq!(
            g.stats(second),
            Stats {
                power: 3,
                toughness: 4
            }
        );
        assert!(
            !g.keywords(second).contains(Keywords::WITHER),
            "equip does not repeat the ETB grant"
        );
    }
}
#[test]
fn wither_noncombat_uses_counter_replacements_but_lifelink_uses_damage_amount() {
    for p in [Player::P0, Player::P1] {
        for doubling in [false, true] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            g.state.objects[source as usize].owner = p.other();
            bloodletter(&mut g, p, source);
            let reference = g.object_ref(source);
            g.grant_keyword_effects(&[source], reference, Keywords::LIFELINK);
            assert!(g.keywords(source).contains(Keywords::LIFELINK));
            let foe = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
            if doubling {
                named(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
            }
            let spell = named(&mut g, p, "Bite Down", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(source),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foe),
                },
            );
            settle(&mut g);
            assert_eq!(
                g.counter_count(foe, CounterKind::MinusOne),
                if doubling { 6 } else { 3 }
            );
            assert_eq!(g.state.objects[foe as usize].damage, 0);
            assert!(g.state.objects[foe as usize].damaged_this_turn);
            assert_eq!(g.state.players[p.index()].life, 23);
            assert_eq!(g.state.players[p.other().index()].life, 20);
        }
    }
}
#[test]
fn wither_combat_is_not_a_doubling_season_effect_and_simultaneous_retaliation_is_preserved() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        bloodletter(&mut g, p, source);
        let reference = g.object_ref(source);
        g.grant_keyword_effects(&[source], reference, Keywords::LIFELINK);
        assert!(g.keywords(source).contains(Keywords::LIFELINK));
        let foe = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        named(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
        g.state.attackers.push(Attacker {
            defender: Some(CombatDefender::Player(p.other())),
            card: source,
            blocked: true,
        });
        g.state.blocks.push((foe, source));
        g.state.blocking.push(foe);
        g.state.step = Step::Blockers;
        g.give_priority();
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.counter_count(foe, CounterKind::MinusOne), 3);
        assert_eq!(g.state.objects[foe as usize].damage, 0);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 23);
    }
}
#[test]
fn wither_ignores_indestructible_at_zero_toughness_and_prevention_stops_all_results() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    bloodletter(&mut g, Player::P0, source);
    let foe = named(&mut g, Player::P1, "Heirloom Auntie", Zone::Battlefield);
    let reference = g.object_ref(foe);
    g.grant_keyword_effects(&[foe], reference, Keywords::INDESTRUCTIBLE);
    assert!(g.keywords(foe).contains(Keywords::INDESTRUCTIBLE));
    for _ in 0..2 {
        let spell = named(&mut g, Player::P0, "Bite Down", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foe),
            },
        );
        settle(&mut g);
    }
    assert_eq!(g.state.objects[foe as usize].zone, Zone::Graveyard);
    let protected = named(&mut g, Player::P1, "Progenitus", Zone::Battlefield);
    let event = DamageEvent {
        source: g.damage_source(source),
        target: Target::Card(protected),
        amount: 3,
        combat: false,
        preventable: true,
    };
    g.apply_damage_batch(&[event]);
    g.advance_replacements(false);
    assert_eq!(g.counter_count(protected, CounterKind::MinusOne), 0);
    assert_eq!(g.state.objects[protected as usize].damage, 0);
    assert!(!g.state.objects[protected as usize].damaged_this_turn);
}
#[test]
fn wither_damage_results_survive_owned_counter_replacement_choices() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    bloodletter(&mut g, Player::P0, source);
    let foe = named(&mut g, Player::P1, "Gigantosaurus", Zone::Battlefield);
    g.state.objects[foe as usize].growth.toughness = 20;
    named(&mut g, Player::P1, "Doubling Season", Zone::Battlefield);
    named(&mut g, Player::P1, "Doubling Season", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Bite Down", Zone::Hand);
    checked(&mut g, Action::AnnounceCast { card: spell });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(foe),
        },
    );
    for _ in 0..100 {
        if matches!(g.state.pending, Pending::Replacement { .. }) {
            break;
        }
        advance(&mut g);
    }
    assert!(matches!(
        g.state.pending,
        Pending::Replacement { player: Player::P1 }
    ));
    assert_eq!(g.counter_count(foe, CounterKind::MinusOne), 0);
    settle(&mut g);
    assert_eq!(g.counter_count(foe, CounterKind::MinusOne), 12);
    assert_eq!(g.state.objects[foe as usize].damage, 0);
    assert!(g.state.objects[foe as usize].damaged_this_turn);
}
#[test]
fn wither_death_trigger_uses_departed_sources_last_known_keyword() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Fear of Lost Teeth", Zone::Battlefield);
    bloodletter(&mut g, Player::P0, source);
    let foe = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    g.move_object(source, Zone::Graveyard);
    g.prepare_triggers();
    for _ in 0..100 {
        if matches!(g.state.pending, Pending::Targets { .. }) {
            break;
        }
        advance(&mut g);
    }
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(foe),
        },
    );
    settle(&mut g);
    assert_eq!(g.counter_count(foe, CounterKind::MinusOne), 1);
    assert_eq!(g.state.objects[foe as usize].damage, 0);
}
#[test]
fn wither_preserves_deathtouch_and_lifelink_on_nonlethal_counter_damage() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Vampire Nighthawk", Zone::Battlefield);
    bloodletter(&mut g, Player::P0, source);
    let foe = named(&mut g, Player::P1, "Gigantosaurus", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Bite Down", Zone::Hand);
    checked(&mut g, Action::AnnounceCast { card: spell });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(foe),
        },
    );
    settle(&mut g);
    assert_eq!(
        g.state.objects[foe as usize].zone,
        Zone::Graveyard,
        "deathtouch kills despite positive remaining toughness"
    );
    assert_eq!(g.state.players[0].life, 23);
}
fn optional_cast(g: &mut Game, id: CardId, witness: Option<CardId>, target: Option<Target>) {
    checked(g, Action::AnnounceCast { card: id });
    assert!(matches!(
        g.state.pending,
        Pending::OptionalAdditionalCost { .. }
    ));
    assert!(
        g.observe(g.actor().unwrap().other())
            .additional_cost
            .is_none()
    );
    let object = witness.map(|id| g.object_ref(id));
    checked(g, Action::ChooseOptionalAdditionalCost { object });
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..100 {
        if g.state.announcing.is_none() {
            return;
        }
        advance(g);
    }
    panic!("optional spell did not finish casting");
}
#[test]
fn cinder_strike_optional_blight_is_chosen_before_targets_and_paid_history_survives_witness_death()
{
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = setup(p);
            let witness = paid.then(|| named(&mut g, p, "Llanowar Elves", Zone::Battlefield));
            let foe = named(&mut g, p.other(), "Gutsplitter Gang", Zone::Battlefield);
            let id = named(&mut g, p, "Cinder Strike", Zone::Hand);
            reject(
                &mut g,
                Action::Cast {
                    card: id,
                    target: Some(Target::Card(foe)),
                    payment: [0, 0, 0, 1, 0, 0],
                },
            );
            reject(
                &mut g,
                Action::BeginCast {
                    card: id,
                    payment: [0, 0, 0, 1, 0, 0],
                },
            );
            checked(&mut g, Action::AnnounceCast { card: id });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foe),
                },
            );
            let invalid = g.object_ref(foe);
            reject(
                &mut g,
                Action::ChooseOptionalAdditionalCost {
                    object: Some(invalid),
                },
            );
            let object = witness.map(|id| g.object_ref(id));
            checked(&mut g, Action::ChooseOptionalAdditionalCost { object });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foe),
                },
            );
            while g.state.announcing.is_some() {
                advance(&mut g);
            }
            assert_eq!(
                g.state
                    .stack
                    .iter()
                    .find(|s| s.card == id)
                    .unwrap()
                    .cast
                    .unwrap()
                    .optional_additional,
                Some(paid)
            );
            if let Some(witness) = witness {
                assert_eq!(g.state.objects[witness as usize].zone, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(
                g.state.objects[foe as usize].damage,
                if paid { 4 } else { 2 }
            );
        }
    }
}
#[test]
fn requiting_hex_limits_mana_value_and_gains_life_only_for_paid_resolving_spell() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            for invalidate in [false, true] {
                let mut g = setup(p);
                let witness = paid.then(|| named(&mut g, p, "Llanowar Elves", Zone::Battlefield));
                let foe = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
                let large = named(&mut g, p.other(), "Gutsplitter Gang", Zone::Battlefield);
                let id = named(&mut g, p, "Requiting Hex", Zone::Hand);
                checked(&mut g, Action::AnnounceCast { card: id });
                let object = witness.map(|id| g.object_ref(id));
                checked(&mut g, Action::ChooseOptionalAdditionalCost { object });
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(large),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(foe),
                    },
                );
                while g.state.announcing.is_some() {
                    advance(&mut g);
                }
                if invalidate {
                    g.move_object(foe, Zone::Hand);
                }
                settle(&mut g);
                assert_eq!(
                    g.state.objects[foe as usize].zone,
                    if invalidate {
                        Zone::Hand
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_eq!(
                    g.state.players[p.index()].life,
                    if paid && !invalidate { 22 } else { 20 }
                );
            }
        }
    }
}
#[test]
fn burning_curiosity_paid_witness_can_produce_mana_and_permissions_last_through_next_own_turn() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = setup(p);
            let witness = paid.then(|| named(&mut g, p, "Llanowar Elves", Zone::Battlefield));
            let id = named(&mut g, p, "Burning Curiosity", Zone::Hand);
            if let Some(witness) = witness {
                let a = named(&mut g, p, "Mountain", Zone::Battlefield);
                let b = named(&mut g, p, "Mountain", Zone::Battlefield);
                g.state.players[p.index()].mana = [0; 6];
                checked(&mut g, Action::AnnounceCast { card: id });
                let object = Some(g.object_ref(witness));
                checked(&mut g, Action::ChooseOptionalAdditionalCost { object });
                for card in [witness, a, b] {
                    checked(&mut g, Action::TapMana { card });
                }
                let pay = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
                    .unwrap();
                checked(&mut g, pay);
                assert_eq!(g.state.objects[witness as usize].zone, Zone::Graveyard);
            } else {
                optional_cast(&mut g, id, None, None);
            }
            let turn = g.state.turn;
            settle(&mut g);
            assert_eq!(g.state.exile_permissions.len(), if paid { 3 } else { 2 });
            assert!(g.state.exile_permissions.iter().all(|permission|permission.player==p&&permission.expires_turn==turn+2));
            let land = g.state.exile_permissions[0].object.card;
            assert!(g.legal_actions().contains(&Action::PlayLand { card: land }));
            checked(&mut g, Action::PlayLand { card: land });
            let remaining = g
                .state
                .exile_permissions
                .iter()
                .find(|permission| permission.object.card != land)
                .unwrap()
                .object
                .card;
            g.finish_cleanup();
            g.finish_cleanup();
            g.state.step = Step::Main1;
            g.state.pending = Pending::Priority;
            g.state.priority = p;
            assert!(
                g.legal_actions()
                    .contains(&Action::PlayLand { card: remaining })
            );
            g.finish_cleanup();
            assert!(!g.can_play_exiled(p, remaining));
        }
    }
}
#[test]
fn optional_additional_paid_qualification_is_inherited_by_spell_copies_without_repayment() {
    let mut g = setup(Player::P0);
    let teach = named(&mut g, Player::P0, "Teach by Example", Zone::Hand);
    cast(&mut g, teach);
    let witness = named(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let foe = named(&mut g, Player::P1, "Gigantosaurus", Zone::Battlefield);
    let id = named(&mut g, Player::P0, "Cinder Strike", Zone::Hand);
    optional_cast(&mut g, id, Some(witness), Some(Target::Card(foe)));
    for _ in 0..100 {
        if g.state.stack.iter().any(|s| s.is_copy) {
            break;
        }
        advance(&mut g);
    }
    let copy = g.state.stack.iter().find(|s| s.is_copy).unwrap();
    assert_eq!(copy.cast.unwrap().optional_additional, Some(true));
    assert_eq!(copy.cast.unwrap().payment, [0; 6]);
    settle(&mut g);
    assert_eq!(g.state.objects[foe as usize].damage, 8);
}

#[test]
fn bogslither_or_additional_routes_preserve_mana_for_free_casts() {
    for p in [Player::P0, Player::P1] {
        for free in [false, true] {
            for mana_route in [false, true] {
                let mut g = setup(p);
                if free {
                    named(&mut g, p, "Omniscience", Zone::Battlefield);
                }
                let witness =
                    (!mana_route).then(|| named(&mut g, p, "Llanowar Elves", Zone::Battlefield));
                let foe = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
                let id = named(&mut g, p, "Bogslither's Embrace", Zone::Hand);
                g.state.players[p.index()].mana = [
                    0,
                    0,
                    if free { 0 } else { 1 },
                    0,
                    0,
                    if mana_route {
                        if free { 3 } else { 4 }
                    } else {
                        if free { 0 } else { 1 }
                    },
                ];
                checked(
                    &mut g,
                    if free {
                        Action::AnnounceFree {
                            card: id,
                            face: CastFace::Normal,
                            flash_surcharge: false,
                        }
                    } else {
                        Action::AnnounceCast { card: id }
                    },
                );
                let cost = ManaCost {
                    colored: [0, 0, if free { 0 } else { 1 }, 0, 0, 0],
                    generic: if mana_route {
                        if free { 3 } else { 4 }
                    } else {
                        if free { 0 } else { 1 }
                    },
                };
                assert_eq!(
                    g.state
                        .announcing
                        .as_ref()
                        .unwrap()
                        .cost_options()
                        .nth(usize::from(mana_route)),
                    Some(cost)
                );
                checked(
                    &mut g,
                    Action::ChooseSpellCost {
                        cost: u16::from(mana_route),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(foe),
                    },
                );
                if let Some(witness) = witness {
                    let object = g.object_ref(witness);
                    checked(&mut g, Action::ChooseAdditionalCost { object });
                }
                while g.state.announcing.is_some() {
                    advance(&mut g);
                }
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                if let Some(witness) = witness {
                    assert_eq!(g.state.objects[witness as usize].zone, Zone::Graveyard);
                }
                settle(&mut g);
                assert_eq!(g.state.objects[foe as usize].zone, Zone::Exile);
            }
        }
    }
}
#[test]
fn free_or_cost_requires_additional_payment_and_colored_additional_mana_is_preserved() {
    for p in [Player::P0, Player::P1] {
        for name in ["Bogslither's Embrace", "Eaten Alive"] {
            let mut g = setup(p);
            named(&mut g, p, "Omniscience", Zone::Battlefield);
            named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
            let id = named(&mut g, p, name, Zone::Hand);
            let action = Action::AnnounceFree {
                card: id,
                face: CastFace::Normal,
                flash_surcharge: false,
            };
            g.state.players[p.index()].mana = [0; 6];
            reject(&mut g, action.clone());
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            if name == "Eaten Alive" {
                reject(&mut g, action.clone());
                g.state.players[p.index()].mana[2] = 1;
            }
            checked(&mut g, action);
            let costs: Vec<_> = g
                .legal_actions()
                .into_iter()
                .filter_map(|action| {
                    if let Action::ChooseSpellCost { cost } = action {
                        Some(cost)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(costs, vec![1]);
            assert_eq!(
                g.state.announcing.as_ref().unwrap().cost_options().nth(1),
                Some(ManaCost {
                    colored: [0, 0, if name == "Eaten Alive" { 1 } else { 0 }, 0, 0, 0],
                    generic: 3
                })
            );
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn dose_of_dawnglow_checks_resolving_main_phase_and_blights_returned_creature_as_effect() {
    for p in [Player::P0, Player::P1] {
        for (active, step, blight) in [
            (p, Step::Main1, false),
            (p, Step::Main2, false),
            (p, Step::BeginCombat, true),
            (p.other(), Step::Main1, true),
        ] {
            for doubled in [false, true] {
                let mut g = setup(p);
                g.state.active = active;
                g.state.step = step;
                if doubled {
                    named(&mut g, p, "Doubling Season", Zone::Battlefield);
                }
                let returned = named(&mut g, p, "Gigantosaurus", Zone::Graveyard);
                let enemy = named(&mut g, p.other(), "Gigantosaurus", Zone::Graveyard);
                let id = named(&mut g, p, "Dose of Dawnglow", Zone::Hand);
                checked(&mut g, Action::AnnounceCast { card: id });
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(returned),
                    },
                );
                while g.state.announcing.is_some() {
                    advance(&mut g);
                }
                if blight {
                    for _ in 0..30 {
                        if g.legal_actions().contains(&Action::Blight {
                            object: g.object_ref(returned),
                        }) {
                            break;
                        }
                        advance(&mut g);
                    }
                    let object = g.object_ref(returned);
                    checked(&mut g, Action::Blight { object });
                }
                settle(&mut g);
                assert_eq!(g.state.objects[returned as usize].zone, Zone::Battlefield);
                assert_eq!(
                    g.counter_count(returned, CounterKind::MinusOne),
                    if blight {
                        if doubled { 4 } else { 2 }
                    } else {
                        0
                    }
                );
            }
        }
    }
}
#[test]
fn dose_of_dawnglow_dead_target_suppresses_blight_and_returned_one_one_can_die() {
    for p in [Player::P0, Player::P1] {
        for invalid in [false, true] {
            let mut g = setup(p);
            g.state.step = Step::BeginCombat;
            let returned = named(&mut g, p, "Llanowar Elves", Zone::Graveyard);
            let witness = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
            let id = named(&mut g, p, "Dose of Dawnglow", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: id });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(returned),
                },
            );
            while g.state.announcing.is_some() {
                advance(&mut g);
            }
            if invalid {
                g.move_object(returned, Zone::Exile);
            } else {
                for _ in 0..30 {
                    if g.legal_actions().contains(&Action::Blight {
                        object: g.object_ref(returned),
                    }) {
                        break;
                    }
                    advance(&mut g);
                }
                let object = g.object_ref(returned);
                checked(&mut g, Action::Blight { object });
            }
            settle(&mut g);
            assert_eq!(
                g.state.objects[returned as usize].zone,
                if invalid {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.counter_count(witness, CounterKind::MinusOne), 0);
        }
    }
}

fn wait_type_choice(g: &mut Game) {
    for _ in 0..100 {
        if matches!(g.state.pending, Pending::ResolutionCreatureType { .. }) {
            return;
        }
        advance(g);
    }
    panic!("type choice missing");
}
#[test]
fn bloodline_bidding_chooses_pinned_type_at_resolution_and_returns_only_own_creature_cards() {
    assert!(serde_json::from_str::<crate::cards::CreatureType>("\"All\"").is_err());
    assert!(serde_json::from_str::<crate::cards::CreatureType>("\"Equipment\"").is_err());
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Impact Tremors", Zone::Battlefield);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        let archdruid = named(&mut g, p, "Elvish Archdruid", Zone::Graveyard);
        let changeling = named(&mut g, p, "Prideful Feastling", Zone::Graveyard);
        let kindred = named(&mut g, p, "Nameless Inversion", Zone::Graveyard);
        let nonelf = named(&mut g, p, "Gigantosaurus", Zone::Graveyard);
        let enemy = named(&mut g, p.other(), "Llanowar Elves", Zone::Graveyard);
        let hand = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        let id = named(&mut g, p, "Bloodline Bidding", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: id });
        assert!(g.observe(p).resolution_creature_type.is_none());
        wait_type_choice(&mut g);
        assert_eq!(
            g.legal_actions()
                .iter()
                .filter(|a| matches!(a, Action::ChooseCreatureType { .. }))
                .count(),
            crate::CREATURE_TYPES.len()
        );
        let kind = crate::cards::CreatureType::from_name("Elf").unwrap();
        checked(
            &mut g,
            Action::ChooseCreatureType {
                creature_type: kind,
            },
        );
        settle(&mut g);
        for object in [elf, archdruid, changeling] {
            assert_eq!(g.state.objects[object as usize].zone, Zone::Battlefield);
            assert_eq!(g.state.objects[object as usize].controller, p);
        }
        for object in [kindred, nonelf, enemy] {
            assert_eq!(g.state.objects[object as usize].zone, Zone::Graveyard);
        }
        assert_eq!(g.state.objects[hand as usize].zone, Zone::Hand);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        assert_eq!(g.stats(elf).power, 2);
        assert_eq!(g.stats(changeling).power, 3);
        assert!(g.observe(p).resolution_creature_type.is_none());
    }
}
#[test]
fn bloodline_chosen_type_survives_simultaneous_as_entry_choices_and_empty_batches() {
    for p in [Player::P0, Player::P1] {
        for empty in [false, true] {
            let mut g = setup(p);
            let first = named(&mut g, p, "Adaptive Automaton", Zone::Graveyard);
            let second = named(&mut g, p, "Adaptive Automaton", Zone::Graveyard);
            let changeling = named(
                &mut g,
                p,
                "Prideful Feastling",
                if empty { Zone::Hand } else { Zone::Graveyard },
            );
            let id = named(&mut g, p, "Bloodline Bidding", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: id });
            wait_type_choice(&mut g);
            let kind = crate::cards::CreatureType::from_name(if empty {
                "Time Lord"
            } else {
                "Construct"
            })
            .unwrap();
            checked(
                &mut g,
                Action::ChooseCreatureType {
                    creature_type: kind,
                },
            );
            if !empty {
                assert_eq!(g.observe(p).resolution_creature_type, Some(kind));
                assert_eq!(g.observe(p.other()).resolution_creature_type, Some(kind));
                for _ in 0..2 {
                    checked(
                        &mut g,
                        Action::ChooseEntryValue {
                            value: crate::cards::EntryChoiceValue::CreatureType(kind),
                        },
                    );
                }
            }
            settle(&mut g);
            assert_eq!(
                g.state.objects[first as usize].zone,
                if empty {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.state.objects[second as usize].zone,
                if empty {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            // An empty chosen-type batch still completes the owned resolution.
            assert_eq!(
                g.state.objects[changeling as usize].zone,
                if empty { Zone::Hand } else { Zone::Battlefield }
            );
            if !empty {
                assert_eq!(g.stats(changeling).power, 4);
            }
            assert!(g.resolution_chosen_creature_type().is_none());
        }
    }
}
#[test]
fn bloodline_bidding_convoke_uses_actual_tap_costs_before_type_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let black = named(&mut g, p, "Vampire Nighthawk", Zone::Battlefield);
        let green = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let returned = named(&mut g, p, "Gigantosaurus", Zone::Graveyard);
        let id = named(&mut g, p, "Bloodline Bidding", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 5];
        checked(&mut g, Action::AnnounceCast { card: id });
        checked(&mut g, Action::BeginConvoke);
        for (id, contribution) in [
            (black, ConvokeContribution::Black),
            (green, ConvokeContribution::Generic),
        ] {
            let object = g.object_ref(id);
            checked(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution,
                },
            );
        }
        wait_type_choice(&mut g);
        assert!(g.state.objects[black as usize].tapped && g.state.objects[green as usize].tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let kind = crate::cards::CreatureType::from_name("Dinosaur").unwrap();
        checked(
            &mut g,
            Action::ChooseCreatureType {
                creature_type: kind,
            },
        );
        settle(&mut g);
        assert_eq!(g.state.objects[returned as usize].zone, Zone::Battlefield);
    }
}

fn collective(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, "Collective Inferno", Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    for _ in 0..50 {
        if matches!(g.state.pending, Pending::EntryValue { .. }) {
            break;
        }
        advance(g);
    }
    let kind = crate::cards::CreatureType::from_name(name).unwrap();
    checked(
        g,
        Action::ChooseEntryValue {
            value: crate::cards::EntryChoiceValue::CreatureType(kind),
        },
    );
    settle(g);
    id
}
fn resolved_damage(g: &mut Game, source: CardId, target: Target, amount: i128, combat: bool) {
    let event = DamageEvent {
        source: g.damage_source(source),
        target,
        amount,
        combat,
        preventable: true,
    };
    g.apply_damage_batch(&[event]);
    g.advance_replacements(false);
    while g.state.replacements.is_some() {
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        checked(g, action);
    }
}
#[test]
fn collective_inferno_doubles_all_controlled_matching_sources_once_per_copy() {
    for p in [Player::P0, Player::P1] {
        for (name, matches) in [
            ("Chitinous Graspling", true),
            ("Boggart Mischief", true),
            ("Grizzly Bears", false),
        ] {
            for own in [false, true] {
                let mut g = setup(p);
                collective(&mut g, p, "Goblin");
                collective(&mut g, p, "Goblin");
                let source = named(
                    &mut g,
                    if own { p } else { p.other() },
                    name,
                    Zone::Battlefield,
                );
                // Ownership does not qualify a source; current control does.
                g.state.objects[source as usize].owner = if own { p.other() } else { p };
                resolved_damage(&mut g, source, Target::Player(p.other()), 2, true);
                assert_eq!(
                    g.state.players[p.other().index()].life,
                    20 - if own && matches { 8 } else { 2 }
                );
            }
        }
    }
}
#[test]
fn collective_inferno_uses_chosen_type_incarnation_and_departed_source_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let enchantment = collective(&mut g, p, "Nightmare");
        let source = named(&mut g, p, "Fear of Lost Teeth", Zone::Battlefield);
        let id = named(&mut g, p, "Cinder Strike", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: id });
        let object = Some(g.object_ref(source));
        checked(&mut g, Action::ChooseOptionalAdditionalCost { object });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
        for _ in 0..50 {
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Player(p.other()),
            }) {
                break;
            }
            advance(&mut g);
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.players[p.index()].life, 21);
        g.move_object(enchantment, Zone::Graveyard);
        let source = named(&mut g, p, "Fear of Lost Teeth", Zone::Battlefield);
        resolved_damage(&mut g, source, Target::Player(p.other()), 1, false);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        assert!(g.entry_value(g.object_ref(enchantment)).is_none());
    }
}
#[test]
fn collective_inferno_damage_doubling_precedes_wither_counter_doubling_and_prevention() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        collective(&mut g, p, "Elf");
        let source = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        bloodletter(&mut g, p, source);
        let foe = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        named(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
        let spell = named(&mut g, p, "Bite Down", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        for target in [source, foe] {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
        }
        settle(&mut g);
        assert_eq!(g.counter_count(foe, CounterKind::MinusOne), 8);
        assert_eq!(g.state.objects[foe as usize].damage, 0);
        let protected = named(&mut g, p.other(), "Progenitus", Zone::Battlefield);
        resolved_damage(&mut g, source, Target::Card(protected), 2, false);
        assert_eq!(g.counter_count(protected, CounterKind::MinusOne), 0);
        assert_eq!(g.state.objects[protected as usize].damage, 0);
    }
}
#[test]
fn collective_inferno_convoke_pays_actual_cost_and_its_choice_covers_friendly_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let id = named(&mut g, p, "Collective Inferno", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 2];
        checked(&mut g, Action::AnnounceCast { card: id });
        checked(&mut g, Action::BeginConvoke);
        let object = g.object_ref(source);
        checked(
            &mut g,
            Action::ChooseConvoker {
                object,
                contribution: ConvokeContribution::Generic,
            },
        );
        for _ in 0..50 {
            if matches!(g.state.pending, Pending::EntryValue { .. }) {
                break;
            }
            advance(&mut g);
        }
        assert!(g.state.objects[source as usize].tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let kind = crate::cards::CreatureType::from_name("Elf").unwrap();
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: crate::cards::EntryChoiceValue::CreatureType(kind),
            },
        );
        settle(&mut g);
        resolved_damage(&mut g, source, Target::Player(p), 1, false);
        assert_eq!(g.state.players[p.index()].life, 18);
    }
}

fn begin_perfect(g: &mut Game, p: Player, mode: u16, creature: CardId) {
    let id = named(g, p, "Perfect Intimidation", Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    checked(g, Action::ChooseMode { mode });
    if mode != 1 {
        reject(
            g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
    }
    if mode != 0 {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
    }
    while g.state.announcing.is_some() {
        advance(g);
    }
}
fn wait_hand_exile(g: &mut Game) {
    for _ in 0..60 {
        if matches!(g.state.pending, Pending::EffectChoice { .. })
            || (g.state.stack.is_empty() && g.state.resolving.is_none())
        {
            return;
        }
        advance(g);
    }
    panic!("missing hand exile choice");
}
#[test]
fn perfect_intimidation_all_modes_exile_up_to_two_privately_and_remove_every_counter_kind() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            for count in [0, 1, 3] {
                let mut g = setup(p);
                let creature = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
                g.state.objects[creature as usize].counters.plus_one = 2;
                g.state.objects[creature as usize].counters.stun = 3;
                g.state.objects[creature as usize].counters.lore = 4;
                g.state.objects[creature as usize].counters.loyalty = 5;
                let own = named(&mut g, p, "Llanowar Elves", Zone::Hand);
                let hand: Vec<_> = (0..count)
                    .map(|_| named(&mut g, p.other(), "Mountain", Zone::Hand))
                    .collect();
                begin_perfect(&mut g, p, mode, creature);
                if mode != 1 {
                    wait_hand_exile(&mut g);
                    for (index, &card) in hand.iter().take(2).enumerate() {
                        assert_eq!(g.actor(), Some(p.other()));
                        assert!(g.observe(p).effect_choice.is_none());
                        let object = g.object_ref(own);
                        reject(&mut g, Action::ExileHandCard { object });
                        let mut invalid = g.object_ref(card);
                        invalid.generation += 1;
                        reject(&mut g, Action::ExileHandCard { object: invalid });
                        let mut public = serde_json::to_value(g.observe(p)).unwrap();
                        public.as_object_mut().unwrap().remove("revision");
                        let object = g.object_ref(card);
                        checked(&mut g, Action::ExileHandCard { object });
                        if index == 0 && count > 1 {
                            let mut after = serde_json::to_value(g.observe(p)).unwrap();
                            after.as_object_mut().unwrap().remove("revision");
                            assert_eq!(after, public);
                            assert_eq!(g.state.objects[card as usize].zone, Zone::Hand);
                            reject(&mut g, Action::ExileHandCard { object });
                        }
                    }
                }
                settle(&mut g);
                for (index, &card) in hand.iter().enumerate() {
                    assert_eq!(
                        g.state.objects[card as usize].zone,
                        if mode != 1 && index < 2 {
                            Zone::Exile
                        } else {
                            Zone::Hand
                        }
                    );
                }
                assert_eq!(g.state.objects[own as usize].zone, Zone::Hand);
                assert_eq!(g.counter_amounts(creature).has_any(), mode == 0);
                assert_eq!(
                    g.state.players[p.other().index()].cards_discarded_this_turn,
                    0
                );
            }
        }
    }
}
#[test]
fn perfect_intimidation_both_modes_resolve_the_legal_player_half_after_creature_target_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let creature = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let first = named(&mut g, p.other(), "Mountain", Zone::Hand);
        let second = named(&mut g, p.other(), "Swamp", Zone::Hand);
        begin_perfect(&mut g, p, 2, creature);
        g.move_object(creature, Zone::Hand);
        wait_hand_exile(&mut g);
        for card in [first, second] {
            let object = g.object_ref(card);
            checked(&mut g, Action::ExileHandCard { object });
        }
        settle(&mut g);
        assert_eq!(g.state.objects[creature as usize].zone, Zone::Hand);
        for card in [first, second] {
            assert_eq!(g.state.objects[card as usize].zone, Zone::Exile);
        }
    }
}
#[test]
fn simultaneous_hand_exile_preserves_apnap_choice_privacy_until_every_player_has_chosen() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let creature = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
        let mut hands: [Vec<CardId>; 2] = std::array::from_fn(|_| Vec::new());
        for owner in [p, p.other()] {
            for name in ["Swamp", "Mountain"] {
                hands[owner.index()].push(named(&mut g, owner, name, Zone::Hand));
            }
        }
        begin_perfect(&mut g, p, 0, creature);
        wait_hand_exile(&mut g);
        let item = g.state.resolving.as_ref().unwrap().item.clone();
        g.start_effect_choice(
            &item,
            EffectChoiceKind::ExileFromHand,
            PlayerSet::Each,
            evidence::cost(0).filter,
            2,
            None,
        );
        for owner in [p, p.other()] {
            assert_eq!(g.actor(), Some(owner));
            let view = g.observe(owner).effect_choice.unwrap();
            assert!(view.selected[owner.other().index()].is_empty());
            for &card in &hands[owner.index()] {
                let object = g.object_ref(card);
                checked(&mut g, Action::ExileHandCard { object });
            }
            if owner == p {
                for &card in &hands[p.index()] {
                    assert_eq!(g.state.objects[card as usize].zone, Zone::Hand);
                }
            }
        }
        settle(&mut g);
        for card in hands.into_iter().flatten() {
            assert_eq!(g.state.objects[card as usize].zone, Zone::Exile);
        }
        for owner in [p, p.other()] {
            assert_eq!(g.state.players[owner.index()].cards_discarded_this_turn, 0);
        }
    }
}

fn hexing_setup(p: Player) -> Game {
    let mut g = setup(p);
    for player in &mut g.state.players {
        player.mana = [30; 6];
    }
    g
}
fn hexing_open(g: &mut Game, p: Player, name: &str, target: Option<Target>) -> CardId {
    while g.actor() != Some(p) {
        advance(g);
    }
    let id = named(g, p, name, Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    while g.state.announcing.is_some() {
        advance(g);
    }
    id
}
fn hexing_finish(g: &mut Game, pay: bool) -> usize {
    let mut payments = 0;
    for _ in 0..120 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return payments;
        }
        if matches!(g.state.pending, Pending::ResolutionLife { .. }) {
            let request = g.observe(g.actor().unwrap()).resolution_payment.unwrap();
            assert_eq!(request.life, Some(2));
            reject(g, Action::PayResolutionMana { payment: [0; 6] });
            if pay {
                checked(g, Action::PayResolutionLife);
            } else {
                checked(g, Action::DeclineResolutionLife);
            }
            payments += 1;
        } else {
            advance(g);
        }
    }
    panic!("unfinished wards");
}
#[test]
fn hexing_squelcher_itself_and_controlled_spells_cannot_be_countered() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = hexing_open(&mut g, p, "Hexing Squelcher", None);
        checked(&mut g, Action::Pass);
        let counter = hexing_open(
            &mut g,
            p.other(),
            "Essence Scatter",
            Some(Target::Spell(source)),
        );
        assert_eq!(hexing_finish(&mut g, true), 0);
        assert_eq!(g.state.objects[counter as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Battlefield);
        let spell = hexing_open(&mut g, p, "Shock", Some(Target::Player(p.other())));
        checked(&mut g, Action::Pass);
        hexing_open(&mut g, p.other(), "Negate", Some(Target::Spell(spell)));
        hexing_finish(&mut g, true);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn hexing_squelcher_ward_life_payment_decline_and_uncounterable_enemy_spells_remain_distinct() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            for enemy_protected in [false, true] {
                let mut g = hexing_setup(p);
                let source = named(&mut g, p, "Hexing Squelcher", Zone::Battlefield);
                if enemy_protected {
                    named(&mut g, p.other(), "Hexing Squelcher", Zone::Battlefield);
                }
                hexing_open(&mut g, p.other(), "Shock", Some(Target::Card(source)));
                assert_eq!(hexing_finish(&mut g, pay), 1);
                assert_eq!(
                    g.state.players[p.other().index()].life,
                    if pay { 18 } else { 20 }
                );
                assert_eq!(
                    g.state.objects[source as usize].zone,
                    if pay || enemy_protected {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
            }
        }
    }
}
#[test]
fn hexing_squelcher_grants_independent_recipient_owned_wards_and_excludes_exact_source() {
    for p in [Player::P0, Player::P1] {
        for (target_name, donors, expected) in [
            ("Grizzly Bears", 1, 1),
            ("Zul Ashur, Lich Lord", 1, 2),
            ("Zul Ashur, Lich Lord", 2, 3),
            ("Hexing Squelcher", 1, 2),
        ] {
            let mut g = hexing_setup(p);
            for _ in 0..donors {
                named(&mut g, p, "Hexing Squelcher", Zone::Battlefield);
            }
            let recipient = named(&mut g, p, target_name, Zone::Battlefield);
            hexing_open(
                &mut g,
                p.other(),
                "Giant Growth",
                Some(Target::Card(recipient)),
            );
            assert!(
                g.state
                    .pending_triggers
                    .iter()
                    .chain(&g.state.stack)
                    .filter(|item| item.attachment_trigger.is_some())
                    .all(|item| item.controller == p && item.object == g.object_ref(recipient))
            );
            assert_eq!(hexing_finish(&mut g, true), expected);
            assert_eq!(
                g.state.players[p.other().index()].life,
                20 - 2 * expected as i128
            );
        }
    }
}
#[test]
fn hexing_squelcher_granted_ward_survives_grantor_removal_and_recipient_control_change() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let giver = named(&mut g, p, "Hexing Squelcher", Zone::Battlefield);
        let recipient = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        hexing_open(
            &mut g,
            p.other(),
            "Giant Growth",
            Some(Target::Card(recipient)),
        );
        let packet = g
            .state
            .pending_triggers
            .iter()
            .chain(&g.state.stack)
            .find(|item| item.attachment_trigger.is_some())
            .unwrap()
            .clone();
        assert_eq!(packet.object, g.object_ref(recipient));
        assert_eq!(packet.controller, p);
        g.move_object(giver, Zone::Graveyard);
        g.gain_control(&packet, Subject::Source, PlayerSet::Opponent);
        assert_eq!(g.state.objects[recipient as usize].controller, p.other());
        assert_eq!(hexing_finish(&mut g, true), 1);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn hexing_squelcher_grant_and_ability_removal_follow_layer_six_timestamps() {
    for p in [Player::P0, Player::P1] {
        for grant_after_aura in [false, true] {
            let mut g = hexing_setup(p);
            let recipient = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            if !grant_after_aura {
                named(&mut g, p, "Hexing Squelcher", Zone::Battlefield);
            }
            hexing_open(
                &mut g,
                p,
                "Witness Protection",
                Some(Target::Card(recipient)),
            );
            hexing_finish(&mut g, true);
            if grant_after_aura {
                named(&mut g, p, "Hexing Squelcher", Zone::Battlefield);
            }
            hexing_open(
                &mut g,
                p.other(),
                "Giant Growth",
                Some(Target::Card(recipient)),
            );
            assert_eq!(hexing_finish(&mut g, true), usize::from(grant_after_aura));
        }
        let mut g = hexing_setup(p);
        let giver = named(&mut g, p, "Hexing Squelcher", Zone::Battlefield);
        let recipient = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        hexing_open(&mut g, p, "Witness Protection", Some(Target::Card(giver)));
        hexing_finish(&mut g, true);
        hexing_open(
            &mut g,
            p.other(),
            "Giant Growth",
            Some(Target::Card(recipient)),
        );
        assert_eq!(hexing_finish(&mut g, true), 0);
        assert!(!g.controlled_spells_uncounterable(p, Card::from_name("Shock").unwrap()));
    }
}
#[test]
fn hexing_squelcher_control_change_reassigns_the_continuous_grant_recipients() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p.other());
        let giver = named(&mut g, p, "Hexing Squelcher", Zone::Battlefield);
        let old_recipient = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        hexing_open(&mut g, p.other(), "Goatnap", Some(Target::Card(giver)));
        assert_eq!(hexing_finish(&mut g, true), 1);
        assert_eq!(g.state.objects[giver as usize].controller, p.other());
        hexing_open(
            &mut g,
            p.other(),
            "Giant Growth",
            Some(Target::Card(old_recipient)),
        );
        assert_eq!(hexing_finish(&mut g, true), 0);
        let new_recipient = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        hexing_open(&mut g, p, "Giant Growth", Some(Target::Card(new_recipient)));
        assert_eq!(hexing_finish(&mut g, true), 1);
    }
}
#[test]
fn hexing_squelcher_ward_counters_an_activated_ability_even_if_its_controller_has_spell_protection()
{
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        named(&mut g, p, "Hexing Squelcher", Zone::Battlefield);
        named(&mut g, p.other(), "Hexing Squelcher", Zone::Battlefield);
        let recipient = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let source = named(&mut g, p.other(), "Fanatical Firebrand", Zone::Battlefield);
        while g.actor() != Some(p.other()) {
            advance(&mut g);
        }
        let action=g.legal_actions().into_iter().find(|action|matches!(action,Action::Activate{card,target:Some(Target::Card(target)),..} if *card==source && *target==recipient)).unwrap();
        checked(&mut g, action);
        assert_eq!(hexing_finish(&mut g, false), 1);
        assert_eq!(g.state.objects[recipient as usize].damage, 0);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
    }
}

fn lasting_end(g: &mut Game) {
    g.begin_step(Step::End);
    g.prepare_triggers();
    settle(g);
}
#[test]
fn lasting_tarfire_observes_paid_blight_even_when_its_witness_dies_and_each_players_end_step() {
    for p in [Player::P0, Player::P1] {
        for active in [p, p.other()] {
            let mut g = hexing_setup(p);
            named(&mut g, p, "Lasting Tarfire", Zone::Battlefield);
            named(&mut g, p.other(), "Lasting Tarfire", Zone::Battlefield);
            let witness = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            let target = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
            let spell = named(&mut g, p, "Cinder Strike", Zone::Hand);
            optional_cast(&mut g, spell, Some(witness), Some(Target::Card(target)));
            settle(&mut g);
            assert_eq!(g.state.objects[witness as usize].zone, Zone::Graveyard);
            assert!(g.observe(p).players[p.index()].put_counter_on_creature_this_turn);
            assert!(!g.observe(p).players[p.other().index()].put_counter_on_creature_this_turn);
            g.state.active = active;
            lasting_end(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, 18);
            assert_eq!(g.state.players[p.index()].life, 20);
        }
    }
}
#[test]
fn lasting_tarfire_attributes_wither_to_the_damage_controller_not_recipient_or_doubling_controller()
{
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        named(&mut g, p, "Lasting Tarfire", Zone::Battlefield);
        named(&mut g, p.other(), "Lasting Tarfire", Zone::Battlefield);
        let source = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        bloodletter(&mut g, p, source);
        g.state.objects[source as usize].owner = p.other();
        let target = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        named(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
        let id = named(&mut g, p, "Bite Down", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: id });
        for card in [source, target] {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        settle(&mut g);
        assert_eq!(g.counter_count(target, CounterKind::MinusOne), 4);
        assert!(g.state.players[p.index()].put_counter_on_creature_this_turn);
        assert!(!g.state.players[p.other().index()].put_counter_on_creature_this_turn);
        lasting_end(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
#[test]
fn lasting_tarfire_ignores_zero_and_noncreature_counters_and_keeps_history_after_cancellation() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let enchantment = named(&mut g, p, "Lasting Tarfire", Zone::Battlefield);
        let creature = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.effect_counter(
            creature,
            CounterKind::PlusOne,
            0,
            replacements::ResumeMode::Pending,
        );
        g.effect_counter(
            enchantment,
            CounterKind::Lore,
            1,
            replacements::ResumeMode::Pending,
        );
        assert!(!g.state.players[p.index()].put_counter_on_creature_this_turn);
        lasting_end(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        // The first counter predates this turn; the new opposite counter counts
        // even though state-based actions subsequently cancel both counters.
        g.state.objects[creature as usize].counters.plus_one = 1;
        g.effect_counter(
            creature,
            CounterKind::MinusOne,
            1,
            replacements::ResumeMode::Pending,
        );
        checked(&mut g, Action::Pass);
        assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(creature, CounterKind::MinusOne), 0);
        lasting_end(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        g.finish_cleanup();
        assert!(!g.state.players[p.index()].put_counter_on_creature_this_turn);
        lasting_end(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn lasting_tarfire_records_actual_entry_counters_under_the_entering_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        named(&mut g, p, "Lasting Tarfire", Zone::Battlefield);
        named(&mut g, p.other(), "Lasting Tarfire", Zone::Battlefield);
        let creature = named(&mut g, p.other(), "Heirloom Auntie", Zone::Graveyard);
        let mut movement = g.entry_move(creature, Zone::Battlefield, false);
        movement.controller = p;
        g.stage_entries(vec![movement], None);
        g.advance_entries();
        settle(&mut g);
        assert_eq!(g.state.objects[creature as usize].owner, p.other());
        assert_eq!(g.state.objects[creature as usize].controller, p);
        assert_eq!(g.counter_count(creature, CounterKind::MinusOne), 2);
        assert!(g.state.players[p.index()].put_counter_on_creature_this_turn);
        assert!(!g.state.players[p.other().index()].put_counter_on_creature_this_turn);
        lasting_end(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn lasting_tarfire_prohibited_counter_placements_and_wither_damage_do_not_create_history() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        named(&mut g, p, "Lasting Tarfire", Zone::Battlefield);
        named(&mut g, p.other(), "Lasting Tarfire", Zone::Battlefield);
        let source = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        bloodletter(&mut g, p, source);
        let target = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        hexing_open(&mut g, p, "Blossombind", Some(Target::Card(target)));
        hexing_finish(&mut g, true);
        assert!(!g.can_receive_counters(target));
        g.effect_counter(
            target,
            CounterKind::PlusOne,
            1,
            replacements::ResumeMode::Pending,
        );
        assert!(!g.state.players[p.other().index()].put_counter_on_creature_this_turn);
        let id = named(&mut g, p, "Bite Down", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: id });
        for card in [source, target] {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        settle(&mut g);
        assert!(!g.state.players[p.index()].put_counter_on_creature_this_turn);
        assert!(!g.state.players[p.other().index()].put_counter_on_creature_this_turn);
        assert_eq!(g.state.objects[target as usize].damage, 0);
        assert_eq!(g.counter_count(target, CounterKind::MinusOne), 0);
        lasting_end(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn lasting_tarfire_captures_trigger_controller_and_resets_only_at_the_next_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let enchantment = named(&mut g, p, "Lasting Tarfire", Zone::Battlefield);
        let creature = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.begin_step(Step::End);
        g.prepare_triggers();
        settle(&mut g);
        g.effect_counter(
            creature,
            CounterKind::Stun,
            1,
            replacements::ResumeMode::Pending,
        );
        settle(&mut g);
        assert_eq!(
            g.state.players[p.other().index()].life,
            20,
            "counter after beginning does not create a retrospective trigger"
        );
        g.begin_step(Step::End);
        g.prepare_triggers();
        let packet = g
            .state
            .stack
            .iter()
            .chain(&g.state.pending_triggers)
            .find(|item| item.card == enchantment)
            .unwrap()
            .clone();
        g.gain_control(&packet, Subject::Source, PlayerSet::Opponent);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.objects[enchantment as usize].controller, p.other());
        g.state.step = Step::Cleanup;
        g.effect_counter(
            creature,
            CounterKind::Stun,
            1,
            replacements::ResumeMode::Pending,
        );
        assert!(g.state.players[p.index()].put_counter_on_creature_this_turn);
        g.finish_cleanup();
        assert!(!g.state.players[p.index()].put_counter_on_creature_this_turn);
    }
}
#[test]
fn lasting_tarfire_records_a_positive_loyalty_cost_on_an_animated_planeswalker() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        named(&mut g, p, "Lasting Tarfire", Zone::Battlefield);
        let walker = named(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 3;
        // Generic animation fixture preserves the planeswalker card and its
        // printed loyalty ability, so the positive loyalty cost is the placement.
        let item = g.activation_item(p, walker, 0);
        g.animate_source(
            &item,
            crate::cards::AnimationSpec {
                artifact: false,
                creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                base: Some((4, 4)),
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::EndOfTurn,
            },
        );
        assert!(g.is_creature(walker));
        checked(
            &mut g,
            Action::Activate {
                card: walker,
                ability: 0,
                target: None,
                payment: [0; 6],
                counter_payment: Counters::default(),
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 4);
        assert!(g.state.players[p.index()].put_counter_on_creature_this_turn);
        lasting_end(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}

fn torchmaster_blight(g: &mut Game) {
    g.begin_step(Step::BeginCombat);
    g.prepare_triggers();
    for _ in 0..100 {
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Blight { .. }))
        {
            return;
        }
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseTarget { .. }))
        );
        advance(g);
    }
    panic!("no combat blight choice");
}
#[test]
fn warren_torchmaster_targets_only_after_payment_even_if_source_dies_and_grants_opponent_haste() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Warren Torchmaster", Zone::Hand);
        cast(&mut g, source);
        assert_eq!(
            g.stats(source),
            crate::cards::Stats {
                power: 2,
                toughness: 2
            }
        );
        let witness = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let recipient = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        torchmaster_blight(&mut g);
        // Paying with a one-toughness creature kills it before selecting targets.
        let object = g.object_ref(witness);
        checked(&mut g, Action::Blight { object });
        for _ in 0..100 {
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseTarget { .. }))
            {
                break;
            }
            advance(&mut g);
        }
        assert_eq!(g.state.objects[witness as usize].zone, Zone::Graveyard);
        g.move_object(source, Zone::Graveyard);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(recipient),
            },
        );
        settle(&mut g);
        assert!(g.keywords(recipient).contains(Keywords::HASTE));
        assert!(g.observe(p).players[p.index()].put_counter_on_creature_this_turn);
        g.finish_cleanup();
        assert!(!g.keywords(recipient).contains(Keywords::HASTE));
    }
}
#[test]
fn warren_torchmaster_optional_blight_can_choose_source_and_decline_without_a_reflexive_trigger() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = hexing_setup(p);
            let source = named(&mut g, p, "Warren Torchmaster", Zone::Battlefield);
            named(&mut g, p, "Doubling Season", Zone::Battlefield);
            torchmaster_blight(&mut g);
            if pay {
                let object = g.object_ref(source);
                checked(&mut g, Action::Blight { object });
                for _ in 0..100 {
                    if g.legal_actions()
                        .iter()
                        .any(|a| matches!(a, Action::ChooseTarget { .. }))
                    {
                        break;
                    }
                    advance(&mut g);
                }
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(source),
                    },
                );
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(source, CounterKind::MinusOne),
                u64::from(pay)
            );
            assert_eq!(g.keywords(source).contains(Keywords::HASTE), pay);
        }
    }
}
#[test]
fn warren_torchmaster_ignores_opponents_combat_and_lost_target_does_not_refund_blight() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Warren Torchmaster", Zone::Battlefield);
        let recipient = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.active = p.other();
        g.begin_step(Step::BeginCombat);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 0);
        g.state.active = p;
        torchmaster_blight(&mut g);
        let object = g.object_ref(source);
        checked(&mut g, Action::Blight { object });
        for _ in 0..100 {
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseTarget { .. }))
            {
                break;
            }
            advance(&mut g);
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(recipient),
            },
        );
        for _ in 0..100 {
            if g.state.announcing.is_none() {
                break;
            }
            advance(&mut g);
        }
        g.move_object(recipient, Zone::Exile);
        g.move_object(recipient, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 1);
        assert!(!g.keywords(recipient).contains(Keywords::HASTE));
    }
}

fn flamebraider_mana(g: &mut Game, id: CardId, production: Mana) {
    checked(
        g,
        Action::ActivateMana {
            card: id,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production,
        },
    );
}
#[test]
fn flamebraider_produces_all_fifteen_two_color_distributions_as_an_immediate_mana_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Flamebraider", Zone::Hand);
        cast(&mut g, source);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::ActivateMana{card,..} if *card==source))
        );
        g.state.objects[source as usize].summoning_sick = false;
        let options: Vec<_> = g
            .legal_actions()
            .into_iter()
            .filter_map(|a| match a {
                Action::ActivateMana {
                    card, production, ..
                } if card == source => Some(production),
                _ => None,
            })
            .collect();
        assert_eq!(options.len(), 15);
        for mana in &options {
            assert_eq!(mana.iter().sum::<u32>(), 2);
            assert_eq!(mana[5], 0);
        }
        let distinct: std::collections::BTreeSet<_> = options.iter().collect();
        assert_eq!(distinct.len(), 15);
        reject(
            &mut g,
            Action::ActivateMana {
                card: source,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 2],
            },
        );
        g.state.players[p.index()].mana = [0; 6];
        flamebraider_mana(&mut g, source, [1, 0, 0, 1, 0, 0]);
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 1, 0, 0]);
        assert!(g.state.objects[source as usize].tapped);
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn flamebraider_restriction_survives_producer_departure_and_pays_elemental_spells_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Flamebraider", Zone::Battlefield);
        g.state.objects[source as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0; 6];
        flamebraider_mana(&mut g, source, [1, 0, 0, 1, 0, 0]);
        let wrong = named(&mut g, p, "Lightning Bolt", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: wrong });
        g.move_object(source, Zone::Exile);
        let right = named(&mut g, p, "Flamebraider", Zone::Hand);
        cast(&mut g, right);
        assert_eq!(g.state.objects[right as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn flamebraider_funds_elemental_activations_and_rejects_other_creature_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Flamebraider", Zone::Battlefield);
        g.state.objects[source as usize].summoning_sick = false;
        let recipient = named(&mut g, p, "Flame-Chain Mauler", Zone::Battlefield);
        let dragon = named(&mut g, p, "Shivan Dragon", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        flamebraider_mana(&mut g, source, [1, 0, 0, 1, 0, 0]);
        reject(
            &mut g,
            Action::Activate {
                card: dragon,
                ability: 0,
                target: None,
                payment: [0, 0, 0, 1, 0, 0],
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::Activate {
                card: recipient,
                ability: 0,
                target: None,
                payment: [1, 0, 0, 1, 0, 0],
                counter_payment: Counters::default(),
            },
        );
        settle(&mut g);
        assert_eq!(g.stats(recipient).power, 3);
        assert!(g.keywords(recipient).contains(Keywords::MENACE));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn flamebraider_speculative_funding_supports_split_hybrid_payment_and_kindred_noncreature_spells() {
    for p in [Player::P0, Player::P1] {
        for kindred in [false, true] {
            let mut g = hexing_setup(p);
            let source = named(&mut g, p, "Flamebraider", Zone::Battlefield);
            g.state.objects[source as usize].summoning_sick = false;
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, if kindred { 3 } else { 1 }];
            let spell = named(
                &mut g,
                p,
                if kindred {
                    "Ashling's Command"
                } else {
                    "Flaring Cinder"
                },
                Zone::Hand,
            );
            checked(&mut g, Action::AnnounceCast { card: spell });
            if !kindred {
                checked(&mut g, Action::ChooseSpellCost { cost: 1 });
            }
            for _ in 0..100 {
                if g.legal_actions().iter().any(|a|matches!(a,Action::ActivateMana{card,production,..} if *card==source && *production==[0,1,0,1,0,0])) {break;}
                advance(&mut g);
            }
            flamebraider_mana(&mut g, source, [0, 1, 0, 1, 0, 0]);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.state.objects[spell as usize].zone,
                if kindred {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn retched_wretch_returns_under_its_owner_blank_and_does_not_repeat_after_another_blighted_death() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p.other(), "Retched Wretch", Zone::Battlefield);
        let item = g.activation_item(p, source, 0);
        g.gain_control(&item, Subject::Source, PlayerSet::Controller);
        assert_eq!(g.state.objects[source as usize].controller, p);
        let spell = named(&mut g, p, "Cinder Strike", Zone::Hand);
        optional_cast(&mut g, spell, Some(source), Some(Target::Card(source)));
        settle(&mut g);
        assert!(g.on_battlefield(source));
        assert_eq!(g.state.objects[source as usize].controller, p.other());
        assert_eq!(
            g.stats(source),
            crate::cards::Stats {
                power: 4,
                toughness: 2
            }
        );
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 0);
        assert!(!g.printed_abilities_available(source));
        assert_eq!(g.observe(p).ability_removal_effects.len(), 1);
        g.finish_cleanup();
        assert!(!g.printed_abilities_available(source));
        // The new incarnation has no printed death trigger, even after another counter.
        g.effect_counter(
            source,
            CounterKind::MinusOne,
            1,
            replacements::ResumeMode::Pending,
        );
        let active = g.state.active;
        g.begin_step(Step::Main1);
        g.state.priority = active;
        g.state.players[active.index()].mana = [30; 6];
        let spell = named(&mut g, active, "Cinder Strike", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(
            &mut g,
            Action::ChooseOptionalAdditionalCost { object: None },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        settle(&mut g);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
        assert!(g.observe(p).ability_removal_effects.is_empty());
    }
}
#[test]
fn retched_wretch_uses_minus_counter_lki_and_requires_the_captured_graveyard_incarnation() {
    for p in [Player::P0, Player::P1] {
        for minus in [false, true] {
            let mut g = hexing_setup(p);
            let source = named(&mut g, p, "Retched Wretch", Zone::Hand);
            cast(&mut g, source);
            if minus {
                g.effect_counter(
                    source,
                    CounterKind::MinusOne,
                    1,
                    replacements::ResumeMode::Pending,
                );
            }
            g.move_object(source, Zone::Graveyard);
            g.prepare_triggers();
            assert_eq!(
                g.state.stack.len() + g.state.pending_triggers.len(),
                usize::from(minus)
            );
            if minus {
                g.move_object(source, Zone::Exile);
                g.move_object(source, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
        }
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Retched Wretch", Zone::Battlefield);
        g.state.objects[source as usize].counters.plus_one = 1;
        g.effect_counter(
            source,
            CounterKind::MinusOne,
            1,
            replacements::ResumeMode::Pending,
        );
        checked(&mut g, Action::Pass);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 0);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn exact_ability_removal_orders_intrinsic_copiable_and_resolution_keywords_and_expires_correctly() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let recipe = CopyRecipe {
            definition: Card::from_name("Bear Cub").unwrap(),
            exceptions: crate::cards::CopyExceptions::HASTE_AND_SACRIFICE,
        };
        g.stage_copy_creation(p, recipe, 1);
        settle(&mut g);
        let source = g
            .state
            .copy_recipes
            .as_ref()
            .unwrap()
            .last()
            .unwrap()
            .object;
        assert!(g.keywords(source.card).contains(Keywords::HASTE));
        g.grant_keyword_effects(&[source.card], source, Keywords::FLYING);
        g.remove_abilities_exact(&[source], crate::cards::AnimationDuration::Permanent);
        assert!(!g.keywords(source.card).contains(Keywords::HASTE));
        assert!(!g.keywords(source.card).contains(Keywords::FLYING));
        g.grant_keyword_effects(&[source.card], source, Keywords::TRAMPLE);
        assert!(g.keywords(source.card).contains(Keywords::TRAMPLE));
        lasting_end(&mut g);
        assert!(
            g.on_battlefield(source.card),
            "copiable intrinsic sacrifice ability was removed"
        );
        g.finish_cleanup();
        assert!(!g.keywords(source.card).contains(Keywords::TRAMPLE));
        assert!(!g.printed_abilities_available(source.card));
        let fresh = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let reference = g.object_ref(fresh);
        g.remove_abilities_exact(&[reference], crate::cards::AnimationDuration::EndOfTurn);
        assert!(!g.printed_abilities_available(fresh));
        g.finish_cleanup();
        assert!(g.printed_abilities_available(fresh));
    }
}
#[test]
fn exact_ability_removal_preserves_later_recipient_owned_grants_and_suppresses_older_ones() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        named(&mut g, p, "Hexing Squelcher", Zone::Battlefield);
        let recipient = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let object = g.object_ref(recipient);
        g.remove_abilities_exact(&[object], crate::cards::AnimationDuration::Permanent);
        hexing_open(
            &mut g,
            p.other(),
            "Lightning Bolt",
            Some(Target::Card(recipient)),
        );
        assert_eq!(hexing_finish(&mut g, true), 0);
        // Use a fresh exact incarnation, since the unwarded bear died.
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let object = g.object_ref(bear);
        g.remove_abilities_exact(&[object], crate::cards::AnimationDuration::Permanent);
        named(&mut g, p, "Hexing Squelcher", Zone::Battlefield);
        hexing_open(
            &mut g,
            p.other(),
            "Lightning Bolt",
            Some(Target::Card(bear)),
        );
        assert_eq!(hexing_finish(&mut g, false), 1);
        assert_eq!(g.state.objects[bear as usize].damage, 0);
    }
}
#[test]
fn exact_ability_removal_changes_speculative_mana_availability_and_blink_restores_printed_abilities()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.state.objects[source as usize].summoning_sick = false;
        let spell = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        let object = g.object_ref(source);
        g.remove_abilities_exact(&[object], crate::cards::AnimationDuration::Permanent);
        reject(&mut g, Action::AnnounceCast { card: spell });
        assert!(
            !g.legal_actions()
                .contains(&Action::TapMana { card: source })
        );
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        g.state.objects[source as usize].summoning_sick = false;
        assert!(g.observe(p).ability_removal_effects.is_empty());
        cast(&mut g, spell);
        assert!(g.on_battlefield(spell));
    }
}

#[test]
fn returning_without_abilities_preserves_the_entry_event_before_the_following_removal_clause() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Dawnhand Eulogist", Zone::Graveyard);
        let before = g.state.players[p.index()].library.len();
        let reference = g.object_ref(source);
        g.stage_linked_return(
            &ReturnLinkView {
                object: reference,
                zone: Zone::Graveyard,
                controller: p,
                counter: None,
            },
            Some((crate::cards::ReturnKind::WithoutAbilities, reference)),
        );
        g.advance_entries();
        assert!(!g.printed_abilities_available(source));
        assert!(!g.keywords(source).contains(Keywords::MENACE));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 3);
    }
}

#[test]
fn exact_ability_removal_noop_does_not_create_cold_state_or_advance_timestamps() {
    let mut g = setup(Player::P0);
    let card = named(&mut g, Player::P0, "Retched Wretch", Zone::Hand);
    let reference = g.object_ref(card);
    let before = g.snapshot();
    g.remove_abilities_exact(&[], crate::cards::AnimationDuration::Permanent);
    g.remove_abilities_exact(&[reference], crate::cards::AnimationDuration::Permanent);
    assert_eq!(g.snapshot(), before);
}
#[test]
fn retched_wretch_copy_token_cannot_return_from_the_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        g.stage_copy_creation(
            p,
            CopyRecipe::new(Card::from_name("Retched Wretch").unwrap()),
            1,
        );
        settle(&mut g);
        let source = g
            .state
            .copy_recipes
            .as_ref()
            .unwrap()
            .last()
            .unwrap()
            .object
            .card;
        let spell = named(&mut g, p, "Cinder Strike", Zone::Hand);
        optional_cast(&mut g, spell, Some(source), Some(Target::Card(source)));
        settle(&mut g);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Outside);
        assert!(g.observe(p).ability_removal_effects.is_empty());
    }
}
