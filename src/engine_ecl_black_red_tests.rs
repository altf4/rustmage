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
            faces: copies::CopyFaces::Definition,
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
                tapped: false,
                additional: Vec::new(),
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

#[test]
fn two_behold_creatures_pay_with_permanents_or_private_hand_cards_and_offer_the_extra_mana_route() {
    for p in [Player::P0, Player::P1] {
        for (name, witness_name, color) in [
            ("Mudbutton Cursetosser", "Warren Torchmaster", 2),
            ("Soulbright Seeker", "Flamebraider", 3),
        ] {
            for hand in [false, true] {
                let mut g = hexing_setup(p);
                let witness = named(
                    &mut g,
                    p,
                    if hand {
                        "Nameless Inversion"
                    } else {
                        witness_name
                    },
                    if hand { Zone::Hand } else { Zone::Battlefield },
                );
                let source = named(&mut g, p, name, Zone::Hand);
                let mut mana = [0; 6];
                mana[color] = 1;
                g.state.players[p.index()].mana = mana;
                checked(&mut g, Action::AnnounceCast { card: source });
                checked(&mut g, Action::ChooseSpellCost { cost: 0 });
                let object = g.object_ref(source);
                reject(&mut g, Action::ChooseAdditionalCost { object });
                assert!(g.observe(p.other()).additional_cost.is_none());
                let object = g.object_ref(witness);
                checked(&mut g, Action::ChooseAdditionalCost { object });
                assert!(!g.observe(p.other()).known_hand.contains(&object));
                checked(&mut g, Action::PayCosts { payment: mana });
                assert_eq!(g.observe(p.other()).known_hand.contains(&object), hand);
                settle(&mut g);
                assert!(g.on_battlefield(source));
                assert_eq!(
                    g.state.objects[witness as usize].zone,
                    if hand { Zone::Hand } else { Zone::Battlefield }
                );
            }
            let mut g = hexing_setup(p);
            let source = named(&mut g, p, name, Zone::Hand);
            let mut mana = [0; 6];
            mana[color] = 1;
            g.state.players[p.index()].mana = mana;
            reject(&mut g, Action::AnnounceCast { card: source });
            g.state.players[p.index()].mana[5] = 2;
            checked(&mut g, Action::AnnounceCast { card: source });
            checked(&mut g, Action::ChooseSpellCost { cost: 1 });
            settle(&mut g);
            assert!(g.on_battlefield(source));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn mudbutton_cursetosser_cannot_block_and_its_death_targets_only_opposing_power_two_or_less() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Mudbutton Cursetosser", Zone::Battlefield);
        let ally = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let strong = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        g.state.active = p.other();
        g.state.step = Step::Blockers;
        g.state.pending = Pending::Blockers;
        g.state.attackers.push(Attacker {
            card: enemy,
            defender: Some(CombatDefender::Player(p)),
            blocked: false,
        });
        reject(
            &mut g,
            Action::Block {
                blocker: source,
                attacker: enemy,
            },
        );
        assert!(g.legal_actions().contains(&Action::Block {
            blocker: ally,
            attacker: enemy
        }));
        g.state.pending = Pending::Priority;
        g.state.step = Step::Main1;
        g.state.active = p;
        g.state.priority = p;
        g.move_object(source, Zone::Graveyard);
        g.prepare_triggers();
        for _ in 0..100 {
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseTarget { .. }))
            {
                break;
            }
            advance(&mut g);
        }
        for target in [ally, strong] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        settle(&mut g);
        assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
        assert!(g.on_battlefield(strong));
    }
}
#[test]
fn mudbutton_cursetosser_rechecks_power_and_respects_indestructible_at_resolution() {
    for p in [Player::P0, Player::P1] {
        for grow in [false, true] {
            let mut g = hexing_setup(p);
            let source = named(&mut g, p, "Mudbutton Cursetosser", Zone::Battlefield);
            let target = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            g.move_object(source, Zone::Graveyard);
            g.prepare_triggers();
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
                    target: Target::Card(target),
                },
            );
            for _ in 0..100 {
                if g.state.announcing.is_none() {
                    break;
                }
                advance(&mut g);
            }
            if grow {
                g.state.objects[target as usize].growth.power = 1;
            } else {
                let object = g.object_ref(target);
                g.grant_keyword_effects(&[target], object, Keywords::INDESTRUCTIBLE);
            }
            settle(&mut g);
            assert!(g.on_battlefield(target));
        }
    }
}
fn seeker_activate(g: &mut Game, source: CardId, target: CardId) {
    checked(
        g,
        Action::Activate {
            card: source,
            ability: 0,
            target: Some(Target::Card(target)),
            payment: [0, 0, 0, 1, 0, 0],
            counter_payment: Counters::default(),
        },
    );
}
#[test]
fn soulbright_seeker_uses_the_stack_and_only_the_third_successful_resolution_adds_four_red() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Soulbright Seeker", Zone::Battlefield);
        let target = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 9, 0, 0];
        reject(
            &mut g,
            Action::Activate {
                card: source,
                ability: 0,
                target: Some(Target::Card(enemy)),
                payment: [0, 0, 0, 1, 0, 0],
                counter_payment: Counters::default(),
            },
        );
        for (count, red) in [(1, 8), (2, 7), (3, 10), (4, 9)] {
            seeker_activate(&mut g, source, target);
            assert!(!g.state.stack.is_empty());
            assert_eq!(
                g.state.players[p.index()].mana[3],
                if count == 1 {
                    8
                } else {
                    red - if count == 3 { 4 } else { 0 }
                }
            );
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana[3], red);
            assert!(g.keywords(target).contains(Keywords::TRAMPLE));
            assert_eq!(g.observe(p).ability_resolutions[0].count, count);
        }
        g.finish_cleanup();
        assert!(!g.keywords(target).contains(Keywords::TRAMPLE));
        assert!(g.observe(p).ability_resolutions.is_empty());
    }
}
#[test]
fn soulbright_seeker_fizzled_target_does_not_count_and_departed_source_can_finish_the_third_resolution()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Soulbright Seeker", Zone::Battlefield);
        let target = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 8, 0, 0];
        for _ in 0..2 {
            seeker_activate(&mut g, source, target);
            settle(&mut g);
        }
        seeker_activate(&mut g, source, target);
        g.move_object(target, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.observe(p).ability_resolutions[0].count, 2);
        assert_eq!(g.state.players[p.index()].mana[3], 5);
        let survivor = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        seeker_activate(&mut g, source, survivor);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert!(g.keywords(survivor).contains(Keywords::TRAMPLE));
        assert_eq!(g.observe(p).ability_resolutions[0].count, 3);
        assert_eq!(g.state.players[p.index()].mana[3], 8);
    }
}
#[test]
fn soulbright_seeker_resolution_history_follows_exact_source_across_control_changes_and_resets_on_blink()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Soulbright Seeker", Zone::Battlefield);
        let ally = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 5, 0, 0];
        g.state.players[p.other().index()].mana = [0, 0, 0, 5, 0, 0];
        for _ in 0..2 {
            seeker_activate(&mut g, source, source);
            settle(&mut g);
        }
        let item = g.activation_item(p.other(), source, 0);
        g.gain_control(&item, Subject::Source, PlayerSet::Controller);
        while g.actor() != Some(p.other()) {
            advance(&mut g);
        }
        seeker_activate(&mut g, source, ally);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].mana[3], 8);
        assert_eq!(g.observe(p).ability_resolutions[0].count, 3);
        g.move_object(source, Zone::Battlefield);
        while g.actor() != Some(p) {
            advance(&mut g);
        }
        seeker_activate(&mut g, source, source);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana[3], 2);
        assert_eq!(g.observe(p).ability_resolutions.len(), 2);
        assert_eq!(g.observe(p).ability_resolutions[1].count, 1);
    }
}

fn moonshadow(g: &mut Game, p: Player) -> CardId {
    let source = named(g, p, "Moonshadow", Zone::Hand);
    cast(g, source);
    source
}
#[test]
fn moonshadow_enters_with_six_minus_counters_and_one_actual_mill_batch_removes_only_one() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = moonshadow(&mut g, p);
        assert_eq!(
            g.stats(source),
            crate::cards::Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 6);
        assert!(g.keywords(source).contains(Keywords::MENACE));
        assert_eq!(g.minimum_blockers(source), 2);
        for name in ["Forest", "Grizzly Bears", "Mountain"] {
            named(&mut g, p, name, Zone::Library);
        }
        let eulogist = named(&mut g, p, "Dawnhand Eulogist", Zone::Hand);
        cast(&mut g, eulogist);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 5);
        assert_eq!(
            g.stats(source),
            crate::cards::Stats {
                power: 2,
                toughness: 2
            }
        );
    }
}
#[test]
fn moonshadow_survives_a_paid_wrath_and_observes_one_owned_permanent_card_death_batch() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = moonshadow(&mut g, p);
        let reference = g.object_ref(source);
        g.grant_keyword_effects(&[source], reference, Keywords::INDESTRUCTIBLE);
        let first = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let second = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let wrath = named(&mut g, p, "Day of Judgment", Zone::Hand);
        cast(&mut g, wrath);
        assert_eq!(g.state.objects[first as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[second as usize].zone, Zone::Graveyard);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 5);
    }
}
#[test]
fn moonshadow_counts_owned_physical_permanent_cards_from_every_zone_independent_of_previous_control()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = moonshadow(&mut g, p);
        for (index, zone) in [
            Zone::Hand,
            Zone::Library,
            Zone::Battlefield,
            Zone::Exile,
            Zone::Stack,
        ]
        .into_iter()
        .enumerate()
        {
            let card = named(&mut g, p, "Grizzly Bears", zone);
            if zone == Zone::Battlefield {
                let item = g.activation_item(p.other(), card, 0);
                g.gain_control(&item, Subject::Source, PlayerSet::Controller);
                assert_eq!(g.state.objects[card as usize].controller, p.other());
            }
            g.move_object(card, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(
                g.counter_count(source, CounterKind::MinusOne),
                5 - index as u64
            );
        }
    }
}
#[test]
fn moonshadow_excludes_opponents_cards_nonpermanent_graveyard_definitions_tokens_and_spell_copies()
{
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = moonshadow(&mut g, p);
        let opposing = named(&mut g, p.other(), "Grizzly Bears", Zone::Hand);
        let instant = named(&mut g, p, "Lightning Bolt", Zone::Hand);
        let events = g.move_objects_raw(&[opposing, instant], Zone::Graveyard);
        g.finish_zone_events(&events, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 6);
        // An animated physical instant is a creature before dying, but the new
        // graveyard card's printed instant definition is not a permanent card.
        let animated = named(&mut g, p, "Lightning Bolt", Zone::Battlefield);
        let item = g.activation_item(p, animated, 0);
        g.animate_source(
            &item,
            crate::cards::AnimationSpec {
                artifact: false,
                creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                base: Some((4, 4)),
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::Permanent,
            },
        );
        assert!(g.is_creature(animated));
        g.move_object(animated, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 6);
        g.stage_copy_creation(
            p,
            CopyRecipe::new(Card::from_name("Grizzly Bears").unwrap()),
            1,
        );
        settle(&mut g);
        let token = g
            .state
            .copy_recipes
            .as_ref()
            .unwrap()
            .last()
            .unwrap()
            .object
            .card;
        g.move_object(token, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 6);
        let copy = named(&mut g, p, "Grizzly Bears", Zone::Stack);
        g.state.objects[copy as usize].copy_kind = CopyKind::Spell;
        g.move_object(copy, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 6);
    }
}
#[test]
fn moonshadow_anywhere_trigger_uses_only_live_post_event_observers_and_one_trigger_per_surviving_source()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let survivor = moonshadow(&mut g, p);
        let dying = moonshadow(&mut g, p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let death_sources = g.death_trigger_sources();
        let events = g.move_objects_raw(&[dying, bear], Zone::Graveyard);
        g.finish_zone_events(&events, &death_sources);
        let moon_triggers: Vec<_> = g
            .state
            .pending_triggers
            .iter()
            .filter(|item| item.source_card == Card::from_name("Moonshadow").unwrap())
            .collect();
        assert_eq!(moon_triggers.len(), 1);
        assert_eq!(moon_triggers[0].card, survivor);
        settle(&mut g);
        assert_eq!(g.counter_count(survivor, CounterKind::MinusOne), 5);
        assert_eq!(g.state.objects[dying as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn moonshadow_event_guard_and_controller_capture_do_not_bind_a_later_source_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = moonshadow(&mut g, p);
        g.state.objects[source as usize].counters.minus_one = 0;
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Hand);
        g.move_object(bear, Zone::Graveyard);
        assert!(g.state.pending_triggers.is_empty());
        g.state.objects[source as usize].counters.minus_one = 6;
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Hand);
        g.move_object(bear, Zone::Graveyard);
        let item = g.activation_item(p.other(), source, 0);
        g.gain_control(&item, Subject::Source, PlayerSet::Controller);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 5);
        let own = named(&mut g, p, "Grizzly Bears", Zone::Hand);
        g.move_object(own, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 5);
        let opposing = named(&mut g, p.other(), "Grizzly Bears", Zone::Hand);
        g.move_object(opposing, Zone::Graveyard);
        assert!(!g.state.pending_triggers.is_empty());
        let before = g.object_ref(source);
        g.move_object(source, Zone::Exile);
        let movement = g.entry_move(source, Zone::Battlefield, false);
        g.stage_entries(vec![movement], None);
        g.advance_entries();
        settle(&mut g);
        assert_ne!(g.object_ref(source), before);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 6);
        let reference = g.object_ref(source);
        g.remove_abilities_exact(&[reference], crate::cards::AnimationDuration::Permanent);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Hand);
        g.move_object(bear, Zone::Graveyard);
        assert!(g.state.pending_triggers.is_empty());
    }
}

fn malice_elves(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            let id = id as CardId;
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id, "Elf")
                && g.object_view(id).colors == 20)
                .then_some(id)
        })
        .collect()
}
fn malice(g: &mut Game, p: Player, target: CardId) -> CardId {
    let spell = hexing_open(g, p, "Scarblade's Malice", Some(Target::Card(target)));
    hexing_finish(g, true);
    spell
}
#[test]
fn scarblades_malice_grants_actual_deathtouch_lifelink_and_creates_the_complete_black_green_elf() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let spell = malice(&mut g, p, source);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
        assert!(
            g.keywords(source)
                .contains(Keywords::DEATHTOUCH.union(Keywords::LIFELINK))
        );
        assert_eq!(g.observe(p).delayed_deaths.len(), 1);
        let bite = named(&mut g, p, "Bite Down", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: bite });
        for card in [source, enemy] {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        settle(&mut g);
        assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert!(malice_elves(&g, p).is_empty());
        let kill = named(&mut g, p, "Cinder Strike", Zone::Hand);
        optional_cast(&mut g, kill, None, Some(Target::Card(source)));
        settle(&mut g);
        let elves = malice_elves(&g, p);
        assert_eq!(elves.len(), 1);
        assert_eq!(
            g.stats(elves[0]),
            crate::cards::Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(!g.state.objects[elves[0] as usize].tapped);
        assert!(
            !g.keywords(elves[0])
                .intersects(Keywords::DEATHTOUCH.union(Keywords::LIFELINK))
        );
        assert!(g.observe(p).delayed_deaths.is_empty());
    }
}
#[test]
fn scarblades_malice_delayed_trigger_keeps_the_spell_controller_after_control_change_and_ability_loss()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let creature = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        malice(&mut g, p, creature);
        let reference = g.object_ref(creature);
        g.remove_abilities_exact(&[reference], crate::cards::AnimationDuration::Permanent);
        assert!(
            !g.keywords(creature)
                .intersects(Keywords::DEATHTOUCH.union(Keywords::LIFELINK))
        );
        let item = g.activation_item(p.other(), creature, 0);
        g.gain_control(&item, Subject::Source, PlayerSet::Controller);
        let kill = named(&mut g, p, "Cinder Strike", Zone::Hand);
        optional_cast(&mut g, kill, None, Some(Target::Card(creature)));
        settle(&mut g);
        assert_eq!(malice_elves(&g, p).len(), 1);
        assert!(malice_elves(&g, p.other()).is_empty());
    }
}
#[test]
fn scarblades_malice_multiple_independent_watches_each_trigger_once_for_a_token_death() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        g.stage_copy_creation(
            p,
            CopyRecipe::new(Card::from_name("Grizzly Bears").unwrap()),
            1,
        );
        settle(&mut g);
        let creature = g
            .state
            .copy_recipes
            .as_ref()
            .unwrap()
            .last()
            .unwrap()
            .object
            .card;
        let before = g.object_ref(creature);
        malice(&mut g, p, creature);
        malice(&mut g, p, creature);
        assert_eq!(g.observe(p).delayed_deaths.len(), 2);
        let kill = named(&mut g, p, "Cinder Strike", Zone::Hand);
        optional_cast(&mut g, kill, None, Some(Target::Card(creature)));
        settle(&mut g);
        assert_ne!(
            g.object_ref(creature),
            before,
            "new Elf token may reuse the departed token slot"
        );
        assert_eq!(malice_elves(&g, p).len(), 2);
        assert!(g.observe(p).delayed_deaths.is_empty());
    }
}
#[test]
fn scarblades_malice_blink_and_non_graveyard_departures_remove_only_the_exact_watch() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let creature = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        malice(&mut g, p, creature);
        g.move_object(creature, Zone::Exile);
        assert!(g.observe(p).delayed_deaths.is_empty());
        g.move_object(creature, Zone::Battlefield);
        g.move_object(creature, Zone::Graveyard);
        settle(&mut g);
        assert!(malice_elves(&g, p).is_empty());
    }
}
#[test]
fn scarblades_malice_expires_before_cleanup_sba_deaths_and_repeated_cleanup_does_not_rearm_it() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let creature = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        malice(&mut g, p, creature);
        hexing_open(&mut g, p, "Giant Growth", Some(Target::Card(creature)));
        hexing_finish(&mut g, true);
        g.effect_counter(
            creature,
            CounterKind::MinusOne,
            3,
            replacements::ResumeMode::Pending,
        );
        assert!(g.stats(creature).toughness > 0);
        g.finish_cleanup();
        settle(&mut g);
        assert_eq!(g.state.objects[creature as usize].zone, Zone::Graveyard);
        assert!(malice_elves(&g, p).is_empty());
        assert!(g.observe(p).delayed_deaths.is_empty());
        g.finish_cleanup();
        assert!(g.observe(p).delayed_deaths.is_empty());
    }
}
#[test]
fn scarblades_malice_illegal_and_lost_targets_cannot_create_watches_and_turn_end_clears_keywords() {
    for p in [Player::P0, Player::P1] {
        for disappear in [false, true] {
            let mut g = hexing_setup(p);
            let creature = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let spell = named(&mut g, p, "Scarblade's Malice", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            for _ in 0..100 {
                if g.state.announcing.is_none() {
                    break;
                }
                advance(&mut g);
            }
            if disappear {
                g.move_object(creature, Zone::Hand);
            }
            settle(&mut g);
            if disappear {
                assert!(g.observe(p).delayed_deaths.is_empty());
            } else {
                assert_eq!(g.observe(p).delayed_deaths.len(), 1);
                g.finish_cleanup();
                assert!(
                    !g.keywords(creature)
                        .intersects(Keywords::DEATHTOUCH.union(Keywords::LIFELINK))
                );
                assert!(g.observe(p).delayed_deaths.is_empty());
            }
        }
    }
}
#[test]
fn scarblades_malice_delayed_ability_is_independent_of_the_resolved_uncounterable_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        named(&mut g, p, "Hexing Squelcher", Zone::Battlefield);
        let creature = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        malice(&mut g, p, creature);
        g.move_object(creature, Zone::Graveyard);
        g.prepare_triggers();
        for _ in 0..100 {
            if g.state
                .stack
                .iter()
                .any(|item| item.source_card == Card::from_name("Scarblade's Malice").unwrap())
            {
                break;
            }
            advance(&mut g);
        }
        let item = g
            .state
            .stack
            .iter()
            .find(|item| item.source_card == Card::from_name("Scarblade's Malice").unwrap())
            .unwrap();
        assert_eq!(item.kind, StackKind::TriggeredAbility { ability: 0 });
        let id = item.id;
        assert!(g.counter_stack_item(id, false));
        settle(&mut g);
        assert!(malice_elves(&g, p).is_empty());
        assert!(g.observe(p).delayed_deaths.is_empty());
    }
}

fn champion_cast(g: &mut Game, p: Player, name: &str, witness: CardId) -> CardId {
    let source = named(g, p, name, Zone::Hand);
    checked(g, Action::AnnounceCast { card: source });
    assert!(g.observe(p.other()).additional_cost.is_none());
    let object = g.object_ref(source);
    reject(g, Action::ChooseAdditionalCost { object });
    let object = g.object_ref(witness);
    checked(g, Action::ChooseAdditionalCost { object });
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, pay);
    assert_eq!(g.object_view(witness).zone, Zone::Exile);
    source
}
#[test]
fn champion_pair_exile_hand_or_battlefield_witness_and_return_on_every_departure() {
    for p in [Player::P0, Player::P1] {
        for name in ["Champion of the Path", "Champion of the Weird"] {
            for hand in [false, true] {
                for destination in [Zone::Graveyard, Zone::Exile, Zone::Hand, Zone::Library] {
                    let mut g = hexing_setup(p);
                    let witness = named(
                        &mut g,
                        p,
                        "Nameless Inversion",
                        if hand { Zone::Hand } else { Zone::Battlefield },
                    );
                    let source = champion_cast(&mut g, p, name, witness);
                    settle(&mut g);
                    assert_eq!(
                        g.stats(source),
                        if name.ends_with("Path") {
                            Stats {
                                power: 7,
                                toughness: 3,
                            }
                        } else {
                            Stats {
                                power: 5,
                                toughness: 5,
                            }
                        }
                    );
                    assert_eq!(
                        g.state.players[p.other().index()].life,
                        20,
                        "no self-entry damage"
                    );
                    g.move_object(source, destination);
                    settle(&mut g);
                    assert_eq!(g.object_view(witness).zone, Zone::Hand);
                    assert!(
                        g.observe(p.other())
                            .known_hand
                            .contains(&g.object_ref(witness))
                    );
                }
            }
        }
    }
}
#[test]
fn champion_pair_countered_cost_cannot_bind_a_later_incarnation() {
    for p in [Player::P0, Player::P1] {
        for name in ["Champion of the Path", "Champion of the Weird"] {
            let mut g = hexing_setup(p);
            let witness = named(&mut g, p, "Nameless Inversion", Zone::Hand);
            let source = champion_cast(&mut g, p, name, witness);
            let item = g.state.stack.last().unwrap().id;
            assert!(g.counter_stack_item(item, false));
            settle(&mut g);
            g.move_object(source, Zone::Battlefield);
            g.move_object(source, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(g.object_view(witness).zone, Zone::Exile);
        }
    }
}
#[test]
fn champion_path_uses_current_entering_power_and_ignores_nonmatching_entries() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Champion of the Path", Zone::Battlefield);
        let ordinary = named(&mut g, p, "Grizzly Bears", Zone::Hand);
        g.move_object(ordinary, Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Flamebraider", Zone::Hand);
        g.move_object(enemy, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        let entrant = named(&mut g, p, "Flamebraider", Zone::Hand);
        g.move_object(entrant, Zone::Battlefield);
        let item = g.activation_item(p, entrant, 0);
        g.set_base_stats(&item, Subject::Source, Some(6), Some(6));
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 14);
    }
}
#[test]
fn champion_path_lki_preserves_power_keywords_and_exact_entering_object() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = hexing_setup(p);
            named(&mut g, p, "Champion of the Path", Zone::Battlefield);
            let entrant = named(&mut g, p, "Flamebraider", Zone::Hand);
            g.move_object(entrant, Zone::Battlefield);
            let reference = g.object_ref(entrant);
            g.grant_keyword_effects(&[entrant], reference, Keywords::LIFELINK);
            let item = g.activation_item(p, entrant, 0);
            g.set_base_stats(&item, Subject::Source, Some(4), Some(4));
            g.move_object(entrant, Zone::Hand);
            if blink {
                g.move_object(entrant, Zone::Battlefield);
                assert_ne!(g.object_ref(entrant), reference);
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 24);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if blink { 14 } else { 16 }
            );
        }
    }
}
#[test]
fn champion_path_damage_uses_entering_creatures_controller_not_observers_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        named(&mut g, p, "Champion of the Path", Zone::Battlefield);
        let entrant = named(&mut g, p, "Flamebraider", Zone::Hand);
        g.move_object(entrant, Zone::Battlefield);
        let reference = g.object_ref(entrant);
        g.grant_keyword_effects(&[entrant], reference, Keywords::LIFELINK);
        // Actual controller change before the trigger resolves; the opponent
        // remains determined by the original observer's controller.
        g.state.objects[entrant as usize].controller = p.other();
        g.move_object(entrant, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn champion_path_noncreature_elemental_has_no_power_and_each_observer_triggers() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        named(&mut g, p, "Champion of the Path", Zone::Battlefield);
        named(&mut g, p, "Champion of the Path", Zone::Battlefield);
        let kindred = named(&mut g, p, "Nameless Inversion", Zone::Hand);
        g.move_object(kindred, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        let entrant = named(&mut g, p, "Flamebraider", Zone::Hand);
        g.move_object(entrant, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}
#[test]
fn champion_weird_pays_life_and_blight_then_opponent_chooses_effect_recipient() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Champion of the Weird", Zone::Battlefield);
        let opponent = named(
            &mut g,
            p.other(),
            "Champion of the Weird",
            Zone::Battlefield,
        );
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        named(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
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
        let object = g.object_ref(source);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        assert_eq!(g.state.players[p.index()].life, 19);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 2);
        for _ in 0..100 {
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::Blight { .. }))
            {
                break;
            }
            advance(&mut g);
        }
        assert_eq!(g.actor(), Some(p.other()));
        let object = g.object_ref(source);
        reject(&mut g, Action::Blight { object });
        let object = g.object_ref(opponent);
        checked(&mut g, Action::Blight { object });
        settle(&mut g);
        assert_eq!(g.counter_count(opponent, CounterKind::MinusOne), 4);
        g.state.active = p.other();
        g.state.priority = p;
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
    }
}
#[test]
fn champion_weird_cost_can_kill_itself_return_witness_and_still_blight_opponent() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let witness = named(&mut g, p, "Nameless Inversion", Zone::Hand);
        let source = champion_cast(&mut g, p, "Champion of the Weird", witness);
        settle(&mut g);
        let opponent = named(
            &mut g,
            p.other(),
            "Champion of the Weird",
            Zone::Battlefield,
        );
        g.state.objects[source as usize].counters.minus_one = 4;
        g.state.objects[source as usize].tapped = true;
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        let object = g.object_ref(source);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(witness).zone, Zone::Hand);
        assert_eq!(g.counter_count(opponent, CounterKind::MinusOne), 2);
        assert_eq!(g.state.players[p.index()].life, 19);
    }
}
#[test]
fn champion_path_witness_can_produce_restricted_mana_before_its_exile_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let witness = named(&mut g, p, "Flamebraider", Zone::Battlefield);
        let source = named(&mut g, p, "Champion of the Path", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(&mut g, Action::AnnounceCast { card: source });
        let object = g.object_ref(witness);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        flamebraider_mana(&mut g, witness, [0, 0, 0, 2, 0, 0]);
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        assert_eq!(g.object_view(witness).zone, Zone::Exile);
        settle(&mut g);
        assert!(g.on_battlefield(source));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(witness).zone, Zone::Hand);
    }
}

fn mornsong_search(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.begin_step(Step::Draw);
    g.draw(p);
    for _ in 0..100 {
        if matches!(g.state.pending, Pending::EffectSearch { .. }) {
            return;
        }
        advance(g);
    }
    panic!("missing Mornsong search");
}
#[test]
fn mornsong_aria_blocks_every_draw_and_life_gain_without_empty_draw_loss() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Mornsong Aria", Zone::Hand);
        cast(&mut g, source);
        for player in [p, p.other()] {
            let card = named(&mut g, player, "Forest", Zone::Library);
            let library = g.state.players[player.index()].library.clone();
            g.draw(player);
            g.gain_life(player, 7);
            assert_eq!(g.state.players[player.index()].library, library);
            assert_eq!(g.state.players[player.index()].life, 20);
            assert_eq!(g.state.players[player.index()].life_gained_this_turn, 0);
            g.move_object(card, Zone::Graveyard);
            for id in g.state.players[player.index()].library.clone() {
                g.move_object(id, Zone::Graveyard);
            }
            g.draw(player);
            assert!(!g.state.players[player.index()].attempted_empty_draw);
        }
        g.move_object(source, Zone::Graveyard);
        g.gain_life(p, 2);
        assert_eq!(g.state.players[p.index()].life, 22);
        g.draw(p);
        assert!(g.state.players[p.index()].attempted_empty_draw);
    }
}
#[test]
fn mornsong_aria_active_player_search_is_private_required_and_does_not_draw() {
    for controller in [Player::P0, Player::P1] {
        for active in [Player::P0, Player::P1] {
            let mut g = hexing_setup(controller);
            let source = named(&mut g, controller, "Mornsong Aria", Zone::Hand);
            cast(&mut g, source);
            let chosen = named(&mut g, active, "Forest", Zone::Library);
            named(&mut g, active, "Mountain", Zone::Library);
            let foreign = named(&mut g, active.other(), "Island", Zone::Library);
            mornsong_search(&mut g, active);
            assert_eq!(g.actor(), Some(active));
            assert_eq!(g.state.players[active.index()].life, 17);
            assert_eq!(g.state.players[active.other().index()].life, 20);
            assert!(g.observe(active.other()).search_choice.is_none());
            assert_eq!(g.observe(active).search_choice.as_ref().unwrap().minimum, 1);
            reject(&mut g, Action::FinishSearch);
            reject(&mut g, Action::ChooseSearch { card: foreign });
            checked(&mut g, Action::ChooseSearch { card: chosen });
            assert_eq!(g.object_view(chosen).zone, Zone::Library);
            assert_eq!(
                g.observe(active).search_choice.as_ref().unwrap().selected,
                vec![chosen]
            );
            assert!(g.observe(active.other()).search_choice.is_none());
            checked(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(g.object_view(chosen).zone, Zone::Hand);
            assert!(
                !g.observe(active.other())
                    .known_hand
                    .contains(&g.object_ref(chosen))
            );
            assert_eq!(g.state.players[active.index()].cards_drawn_this_turn, 0);
        }
    }
}
#[test]
fn mornsong_aria_search_resolves_after_source_departs_and_empty_library_is_safe() {
    for p in [Player::P0, Player::P1] {
        for empty in [false, true] {
            let mut g = hexing_setup(p);
            let source = named(&mut g, p.other(), "Mornsong Aria", Zone::Battlefield);
            if empty {
                for id in g.state.players[p.index()].library.clone() {
                    g.move_object(id, Zone::Graveyard);
                }
            }
            let card = (!empty).then(|| named(&mut g, p, "Forest", Zone::Library));
            g.state.active = p;
            g.state.priority = p;
            g.begin_step(Step::Draw);
            g.draw(p);
            g.move_object(source, Zone::Graveyard);
            for _ in 0..100 {
                if matches!(g.state.pending, Pending::EffectSearch { .. }) {
                    break;
                }
                advance(&mut g);
            }
            assert_eq!(g.actor(), Some(p));
            if let Some(card) = card {
                checked(&mut g, Action::ChooseSearch { card });
            }
            checked(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 17);
            assert!(!g.state.players[p.index()].attempted_empty_draw);
            if let Some(card) = card {
                assert_eq!(g.object_view(card).zone, Zone::Hand);
            }
        }
    }
}
#[test]
fn mornsong_aria_ability_removal_restores_draw_and_life_gain_and_blocks_future_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Mornsong Aria", Zone::Battlefield);
        let reference = g.object_ref(source);
        g.remove_abilities_exact(&[reference], crate::cards::AnimationDuration::EndOfTurn);
        let card = named(&mut g, p, "Forest", Zone::Library);
        g.gain_life(p, 2);
        g.begin_step(Step::Draw);
        g.draw(p);
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert!(g.state.stack.is_empty());
        g.finish_cleanup();
        let card = named(&mut g, p, "Mountain", Zone::Library);
        g.draw(p);
        g.gain_life(p, 3);
        assert_eq!(g.object_view(card).zone, Zone::Library);
        assert_eq!(g.state.players[p.index()].life, 22);
    }
}
#[test]
fn unrelated_static_sources_do_not_prohibit_draw_and_two_arias_have_independent_draw_step_triggers()
{
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        named(&mut g, p, "Giant Cindermaw", Zone::Battlefield);
        let ordinary = named(&mut g, p, "Forest", Zone::Library);
        g.draw(p);
        assert_eq!(g.object_view(ordinary).zone, Zone::Hand);
        named(&mut g, p, "Mornsong Aria", Zone::Battlefield);
        named(&mut g, p.other(), "Mornsong Aria", Zone::Battlefield);
        let first = named(&mut g, p, "Mountain", Zone::Library);
        let second = named(&mut g, p, "Island", Zone::Library);
        mornsong_search(&mut g, p);
        checked(&mut g, Action::ChooseSearch { card: first });
        checked(&mut g, Action::FinishSearch);
        for _ in 0..100 {
            if matches!(g.state.pending, Pending::EffectSearch { .. }) {
                break;
            }
            advance(&mut g);
        }
        checked(&mut g, Action::ChooseSearch { card: second });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 14);
        assert_eq!(g.object_view(first).zone, Zone::Hand);
        assert_eq!(g.object_view(second).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}
fn endblaze_cast(g: &mut Game, p: Player, target: CardId, x: u32) -> CardId {
    // Keep this bounded scenario's complete X domain enumerable for raw/index parity.
    for amount in &mut g.state.players[p.index()].mana {
        *amount = (*amount).min(6);
    }
    while g.actor() != Some(p) {
        advance(g);
    }
    let spell = named(g, p, "End-Blaze Epiphany", Zone::Hand);
    checked(g, Action::AnnounceCast { card: spell });
    checked(g, Action::ChooseX { value: x });
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, pay);
    spell
}
fn endblaze_choose(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..100 {
        if matches!(g.state.pending, Pending::MilledSelection { .. }) {
            return;
        }
        advance(g);
    }
    panic!("missing End-Blaze exile choice");
}
#[test]
fn end_blaze_epiphany_lethal_x_damage_creates_delayed_trigger_before_state_based_death() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let unchosen = named(&mut g, p, "Forest", Zone::Library);
        let chosen = named(&mut g, p, "Lightning Bolt", Zone::Library);
        endblaze_cast(&mut g, p, victim, 2);
        endblaze_choose(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(g.object_view(chosen).zone, Zone::Exile);
        assert_eq!(g.object_view(unchosen).zone, Zone::Exile);
        assert!(g.observe(p.other()).milled_choice.is_none());
        assert_eq!(
            g.observe(p)
                .milled_choice
                .as_ref()
                .unwrap()
                .candidates
                .len(),
            2
        );
        reject(&mut g, Action::FinishMilledSelection);
        let object = g.object_ref(victim);
        reject(&mut g, Action::SelectMilled { object });
        let object = g.object_ref(chosen);
        checked(&mut g, Action::SelectMilled { object });
        settle(&mut g);
        assert!(g.can_play_exiled(p, chosen));
        assert!(!g.can_play_exiled(p.other(), chosen));
        assert!(!g.can_play_exiled(p, unchosen));
        checked(&mut g, Action::AnnounceExile { card: chosen });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 17);
    }
}
#[test]
fn end_blaze_epiphany_zero_x_watch_uses_later_signed_death_power_without_overwriting_x() {
    for p in [Player::P0, Player::P1] {
        for power in [-2, 0, 5] {
            let mut g = hexing_setup(p);
            let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            endblaze_cast(&mut g, p, victim, 0);
            settle(&mut g);
            assert_eq!(g.observe(p).delayed_deaths.len(), 1);
            let item = g.activation_item(p.other(), victim, 0);
            g.set_base_stats(&item, Subject::Source, Some(power), Some(8));
            let old = g.object_ref(victim);
            g.move_object(victim, Zone::Graveyard);
            g.prepare_triggers();
            let pending = g
                .state
                .pending_triggers
                .iter()
                .chain(g.state.stack.iter())
                .chain(g.state.ordering_triggers.iter())
                .chain(g.state.announcing.iter().map(|a| &a.item))
                .find(|s| s.source_card == Card::from_name("End-Blaze Epiphany").unwrap())
                .unwrap();
            assert_eq!(pending.x, Some(0));
            assert_eq!(pending.death_event.as_ref().unwrap().object, old);
            assert_eq!(pending.death_event.as_ref().unwrap().power, power);
            let library = g.state.players[p.index()].library.len();
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                library - power.max(0) as usize
            );
            assert_eq!(g.observe(p).exile_permissions.len(), usize::from(power > 0));
        }
    }
}
#[test]
fn end_blaze_epiphany_lost_target_and_blinked_or_exiled_watch_never_create_permissions() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = hexing_setup(p);
            let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            endblaze_cast(&mut g, p, victim, 0);
            if mode == 0 {
                g.move_object(victim, Zone::Hand);
            }
            settle(&mut g);
            if mode == 1 {
                g.move_object(victim, Zone::Hand);
                g.move_object(victim, Zone::Battlefield);
                g.move_object(victim, Zone::Graveyard);
            } else if mode == 2 {
                g.move_object(victim, Zone::Exile);
            } else if mode == 3 {
                g.finish_cleanup();
                g.move_object(victim, Zone::Graveyard);
            }
            settle(&mut g);
            assert!(g.observe(p).delayed_deaths.is_empty());
            assert!(g.observe(p).exile_permissions.is_empty());
        }
    }
}
#[test]
fn end_blaze_epiphany_permission_survives_current_turn_and_expires_at_casters_next_cleanup() {
    for p in [Player::P0, Player::P1] {
        for during_your_turn in [false, true] {
            let mut g = hexing_setup(if during_your_turn { p } else { p.other() });
            let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Library);
            endblaze_cast(&mut g, p, victim, 2);
            endblaze_choose(&mut g);
            let object = g.object_ref(land);
            checked(&mut g, Action::SelectMilled { object });
            settle(&mut g);
            assert!(g.can_play_exiled(p, land));
            let deadline = g.state.turn + if during_your_turn { 2 } else { 1 };
            assert_eq!(g.observe(p).exile_permissions[0].expires_turn, deadline);
            while g.state.turn < deadline {
                g.finish_cleanup();
            }
            assert!(g.can_play_exiled(p, land));
            assert_eq!(g.state.active, p);
            g.state.step = Step::Main1;
            g.state.priority = p;
            checked(&mut g, Action::PlayLand { card: land });
            assert_eq!(g.object_view(land).zone, Zone::Battlefield);
            g.finish_cleanup();
            assert!(g.observe(p).exile_permissions.is_empty());
        }
    }
}

#[test]
fn end_blaze_epiphany_watched_noncreature_has_no_power_when_it_dies() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        endblaze_cast(&mut g, p, victim, 0);
        settle(&mut g);
        hexing_open(
            &mut g,
            p,
            "Imprisoned in the Moon",
            Some(Target::Card(victim)),
        );
        hexing_finish(&mut g, true);
        assert!(!g.is_creature(victim));
        let library = g.state.players[p.index()].library.clone();
        g.move_object(victim, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library, library);
        assert!(g.observe(p).exile_permissions.is_empty());
    }
}
fn kindle_copies(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .copy_recipes
        .as_ref()
        .into_iter()
        .flat_map(|rows| rows.iter())
        .filter(|row| {
            g.on_battlefield(row.object.card)
                && g.object_ref(row.object.card) == row.object
                && g.state.objects[row.object.card as usize].controller == p
                && row.recipe.exceptions.sacrifice_abilities > 0
        })
        .map(|row| row.object.card)
        .collect()
}
fn kindle_pay(g: &mut Game) {
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, pay);
}
#[test]
fn kindle_inner_flame_normal_cast_needs_no_behold_and_copies_copiable_stats_with_intrinsic_haste_sacrifice()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let target = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.state.objects[target as usize].counters.plus_one = 2;
        let source = named(&mut g, p, "Kindle the Inner Flame", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: source });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        assert!(g.observe(p).additional_cost.is_none());
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        kindle_pay(&mut g);
        assert_eq!(
            mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
            4
        );
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        let copies = kindle_copies(&g, p);
        assert_eq!(copies.len(), 1);
        let copy = copies[0];
        assert_eq!(
            g.stats(copy),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(g.keywords(copy).contains(Keywords::HASTE));
        let reference = g.object_ref(copy);
        g.remove_abilities_exact(&[reference], crate::cards::AnimationDuration::EndOfTurn);
        g.begin_step(Step::End);
        settle(&mut g);
        assert!(
            g.on_battlefield(copy),
            "intrinsic sacrifice ability was removed"
        );
        g.finish_cleanup();
        assert!(g.keywords(copy).contains(Keywords::HASTE));
        g.begin_step(Step::End);
        settle(&mut g);
        assert!(!g.on_battlefield(copy));
    }
}
#[test]
fn kindle_inner_flame_flashback_beholds_three_distinct_elementals_including_kindred_hand_cards_and_mana_witness()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let target = named(&mut g, p, "Flamebraider", Zone::Battlefield);
        let first = named(&mut g, p, "Nameless Inversion", Zone::Hand);
        let second = named(&mut g, p, "Nameless Inversion", Zone::Hand);
        let enemy = named(&mut g, p.other(), "Flamebraider", Zone::Battlefield);
        let source = named(&mut g, p, "Kindle the Inner Flame", Zone::Graveyard);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnounceFlashback { card: source });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        assert!(g.observe(p.other()).additional_cost.is_none());
        let object = g.object_ref(enemy);
        reject(&mut g, Action::ChooseAdditionalCost { object });
        let object = g.object_ref(first);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        reject(&mut g, Action::ChooseAdditionalCost { object });
        let object = g.object_ref(second);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        let object = g.object_ref(target);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        for card in [first, second] {
            assert!(
                !g.observe(p.other())
                    .known_hand
                    .contains(&g.object_ref(card))
            );
        }
        flamebraider_mana(&mut g, target, [0, 0, 0, 2, 0, 0]);
        kindle_pay(&mut g);
        for card in [first, second] {
            assert_eq!(g.object_view(card).zone, Zone::Hand);
            assert!(
                g.observe(p.other())
                    .known_hand
                    .contains(&g.object_ref(card))
            );
        }
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert!(g.object_view(target).tapped);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        assert_eq!(kindle_copies(&g, p).len(), 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn kindle_inner_flame_flashback_requires_three_witnesses_and_correct_timing_without_shortcut_bypasses()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let target = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let source = named(&mut g, p, "Kindle the Inner Flame", Zone::Graveyard);
        for _ in 0..2 {
            named(&mut g, p, "Nameless Inversion", Zone::Hand);
        }
        reject(&mut g, Action::AnnounceFlashback { card: source });
        reject(&mut g, Action::AnnounceCast { card: source });
        reject(
            &mut g,
            Action::BeginCast {
                card: source,
                payment: [0, 0, 0, 1, 0, 1],
            },
        );
        reject(
            &mut g,
            Action::Cast {
                card: source,
                target: Some(Target::Card(target)),
                payment: [0, 0, 0, 1, 0, 1],
            },
        );
        named(&mut g, p, "Nameless Inversion", Zone::Hand);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceFlashback { card: source })
        );
        g.state.active = p.other();
        g.state.priority = p;
        reject(&mut g, Action::AnnounceFlashback { card: source });
    }
}
#[test]
fn kindle_inner_flame_flashback_lost_target_and_countered_spell_still_exile_source_and_keep_beheld_cards()
 {
    for p in [Player::P0, Player::P1] {
        for counter in [false, true] {
            let mut g = hexing_setup(p);
            let target = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let witnesses: Vec<_> = (0..3)
                .map(|_| named(&mut g, p, "Nameless Inversion", Zone::Hand))
                .collect();
            let source = named(&mut g, p, "Kindle the Inner Flame", Zone::Graveyard);
            checked(&mut g, Action::AnnounceFlashback { card: source });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            for &card in &witnesses {
                let object = g.object_ref(card);
                checked(&mut g, Action::ChooseAdditionalCost { object });
            }
            kindle_pay(&mut g);
            if counter {
                let id = g.state.stack.last().unwrap().id;
                assert!(g.counter_stack_item(id, false));
            } else {
                g.move_object(target, Zone::Hand);
            }
            settle(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Exile);
            assert!(kindle_copies(&g, p).is_empty());
            for card in witnesses {
                assert_eq!(g.object_view(card).zone, Zone::Hand);
            }
        }
    }
}
#[test]
fn kindle_inner_flame_free_and_granted_graveyard_casting_do_not_pay_the_printed_flashback_behold_cost()
 {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = hexing_setup(p);
            let target = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let source = named(
                &mut g,
                p,
                "Kindle the Inner Flame",
                if mode == 0 {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                },
            );
            if mode == 0 {
                named(&mut g, p, "Omniscience", Zone::Battlefield);
                checked(
                    &mut g,
                    Action::AnnounceFree {
                        card: source,
                        face: CastFace::Normal,
                        flash_surcharge: false,
                    },
                );
            } else {
                let mut item = g.activation_item(p, target, 0);
                item.targets = vec![g.bind_target(Target::Card(source))];
                g.grant_graveyard_cast(
                    &item,
                    if mode == 1 {
                        GraveyardCastKind::Cast
                    } else {
                        GraveyardCastKind::Flashback
                    },
                );
                checked(
                    &mut g,
                    if mode == 1 {
                        Action::AnnounceCast { card: source }
                    } else {
                        Action::AnnounceGrantedFlashback {
                            card: source,
                            flash_surcharge: false,
                        }
                    },
                );
            }
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            assert!(g.observe(p).additional_cost.is_none());
            let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
            kindle_pay(&mut g);
            assert_eq!(
                mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
                if mode == 0 { 0 } else { 4 }
            );
            settle(&mut g);
            assert_eq!(kindle_copies(&g, p).len(), 1);
            assert_eq!(
                g.object_view(source).zone,
                if mode == 2 {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
#[test]
fn kindle_inner_flame_spell_copy_inherits_effect_without_repaying_printed_flashback_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let teach = named(&mut g, p, "Teach by Example", Zone::Hand);
        cast(&mut g, teach);
        let target = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let witnesses: Vec<_> = (0..3)
            .map(|_| named(&mut g, p, "Nameless Inversion", Zone::Hand))
            .collect();
        let source = named(&mut g, p, "Kindle the Inner Flame", Zone::Graveyard);
        checked(&mut g, Action::AnnounceFlashback { card: source });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        for &card in &witnesses {
            let object = g.object_ref(card);
            checked(&mut g, Action::ChooseAdditionalCost { object });
        }
        kindle_pay(&mut g);
        settle(&mut g);
        assert_eq!(kindle_copies(&g, p).len(), 2);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        for card in witnesses {
            assert_eq!(g.object_view(card).zone, Zone::Hand);
        }
        assert!(g.observe(p).additional_cost.is_none());
    }
}
fn meek_choice(g: &mut Game, source: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    kindle_pay(g);
    for _ in 0..100 {
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::DeclineHandEntry))
        {
            return;
        }
        advance(g);
    }
    panic!("missing optional Meek hand choice");
}
#[test]
fn meek_attack_places_boundary_enchantment_creature_untapped_not_attacking_with_haste_then_sacrifices()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Meek Attack", Zone::Hand);
        cast(&mut g, source);
        g.state.objects[source as usize].tapped = true;
        let creature = named(&mut g, p, "Appendage Amalgam", Zone::Hand);
        let before = g.state.players[p.index()].mana.iter().sum::<u32>();
        meek_choice(&mut g, source);
        assert_eq!(
            before - g.state.players[p.index()].mana.iter().sum::<u32>(),
            2
        );
        assert!(g.observe(p.other()).effect_choice.is_none());
        let object = g.object_ref(creature);
        checked(&mut g, Action::PutHandCreature { object });
        settle(&mut g);
        assert!(g.on_battlefield(creature));
        assert!(!g.object_view(creature).tapped);
        assert!(g.state.attackers.is_empty());
        assert!(g.keywords(creature).contains(Keywords::HASTE));
        assert_eq!(g.observe(p).delayed_sacrifices.len(), 1);
        g.move_object(source, Zone::Graveyard);
        g.begin_step(Step::End);
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
    }
}
#[test]
fn meek_attack_optional_private_choice_declines_even_with_no_eligible_hand_and_never_refunds_mana()
{
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = hexing_setup(p);
            let source = named(&mut g, p, "Meek Attack", Zone::Battlefield);
            if mode == 1 {
                named(&mut g, p, "Moonshadow", Zone::Hand);
            }
            if mode == 2 {
                named(&mut g, p, "Grizzly Bears", Zone::Hand);
            }
            let before = g.state.players[p.index()].mana.iter().sum::<u32>();
            meek_choice(&mut g, source);
            assert!(g.observe(p).effect_choice.is_some());
            assert!(g.observe(p.other()).effect_choice.is_none());
            checked(&mut g, Action::DeclineHandEntry);
            settle(&mut g);
            assert_eq!(
                before - g.state.players[p.index()].mana.iter().sum::<u32>(),
                2
            );
            assert!(g.observe(p).delayed_sacrifices.is_empty());
        }
    }
}
#[test]
fn meek_attack_checks_current_hand_characteristics_and_excludes_foreign_noncreature_and_large_cards()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Meek Attack", Zone::Battlefield);
        let large = named(&mut g, p, "Moonshadow", Zone::Hand);
        let noncreature = named(&mut g, p, "Lightning Bolt", Zone::Hand);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Hand);
        let first = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let cda = named(&mut g, p, "Crusader of Odric", Zone::Hand);
        assert_eq!(
            g.stats(cda),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        meek_choice(&mut g, source);
        for card in [large, noncreature, enemy, cda] {
            let object = g.object_ref(card);
            reject(&mut g, Action::PutHandCreature { object });
        }
        checked(&mut g, Action::DeclineHandEntry);
        settle(&mut g);
        g.move_object(first, Zone::Graveyard);
        assert_eq!(
            g.stats(cda),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        meek_choice(&mut g, source);
        let object = g.object_ref(cda);
        checked(&mut g, Action::PutHandCreature { object });
        settle(&mut g);
        assert_eq!(
            g.stats(cda),
            Stats {
                power: 3,
                toughness: 3
            },
            "hand qualification precedes battlefield CDA growth"
        );
        assert!(g.keywords(cda).contains(Keywords::HASTE));
    }
}
#[test]
fn meek_attack_source_departure_keeps_owned_hand_entry_and_activation_works_during_opponents_turn()
{
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p.other());
        g.state.priority = p;
        let source = named(&mut g, p, "Meek Attack", Zone::Battlefield);
        let creature = named(&mut g, p, "Grizzly Bears", Zone::Hand);
        meek_choice(&mut g, source);
        g.move_object(source, Zone::Graveyard);
        let object = g.object_ref(creature);
        checked(&mut g, Action::PutHandCreature { object });
        settle(&mut g);
        assert!(g.on_battlefield(creature));
        assert!(g.keywords(creature).contains(Keywords::HASTE));
        g.begin_step(Step::End);
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
    }
}
#[test]
fn meek_attack_after_end_step_waits_until_next_end_and_haste_is_permanent_if_delay_countered() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Meek Attack", Zone::Battlefield);
        let creature = named(&mut g, p, "Grizzly Bears", Zone::Hand);
        g.begin_step(Step::End);
        meek_choice(&mut g, source);
        let object = g.object_ref(creature);
        checked(&mut g, Action::PutHandCreature { object });
        settle(&mut g);
        assert_eq!(g.observe(p).delayed_sacrifices.len(), 1);
        g.finish_cleanup();
        assert!(g.on_battlefield(creature));
        assert!(g.keywords(creature).contains(Keywords::HASTE));
        g.begin_step(Step::End);
        g.prepare_triggers();
        for _ in 0..100 {
            if g.state
                .stack
                .iter()
                .any(|s| s.kind == StackKind::DelayedSacrifice)
            {
                break;
            }
            advance(&mut g);
        }
        let id = g
            .state
            .stack
            .iter()
            .find(|s| s.kind == StackKind::DelayedSacrifice)
            .unwrap()
            .id;
        assert!(g.counter_stack_item(id, false));
        settle(&mut g);
        g.finish_cleanup();
        g.begin_step(Step::End);
        settle(&mut g);
        assert!(g.on_battlefield(creature));
        assert!(g.keywords(creature).contains(Keywords::HASTE));
        assert!(g.observe(p).delayed_sacrifices.is_empty());
    }
}
#[test]
fn meek_attack_exact_delayed_object_does_not_follow_a_recast_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Meek Attack", Zone::Battlefield);
        let creature = named(&mut g, p, "Grizzly Bears", Zone::Hand);
        meek_choice(&mut g, source);
        let object = g.object_ref(creature);
        checked(&mut g, Action::PutHandCreature { object });
        settle(&mut g);
        let old = g.object_ref(creature);
        g.move_object(creature, Zone::Hand);
        cast(&mut g, creature);
        assert_ne!(g.object_ref(creature), old);
        assert!(!g.keywords(creature).contains(Keywords::HASTE));
        g.begin_step(Step::End);
        settle(&mut g);
        assert!(g.on_battlefield(creature));
    }
}

#[test]
fn meek_attack_delay_keeps_caster_controller_and_haste_after_control_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Meek Attack", Zone::Battlefield);
        let creature = named(&mut g, p, "Grizzly Bears", Zone::Hand);
        meek_choice(&mut g, source);
        let object = g.object_ref(creature);
        checked(&mut g, Action::PutHandCreature { object });
        settle(&mut g);
        let offering = named(&mut g, p, "Harmless Offering", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: offering });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        kindle_pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(creature).controller, p.other());
        assert!(g.keywords(creature).contains(Keywords::HASTE));
        g.begin_step(Step::End);
        settle(&mut g);
        assert!(
            g.on_battlefield(creature),
            "the captured caster cannot sacrifice an opponent's creature"
        );
        g.finish_cleanup();
        assert!(g.keywords(creature).contains(Keywords::HASTE));
        assert!(g.observe(p).delayed_sacrifices.is_empty());
    }
}
fn soul_declare(g: &mut Game, spell: CardId, x: u32, witness: CardId) {
    checked(g, Action::AnnounceCast { card: spell });
    assert!(g.observe(g.actor().unwrap().other()).x_choice.is_none());
    checked(g, Action::ChooseX { value: x });
    let object = g.object_ref(witness);
    checked(g, Action::ChooseAdditionalCost { object });
}
#[test]
fn soul_immolation_uses_greatest_toughness_but_may_blight_a_smaller_creature_and_damages_only_opposition()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let greatest = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
        let witness = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let enemy_big = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let spell = named(&mut g, p, "Soul Immolation", Zone::Hand);
        let before = g.state.players[p.index()].mana.iter().sum::<u32>();
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, Action::ChooseX { value: 11 });
        checked(&mut g, Action::ChooseX { value: 10 });
        let foreign = g.object_ref(enemy);
        reject(&mut g, Action::ChooseAdditionalCost { object: foreign });
        let object = g.object_ref(witness);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        assert_eq!(g.counter_count(witness, CounterKind::MinusOne), 0);
        kindle_pay(&mut g);
        assert_eq!(g.object_view(witness).zone, Zone::Graveyard);
        assert_eq!(g.state.stack.last().unwrap().x, Some(10));
        assert_eq!(g.state.stack.last().unwrap().mana_value(), 5);
        assert_eq!(
            before - g.state.players[p.index()].mana.iter().sum::<u32>(),
            5
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 10);
        assert_eq!(g.state.players[p.index()].life, 20);
        for victim in [enemy, enemy_big] {
            assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        }
        assert!(g.on_battlefield(greatest));
        assert_eq!(g.state.objects[greatest as usize].damage, 0);
    }
}
#[test]
fn soul_immolation_free_cast_still_chooses_nonmana_x_and_keeps_it_after_the_only_creature_dies() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        named(&mut g, p, "Omniscience", Zone::Battlefield);
        let witness = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let spell = named(&mut g, p, "Soul Immolation", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::AnnounceFree {
                card: spell,
                face: CastFace::Normal,
                flash_surcharge: false,
            },
        );
        checked(&mut g, Action::ChooseX { value: 10 });
        let object = g.object_ref(witness);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        checked(&mut g, Action::PayCosts { payment: [0; 6] });
        assert_eq!(g.object_view(witness).zone, Zone::Graveyard);
        assert_eq!(g.state.stack.last().unwrap().mana_value(), 5);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 10);
    }
}
#[test]
fn soul_immolation_same_blight_witness_can_tap_for_mana_before_placing_cost_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let witness = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let lands: Vec<_> = ["Mountain", "Mountain", "Forest", "Forest"]
            .into_iter()
            .map(|name| named(&mut g, p, name, Zone::Battlefield))
            .collect();
        let spell = named(&mut g, p, "Soul Immolation", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        soul_declare(&mut g, spell, 1, witness);
        for card in std::iter::once(witness).chain(lands) {
            checked(&mut g, Action::TapMana { card });
        }
        assert_eq!(g.counter_count(witness, CounterKind::MinusOne), 0);
        kindle_pay(&mut g);
        assert_eq!(g.object_view(witness).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}
#[test]
fn soul_immolation_zero_requires_a_creature_but_does_not_place_counters_even_when_prohibited() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let spell = named(&mut g, p, "Soul Immolation", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::Cast {
                card: spell,
                target: None,
                payment: [0, 0, 0, 5, 0, 0],
            },
        );
        let witness = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        hexing_open(&mut g, p, "Blossombind", Some(Target::Card(witness)));
        hexing_finish(&mut g, true);
        assert!(!g.can_receive_counters(witness));
        checked(&mut g, Action::AnnounceCast { card: spell });
        assert_eq!(
            g.x_choice().unwrap().intervals,
            vec![XInterval {
                minimum: 0,
                maximum: 0
            }]
        );
        reject(&mut g, Action::ChooseX { value: 1 });
        checked(&mut g, Action::ChooseX { value: 0 });
        let object = g.object_ref(witness);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        kindle_pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(witness, CounterKind::MinusOne), 0);
        assert!(!g.state.players[p.index()].put_counter_on_creature_this_turn);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn soul_immolation_spell_copy_preserves_chosen_x_without_a_second_blight_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let teach = named(&mut g, p, "Teach by Example", Zone::Hand);
        cast(&mut g, teach);
        let witness = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
        let spell = named(&mut g, p, "Soul Immolation", Zone::Hand);
        soul_declare(&mut g, spell, 2, witness);
        kindle_pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(witness, CounterKind::MinusOne), 2);
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}
#[test]
fn soul_immolation_full_u32_counter_amount_is_owned_and_does_not_increase_mana_value_or_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let witness = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.objects[witness as usize].growth.toughness = i128::from(u32::MAX);
        let spell = named(&mut g, p, "Soul Immolation", Zone::Hand);
        soul_declare(&mut g, spell, u32::MAX, witness);
        let request = g.observe(p).payment_request.unwrap();
        assert_eq!(request.generic, 3);
        assert_eq!(request.required, [0, 0, 0, 2, 0, 0]);
        kindle_pay(&mut g);
        assert_eq!(
            g.counter_count(witness, CounterKind::MinusOne),
            u64::from(u32::MAX)
        );
        assert!(g.on_battlefield(witness));
        assert_eq!(g.state.stack.last().unwrap().mana_value(), 5);
        g.state.players[p.other().index()].life = i128::from(u32::MAX) + 20;
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn forced_blight_effect_can_choose_a_counter_prohibited_creature_without_placing_any_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let witness = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        hexing_open(&mut g, p, "Blossombind", Some(Target::Card(witness)));
        hexing_finish(&mut g, true);
        let source = named(&mut g, p, "Champion of the Weird", Zone::Battlefield);
        let own = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        let object = g.object_ref(own);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        kindle_pay(&mut g);
        for _ in 0..100 {
            if g.actor() == Some(p.other())
                && matches!(g.state.pending, Pending::EffectChoice { .. })
            {
                break;
            }
            advance(&mut g);
        }
        let object = g.object_ref(witness);
        checked(&mut g, Action::Blight { object });
        settle(&mut g);
        assert_eq!(g.counter_count(witness, CounterKind::MinusOne), 0);
        assert!(!g.state.players[p.other().index()].put_counter_on_creature_this_turn);
        assert_eq!(g.state.players[p.index()].life, 19);
    }
}
#[test]
fn soul_immolation_untargeted_simultaneous_damage_respects_protection_and_indestructible() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let witness = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
        let protected = named(&mut g, p.other(), "Progenitus", Zone::Battlefield);
        let indestructible = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let object = g.object_ref(indestructible);
        g.grant_keyword_effects(&[indestructible], object, Keywords::INDESTRUCTIBLE);
        let hexproof = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let object = g.object_ref(hexproof);
        g.grant_keyword_effects(&[hexproof], object, Keywords::HEXPROOF);
        let spell = named(&mut g, p, "Soul Immolation", Zone::Hand);
        soul_declare(&mut g, spell, 2, witness);
        kindle_pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.objects[protected as usize].damage, 0);
        assert!(!g.state.objects[protected as usize].damaged_this_turn);
        assert_eq!(g.state.objects[indestructible as usize].damage, 2);
        assert!(g.on_battlefield(indestructible));
        assert_eq!(g.object_view(hexproof).zone, Zone::Graveyard);
    }
}
#[test]
fn soul_immolation_timing_and_countered_spell_keep_blight_and_mana_paid() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let witness = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
        let spell = named(&mut g, p, "Soul Immolation", Zone::Hand);
        g.state.step = Step::End;
        reject(&mut g, Action::AnnounceCast { card: spell });
        g.state.step = Step::Main1;
        soul_declare(&mut g, spell, 2, witness);
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        kindle_pay(&mut g);
        let item = g.state.stack.last().unwrap().id;
        assert!(g.counter_stack_item(item, false));
        settle(&mut g);
        assert_eq!(g.counter_count(witness, CounterKind::MinusOne), 2);
        assert_eq!(
            mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
            5
        );
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}

fn dawnhand_link(g: &mut Game, p: Player, name: &str) -> (CardId, CardId, ObjectRef) {
    let source = named(g, p, "Dawnhand Dissident", Zone::Battlefield);
    let card = named(g, p, name, Zone::Exile);
    let source_ref = g.object_ref(source);
    g.remember_linked_exile_for(source_ref, &[card]);
    (source, card, source_ref)
}
fn dawnhand_announce(g: &mut Game, card: CardId, source: ObjectRef) {
    assert!(
        g.state.static_sources.contains(&source.card),
        "provider index"
    );
    assert_eq!(
        g.linked_counter_permission(g.actor().unwrap(), card, source),
        Some(3),
        "permission"
    );
    checked(
        g,
        Action::AnnounceLinkedExileCounterCost {
            card,
            source,
            flash_surcharge: false,
        },
    );
}
fn dawnhand_remove(g: &mut Game, object: ObjectRef, kinds: &[CounterKind]) {
    for &counter in kinds {
        checked(g, Action::ChooseCreatureCounterCost { object, counter });
    }
}
#[test]
fn dawnhand_paid_activations_tap_blight_surveillance_privacy_and_exact_graveyard_link() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Dawnhand Dissident", Zone::Hand);
        cast(&mut g, source);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.objects[source as usize].summoning_sick = false;
        let donor = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
        let top = named(&mut g, p, "Shock", Zone::Library);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let donor_ref = g.object_ref(donor);
        checked(&mut g, Action::ChooseAdditionalCost { object: donor_ref });
        kindle_pay(&mut g);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.counter_count(donor, CounterKind::MinusOne), 1);
        for _ in 0..6 {
            if g.observe(p).library_choice.is_some() {
                break;
            }
            advance(&mut g);
        }
        assert!(g.observe(p).library_choice.is_some());
        assert!(g.observe(p.other()).library_choice.is_none());
        checked(
            &mut g,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Graveyard,
            },
        );
        settle(&mut g);
        g.state.objects[source as usize].tapped = false;
        let card = named(&mut g, p.other(), "Divination", Zone::Graveyard);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
        checked(&mut g, Action::ChooseAdditionalCost { object: donor_ref });
        kindle_pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Exile);
        assert_eq!(g.counter_count(donor, CounterKind::MinusOne), 3);
        assert!(
            g.state
                .linked_choices
                .as_ref()
                .unwrap()
                .links
                .iter()
                .any(|l| l.source == g.object_ref(source) && l.exiled == g.object_ref(card))
        );
        assert!(!g.legal_actions().iter().any(
            |a| matches!(a, Action::AnnounceLinkedExileCounterCost { card: c, .. } if *c == card)
        ));
    }
}
#[test]
fn dawnhand_mixed_named_counter_payment_is_additional_to_mana_and_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let (source, card, reference) = dawnhand_link(&mut g, p, "Grizzly Bears");
        for kind in [
            CounterKind::Flying,
            CounterKind::FirstStrike,
            CounterKind::Lifelink,
        ] {
            g.add_counter(source, kind, 1);
        }
        let wrong = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        g.add_counter(wrong, CounterKind::PlusOne, 20);
        reject(&mut g, Action::AnnounceExile { card });
        reject(
            &mut g,
            Action::BeginCast {
                card,
                payment: [0, 0, 0, 0, 1, 1],
            },
        );
        reject(
            &mut g,
            Action::Cast {
                card,
                target: None,
                payment: [0, 0, 0, 0, 1, 1],
            },
        );
        let before = g.state.players[p.index()].mana;
        dawnhand_announce(&mut g, card, reference);
        assert!(g.observe(p).additional_cost.is_some());
        assert!(g.observe(p.other()).additional_cost.is_none());
        let wrong_ref = g.object_ref(wrong);
        reject(
            &mut g,
            Action::ChooseCreatureCounterCost {
                object: wrong_ref,
                counter: CounterKind::PlusOne,
            },
        );
        dawnhand_remove(
            &mut g,
            reference,
            &[
                CounterKind::Flying,
                CounterKind::FirstStrike,
                CounterKind::Lifelink,
            ],
        );
        kindle_pay(&mut g);
        assert!(
            g.state.players[p.index()]
                .mana
                .iter()
                .map(|&v| u64::from(v))
                .sum::<u64>()
                < before.iter().map(|&v| u64::from(v)).sum::<u64>()
        );
        assert_eq!(g.counter_amounts(source).total(), 0);
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Battlefield);
    }
}
#[test]
fn dawnhand_exact_provider_turn_control_and_source_departure_during_announcement() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let (source, card, reference) = dawnhand_link(&mut g, p, "Grizzly Bears");
        g.add_counter(source, CounterKind::PlusOne, 3);
        let linked = Action::AnnounceLinkedExileCounterCost {
            card,
            source: reference,
            flash_surcharge: false,
        };
        g.state.objects[card as usize].owner = p.other();
        reject(&mut g, linked.clone());
        g.state.objects[card as usize].owner = p;
        g.state.step = Step::Upkeep;
        reject(&mut g, linked);
        g.state.step = Step::Main1;
        let noncreature = named(&mut g, p, "Divination", Zone::Exile);
        g.remember_linked_exile_for(reference, &[noncreature]);
        reject(
            &mut g,
            Action::AnnounceLinkedExileCounterCost {
                card: noncreature,
                source: reference,
                flash_surcharge: false,
            },
        );
        g.state.active = p.other();
        reject(
            &mut g,
            Action::AnnounceLinkedExileCounterCost {
                card,
                source: reference,
                flash_surcharge: false,
            },
        );
        g.state.active = p;
        g.state.objects[source as usize].controller = p.other();
        reject(
            &mut g,
            Action::AnnounceLinkedExileCounterCost {
                card,
                source: reference,
                flash_surcharge: false,
            },
        );
        g.state.objects[source as usize].controller = p;
        let other = named(&mut g, p, "Dawnhand Dissident", Zone::Battlefield);
        let other_ref = g.object_ref(other);
        reject(
            &mut g,
            Action::AnnounceLinkedExileCounterCost {
                card,
                source: other_ref,
                flash_surcharge: false,
            },
        );
        dawnhand_announce(&mut g, card, reference);
        dawnhand_remove(&mut g, reference, &[CounterKind::PlusOne; 3]);
        g.move_object(source, Zone::Graveyard);
        // The chosen donor must remain, even though the provider is no longer required.
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayCosts { .. }))
        );
        reject(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 1, 1],
            },
        );
        let mut h = hexing_setup(p);
        let (source, card, reference) = dawnhand_link(&mut h, p, "Grizzly Bears");
        let donor = named(&mut h, p, "Gigantosaurus", Zone::Battlefield);
        h.add_counter(donor, CounterKind::PlusOne, 3);
        dawnhand_announce(&mut h, card, reference);
        let donor_ref = h.object_ref(donor);
        dawnhand_remove(&mut h, donor_ref, &[CounterKind::PlusOne; 3]);
        h.move_object(source, Zone::Graveyard);
        kindle_pay(&mut h);
        settle(&mut h);
        assert_eq!(h.object_view(card).zone, Zone::Battlefield);
    }
}
#[test]
fn dawnhand_printed_champion_exile_and_counter_cost_can_share_donor_before_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let (source, card, reference) = dawnhand_link(&mut g, p, "Champions of the Perfect");
        g.add_counter(source, CounterKind::PlusOne, 3);
        dawnhand_announce(&mut g, card, reference);
        checked(&mut g, Action::ChooseAdditionalCost { object: reference });
        checked(
            &mut g,
            Action::ChooseCounterCostOrder {
                after_printed: false,
            },
        );
        dawnhand_remove(&mut g, reference, &[CounterKind::PlusOne; 3]);
        kindle_pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Battlefield);
        g.move_object(card, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Hand);
    }
}
#[test]
fn dawnhand_after_printed_exile_returns_fresh_counter_donor_before_cast_completes() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let (_, card, reference) = dawnhand_link(&mut g, p, "Champions of the Perfect");
        let hunter = named(&mut g, p, "Henchbots", Zone::Battlefield);
        let auntie = named(&mut g, p, "Heirloom Auntie", Zone::Exile);
        let hunter_ref = g.object_ref(hunter);
        // Reachable prior link: Henchbots exiled our tapped Auntie after control
        // was donated to the opponent. Oko subsequently made Henchbots an Elf.
        let item = g.cast_preview(p, hunter, Zone::Battlefield, hunter_ref, 0, [0; 6]);
        g.gain_all_creature_types_permanently(&item, Subject::Source);
        g.state.exile_links.push(ExileLink {
            source: hunter_ref,
            exiled: g.object_ref(auntie),
            return_zone: Zone::Battlefield,
        });
        let donor = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
        g.add_counter(donor, CounterKind::PlusOne, 1);
        dawnhand_announce(&mut g, card, reference);
        checked(&mut g, Action::ChooseAdditionalCost { object: hunter_ref });
        assert!(
            !g.legal_actions().contains(&Action::ChooseCounterCostOrder {
                after_printed: false
            })
        );
        checked(
            &mut g,
            Action::ChooseCounterCostOrder {
                after_printed: true,
            },
        );
        kindle_pay(&mut g);
        for _ in 0..20 {
            if matches!(g.state.pending, Pending::CreatureCounterCost { .. }) {
                break;
            }
            advance(&mut g);
        }
        assert_eq!(g.object_view(auntie).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(auntie, CounterKind::MinusOne), 2);
        assert!(g.state.stack.iter().all(|s| s.card != card));
        let auntie_ref = g.object_ref(auntie);
        dawnhand_remove(&mut g, auntie_ref, &[CounterKind::MinusOne; 2]);
        let donor_ref = g.object_ref(donor);
        dawnhand_remove(&mut g, donor_ref, &[CounterKind::PlusOne]);
        assert!(g.state.announcing.is_none());
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Battlefield);
    }
}
#[test]
fn dawnhand_counter_mana_funding_cannot_double_spend_reserved_counters() {
    for p in [Player::P0, Player::P1] {
        for count in [7, 8] {
            let mut g = hexing_setup(p);
            let (_, card, reference) = dawnhand_link(&mut g, p, "Grizzly Bears");
            let ramos = named(&mut g, p, "Ramos, Dragon Engine", Zone::Battlefield);
            g.add_counter(ramos, CounterKind::PlusOne, count);
            g.state.players[p.index()].mana = [0; 6];
            let action = Action::AnnounceLinkedExileCounterCost {
                card,
                source: reference,
                flash_surcharge: false,
            };
            if count == 7 {
                reject(&mut g, action);
                continue;
            }
            checked(&mut g, action);
            let r = g.object_ref(ramos);
            dawnhand_remove(&mut g, r, &[CounterKind::PlusOne; 3]);
            let mana = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::ActivateMana { card, .. } if *card == ramos))
                .unwrap();
            checked(&mut g, mana);
            assert_eq!(g.counter_count(ramos, CounterKind::PlusOne), 3);
            kindle_pay(&mut g);
            assert_eq!(g.counter_count(ramos, CounterKind::PlusOne), 0);
            settle(&mut g);
            assert_eq!(g.object_view(card).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn dawnhand_variable_creature_spell_requires_existing_counters_and_pays_declared_x() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let (source, card, reference) = dawnhand_link(&mut g, p, "Wildwood Scourge");
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 4];
        reject(
            &mut g,
            Action::AnnounceLinkedExileCounterCost {
                card,
                source: reference,
                flash_surcharge: false,
            },
        );
        g.add_counter(source, CounterKind::Flying, 3);
        dawnhand_announce(&mut g, card, reference);
        assert!(g.x_choice().unwrap().contains(4));
        assert!(!g.x_choice().unwrap().contains(5));
        reject(&mut g, Action::ChooseX { value: 5 });
        checked(&mut g, Action::ChooseX { value: 4 });
        dawnhand_remove(&mut g, reference, &[CounterKind::Flying; 3]);
        kindle_pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.counter_count(card, CounterKind::PlusOne), 4);
        assert_eq!(g.counter_count(source, CounterKind::Flying), 0);
    }
}
#[test]
fn dawnhand_other_exile_permission_does_not_inherit_supplemental_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let (_, card, _) = dawnhand_link(&mut g, p, "Grizzly Bears");
        let object = g.object_ref(card);
        g.grant_exile_cast_permission(p, object);
        checked(&mut g, Action::AnnounceExile { card });
        kindle_pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Battlefield);
    }
}
#[test]
fn dawnhand_evoke_alternative_keeps_supplemental_payment_and_evoked_history() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let (source, card, reference) = dawnhand_link(&mut g, p, "Emptiness");
        g.add_counter(source, CounterKind::Flying, 3);
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 0];
        dawnhand_announce(&mut g, card, reference);
        let cost = g
            .observe(p)
            .cost_choices
            .into_iter()
            .find(|c| {
                c.cost
                    == ManaCost {
                        colored: [2, 0, 0, 0, 0, 0],
                        generic: 0,
                    }
            })
            .unwrap();
        checked(&mut g, Action::ChooseSpellCost { cost: cost.index });
        dawnhand_remove(&mut g, reference, &[CounterKind::Flying; 3]);
        kindle_pay(&mut g);
        assert!(g.state.stack.last().unwrap().cast.unwrap().evoked);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.counter_count(source, CounterKind::Flying), 0);
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Graveyard);
    }
}
#[test]
fn dawnhand_provider_ability_loss_and_reentry_do_not_reuse_old_exile_link() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let (source, card, reference) = dawnhand_link(&mut g, p, "Grizzly Bears");
        let donor = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
        g.add_counter(donor, CounterKind::Flying, 3);
        g.remove_abilities_exact(&[reference], crate::cards::AnimationDuration::Permanent);
        reject(
            &mut g,
            Action::AnnounceLinkedExileCounterCost {
                card,
                source: reference,
                flash_surcharge: false,
            },
        );
        g.move_object(source, Zone::Hand);
        cast(&mut g, source);
        let newer = g.object_ref(source);
        assert_ne!(newer, reference);
        reject(
            &mut g,
            Action::AnnounceLinkedExileCounterCost {
                card,
                source: newer,
                flash_surcharge: false,
            },
        );
    }
}

#[test]
fn dawnhand_actual_exile_activation_unlocks_cast_and_source_loss_preserves_only_old_link() {
    for p in [Player::P0, Player::P1] {
        for departed in [false, true] {
            let mut g = hexing_setup(p);
            let source = named(&mut g, p, "Dawnhand Dissident", Zone::Battlefield);
            let reference = g.object_ref(source);
            let donor = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
            g.add_counter(donor, CounterKind::MinusOne, 1);
            let card = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 1,
                    counter_payment: Counters::default(),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
            let donor_ref = g.object_ref(donor);
            checked(&mut g, Action::ChooseAdditionalCost { object: donor_ref });
            kindle_pay(&mut g);
            if departed {
                g.move_object(source, Zone::Hand);
            }
            settle(&mut g);
            assert_eq!(g.object_view(card).zone, Zone::Exile);
            assert_eq!(g.counter_count(donor, CounterKind::MinusOne), 3);
            if departed {
                cast(&mut g, source);
                let newer = g.object_ref(source);
                reject(
                    &mut g,
                    Action::AnnounceLinkedExileCounterCost {
                        card,
                        source: newer,
                        flash_surcharge: false,
                    },
                );
                continue;
            }
            dawnhand_announce(&mut g, card, reference);
            dawnhand_remove(&mut g, donor_ref, &[CounterKind::MinusOne; 3]);
            kindle_pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(card).zone, Zone::Battlefield);
            assert_eq!(g.counter_count(donor, CounterKind::MinusOne), 0);
        }
    }
}

#[test]
fn dawnhand_insufficient_counters_reject_without_exploring_equivalent_land_subsets() {
    let mut g = hexing_setup(Player::P0);
    let (source, card, reference) = dawnhand_link(&mut g, Player::P0, "Grizzly Bears");
    g.add_counter(source, CounterKind::Flying, 2);
    for _ in 0..12 {
        named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    }
    g.state.players[0].mana = [0; 6];
    reject(
        &mut g,
        Action::AnnounceLinkedExileCounterCost {
            card,
            source: reference,
            flash_surcharge: false,
        },
    );
}

#[test]
fn dawnhand_copy_epochs_keep_queued_exile_packets_separate_from_new_linked_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Dawnhand Dissident", Zone::Battlefield);
        let reference = g.object_ref(source);
        let donor = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
        let donor_ref = g.object_ref(donor);
        let old_card = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(old_card),
            },
        );
        checked(&mut g, Action::ChooseAdditionalCost { object: donor_ref });
        kindle_pay(&mut g);
        assert_eq!(g.state.stack.last().unwrap().ability_origin(), 0);
        // Typed boundary for acquiring the same printed abilities through a new
        // in-place copy effect. The permanent is still the exact same object.
        let definition = g.state.objects[source as usize].card;
        g.state.copy_recipes = Some(
            vec![copies::CopyRow {
                object: reference,
                recipe: CopyRecipe::new(definition),
                ability_origin: 42,
                native: Some(CopyRecipe::new(definition)),
                loyalty_used_turn: None,
            }]
            .into(),
        );
        settle(&mut g);
        g.add_counter(source, CounterKind::PlusOne, 1);
        assert_eq!(g.object_ref(source), reference);
        assert!(
            g.observe(p)
                .linked_exile
                .iter()
                .any(|link| link.exiled == g.object_ref(old_card) && link.ability_origin == 0)
        );
        reject(
            &mut g,
            Action::AnnounceLinkedExileCounterCost {
                card: old_card,
                source: reference,
                flash_surcharge: false,
            },
        );
        g.state.objects[source as usize].tapped = false;
        let new_card = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(new_card),
            },
        );
        checked(&mut g, Action::ChooseAdditionalCost { object: donor_ref });
        kindle_pay(&mut g);
        assert_eq!(g.state.stack.last().unwrap().ability_origin(), 42);
        g.state.copy_recipes.as_mut().unwrap()[0].ability_origin = 84;
        settle(&mut g);
        assert!(
            g.observe(p)
                .linked_exile
                .iter()
                .any(|link| link.exiled == g.object_ref(new_card) && link.ability_origin == 42)
        );
        reject(
            &mut g,
            Action::AnnounceLinkedExileCounterCost {
                card: new_card,
                source: reference,
                flash_surcharge: false,
            },
        );
        g.state.copy_recipes.as_mut().unwrap()[0].ability_origin = 42;
        dawnhand_announce(&mut g, new_card, reference);
        dawnhand_remove(&mut g, donor_ref, &[CounterKind::MinusOne; 3]);
        kindle_pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(new_card).zone, Zone::Battlefield);
        assert_eq!(g.object_view(old_card).zone, Zone::Exile);
        g.state.copy_recipes = None;
        assert_eq!(g.linked_counter_permission(p, old_card, reference), Some(3));
    }
}

fn spinerock_open(g: &mut Game, p: Player, target: CardId) -> (CardId, SpellView) {
    let source = named(g, p, "Spinerock Tyrant", Zone::Battlefield);
    let spell = hexing_open(g, p, "Shock", Some(Target::Card(target)));
    let original = g
        .state
        .stack
        .iter()
        .find(|item| item.kind == StackKind::Spell && item.card == spell)
        .unwrap()
        .clone();
    (source, original)
}
fn spinerock_choice(g: &mut Game) {
    for _ in 0..50 {
        if matches!(g.state.pending, Pending::SpellCopy { .. }) {
            return;
        }
        advance(g);
    }
    panic!("no optional copy decision: {:?}", g.state.pending);
}
#[test]
fn spinerock_paid_body_flying_wither_and_actual_blocked_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Spinerock Tyrant", Zone::Hand);
        cast(&mut g, source);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 6,
                toughness: 6
            }
        );
        assert!(
            g.keywords(source)
                .contains(Keywords::FLYING.union(Keywords::WITHER))
        );
        g.state.objects[source as usize].summoning_sick = false;
        let blocker = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let reference = g.object_ref(blocker);
        g.grant_keyword_effects(&[blocker], reference, Keywords::REACH);
        g.state.attackers.push(Attacker {
            defender: Some(CombatDefender::Player(p.other())),
            card: source,
            blocked: true,
        });
        g.state.blocks.push((blocker, source));
        g.state.blocking.push(blocker);
        g.state.step = Step::Blockers;
        g.give_priority();
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.counter_count(blocker, CounterKind::MinusOne), 6);
        assert_eq!(g.object_view(blocker).damage, 0);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
}
#[test]
fn spinerock_optional_copy_decline_leaves_original_damage_unmodified() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let target = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let (_, original) = spinerock_open(&mut g, p, target);
        spinerock_choice(&mut g);
        assert!(!g.keywords(original.card).contains(Keywords::WITHER));
        checked(&mut g, Action::ChooseSpellCopy { copy: false });
        settle(&mut g);
        assert_eq!(g.object_view(target).damage, 2);
        assert_eq!(g.counter_count(target, CounterKind::MinusOne), 0);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 1);
    }
}
#[test]
fn spinerock_copy_and_original_gain_wither_independently_and_retarget_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let target = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let second = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let (source, original) = spinerock_open(&mut g, p, target);
        spinerock_choice(&mut g);
        checked(&mut g, Action::ChooseSpellCopy { copy: true });
        let copying = g.retarget_view().unwrap().item;
        assert!(g.keywords(original.card).contains(Keywords::WITHER));
        assert!(g.keywords(copying.card).contains(Keywords::WITHER));
        assert!(!copying.source_keywords.contains(Keywords::WITHER));
        for recipient in [original.object, copying.object] {
            assert!(
                g.state
                    .roles
                    .as_ref()
                    .unwrap()
                    .keyword_effects
                    .iter()
                    .any(|row| {
                        row.recipient == recipient
                            && row.origin
                                == KeywordEffectOrigin::Resolution {
                                    source: g.object_ref(source),
                                }
                            && row.expires_turn.is_none()
                    })
            );
        }
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(second),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        for id in [target, second] {
            assert_eq!(g.counter_count(id, CounterKind::MinusOne), 2);
            assert_eq!(g.object_view(id).damage, 0);
        }
        assert_eq!(g.object_view(copying.card).zone, Zone::Outside);
        assert!(!g.keywords(original.card).contains(Keywords::WITHER));
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 1);
    }
}
#[test]
fn spinerock_captured_spell_copy_survives_countered_original_and_departed_dragon() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let target = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let (source, original) = spinerock_open(&mut g, p, target);
        g.move_object(source, Zone::Graveyard);
        checked(&mut g, Action::Pass);
        let negate = hexing_open(
            &mut g,
            p.other(),
            "Negate",
            Some(Target::Spell(original.card)),
        );
        spinerock_choice(&mut g);
        assert_eq!(g.object_view(original.card).zone, Zone::Graveyard);
        assert_eq!(g.object_view(negate).zone, Zone::Graveyard);
        checked(&mut g, Action::ChooseSpellCopy { copy: true });
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.counter_count(target, CounterKind::MinusOne), 2);
        assert_eq!(g.object_view(target).damage, 0);
    }
}
#[test]
fn spinerock_target_instances_exclude_zero_two_and_repeated_same_object_groups() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Spinerock Tyrant", Zone::Battlefield);
        let draw = named(&mut g, p, "Divination", Zone::Hand);
        cast(&mut g, draw);
        assert!(g.state.stack.is_empty());
        let donor = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
        let target = named(&mut g, p.other(), "Appendage Amalgam", Zone::Battlefield);
        let spell = named(&mut g, p, "Pyrrhic Strike", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        let donor_ref = g.object_ref(donor);
        checked(
            &mut g,
            Action::ChooseOptionalAdditionalCost {
                object: Some(donor_ref),
            },
        );
        checked(&mut g, Action::ChooseMode { mode: 2 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        kindle_pay(&mut g);
        let packet = g
            .state
            .stack
            .iter()
            .find(|item| item.card == spell && item.kind == StackKind::Spell)
            .unwrap();
        assert_eq!(packet.targets.len(), 2);
        assert_eq!(packet.targets[0].target, packet.targets[1].target);
        assert!(
            g.state
                .pending_triggers
                .iter()
                .chain(&g.state.stack)
                .all(|item| item.card != source || item.copy_event.is_none())
        );
        settle(&mut g);
    }
}
#[test]
fn spinerock_other_players_single_target_cast_does_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Spinerock Tyrant", Zone::Battlefield);
        let target = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
        g.state.priority = p.other();
        hexing_open(&mut g, p.other(), "Shock", Some(Target::Card(target)));
        assert!(
            g.state
                .pending_triggers
                .iter()
                .chain(&g.state.stack)
                .all(|item| item.card != source || item.copy_event.is_none())
        );
        settle(&mut g);
        assert_eq!(g.object_view(target).damage, 2);
        assert_eq!(g.counter_count(target, CounterKind::MinusOne), 0);
    }
}
#[test]
fn spinerock_failed_optional_copy_is_atomic_and_does_not_grant_wither() {
    let p = Player::P0;
    let mut g = hexing_setup(p);
    let target = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
    let (_, original) = spinerock_open(&mut g, p, target);
    spinerock_choice(&mut g);
    g.state.next_stack_id = u64::MAX;
    let before = g.snapshot();
    let revision = g.revision();
    let views = [
        serde_json::to_value(g.observe(p)).unwrap(),
        serde_json::to_value(g.observe(p.other())).unwrap(),
    ];
    assert!(matches!(
        g.apply(p, Action::ChooseSpellCopy { copy: true }),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    assert_eq!(serde_json::to_value(g.observe(p)).unwrap(), views[0]);
    assert_eq!(
        serde_json::to_value(g.observe(p.other())).unwrap(),
        views[1]
    );
    assert!(!g.keywords(original.card).contains(Keywords::WITHER));
    checked(&mut g, Action::ChooseSpellCopy { copy: false });
    settle(&mut g);
}
#[test]
fn spinerock_granted_wither_is_not_inherited_by_an_unrelated_spell_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        // Storm will make exactly one independent copy of the second spell.
        hexing_open(&mut g, p, "Shock", Some(Target::Player(p.other())));
        settle(&mut g);
        let storm = named(&mut g, p, "Thousand-Year Storm", Zone::Battlefield);
        let target = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let (source, original) = spinerock_open(&mut g, p, target);
        assert!(matches!(g.state.pending, Pending::OrderTriggers { .. }));
        let storm_id = g
            .state
            .ordering_triggers
            .iter()
            .find(|s| s.card == storm)
            .unwrap()
            .id;
        checked(&mut g, Action::PutTrigger { id: storm_id });
        // Put Spinerock above Storm; its own copy resolves first.
        if matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            let source_id = g
                .state
                .ordering_triggers
                .iter()
                .find(|s| s.card == source)
                .unwrap()
                .id;
            checked(&mut g, Action::PutTrigger { id: source_id });
        }
        spinerock_choice(&mut g);
        checked(&mut g, Action::ChooseSpellCopy { copy: true });
        let own_copy = g.retarget_view().unwrap().item;
        assert!(g.keywords(own_copy.card).contains(Keywords::WITHER));
        checked(&mut g, Action::FinishStackTargets);
        for _ in 0..50 {
            if g.retarget_view().is_some() {
                break;
            }
            advance(&mut g);
        }
        assert_eq!(g.counter_count(target, CounterKind::MinusOne), 2);
        let storm_copy = g.retarget_view().unwrap().item;
        assert!(!g.keywords(storm_copy.card).contains(Keywords::WITHER));
        assert!(g.keywords(original.card).contains(Keywords::WITHER));
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.counter_count(target, CounterKind::MinusOne), 4);
        assert_eq!(g.object_view(target).damage, 2);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 2);
    }
}

#[test]
fn lavaleaper_paid_body_grants_both_players_haste_and_removal_stops_it() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p, "Lavaleaper", Zone::Hand);
        cast(&mut g, source);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        assert!(g.keywords(source).contains(Keywords::HASTE));
        let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        for creature in [own, foe] {
            assert!(g.keywords(creature).contains(Keywords::HASTE));
            g.state.objects[creature as usize].summoning_sick = true;
        }
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        assert!(g.legal_actions().contains(&Action::Attack { card: own }));
        g.move_object(source, Zone::Hand);
        assert!(!g.keywords(own).contains(Keywords::HASTE));
        assert!(!g.keywords(foe).contains(Keywords::HASTE));
        reject(&mut g, Action::Attack { card: own });
    }
}
#[test]
fn lavaleaper_all_six_basic_land_types_add_the_type_actually_produced_for_either_player() {
    for p in [Player::P0, Player::P1] {
        for (color, name) in ["Plains", "Island", "Swamp", "Mountain", "Forest", "Wastes"]
            .into_iter()
            .enumerate()
        {
            let mut g = hexing_setup(p);
            named(&mut g, p.other(), "Lavaleaper", Zone::Battlefield);
            let land = named(&mut g, p, name, Zone::Battlefield);
            g.state.players[p.index()].mana = [0; 6];
            let action = if color == 5 {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::ActivateMana { card, .. } if *card == land))
                    .unwrap()
            } else {
                Action::TapMana { card: land }
            };
            checked(&mut g, action);
            let mut expected = [0; 6];
            expected[color] = 2;
            assert_eq!(g.state.players[p.index()].mana, expected);
            assert_eq!(g.state.players[p.other().index()].mana, [30; 6]);
            assert!(g.state.stack.is_empty());
            assert!(g.state.pending_triggers.is_empty());
        }
    }
}
#[test]
fn lavaleaper_multiple_providers_use_their_own_trigger_multipliers_and_ability_status() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let first = named(&mut g, p.other(), "Lavaleaper", Zone::Battlefield);
        named(&mut g, p.other(), "Twinflame Travelers", Zone::Battlefield);
        let second = named(&mut g, p, "Lavaleaper", Zone::Battlefield);
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::TapMana { card: forest });
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 4, 0]);
        let first_ref = g.object_ref(first);
        g.remove_abilities_exact(&[first_ref], crate::cards::AnimationDuration::Permanent);
        let forest2 = named(&mut g, p, "Forest", Zone::Battlefield);
        checked(&mut g, Action::TapMana { card: forest2 });
        assert_eq!(g.state.players[p.index()].mana[4], 6);
        g.move_object(second, Zone::Hand);
        let forest3 = named(&mut g, p, "Forest", Zone::Battlefield);
        checked(&mut g, Action::TapMana { card: forest3 });
        assert_eq!(g.state.players[p.index()].mana[4], 7);
    }
}
#[test]
fn lavaleaper_exact_funding_uses_bonus_quantity_without_inventing_other_mana_types() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        named(&mut g, p, "Lavaleaper", Zone::Battlefield);
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let mountain = named(&mut g, p, "Mountain", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let giant = named(&mut g, p, "Hill Giant", Zone::Hand);
        let draw = named(&mut g, p, "Divination", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: draw });
        checked(&mut g, Action::AnnounceCast { card: giant });
        checked(&mut g, Action::TapMana { card: forest });
        checked(&mut g, Action::TapMana { card: mountain });
        kindle_pay(&mut g);
        settle(&mut g);
        assert!(g.on_battlefield(giant));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn lavaleaper_granted_land_mana_gets_one_bonus_not_one_per_mana_and_nonbasic_is_excluded() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        named(&mut g, p, "Lavaleaper", Zone::Battlefield);
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let aura = named(&mut g, p.other(), "New Horizons", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(forest));
        g.state.players[p.index()].mana = [0; 6];
        let grantor = g.object_ref(aura);
        checked(
            &mut g,
            Action::TapGrantedMana {
                card: forest,
                grantor,
                production: [0, 0, 0, 2, 0, 0],
            },
        );
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 3, 0, 0]);
        let nonbasic = named(&mut g, p, "Hallowed Fountain", Zone::Battlefield);
        let tap = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ActivateMana { card, .. } if *card == nonbasic))
            .unwrap();
        checked(&mut g, tap);
        assert_eq!(g.state.players[p.index()].mana.iter().sum::<u32>(), 4);
    }
}
#[test]
fn lavaleaper_mixed_event_choices_are_owned_and_keep_frozen_types_and_multiplicity() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        let source = named(&mut g, p.other(), "Lavaleaper", Zone::Battlefield);
        let travelers = named(&mut g, p.other(), "Twinflame Travelers", Zone::Battlefield);
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        // Narrow event boundary: no currently admitted basic-land ability makes
        // this mixed vector, but all three types must remain valid choices.
        let bonus = g.capture_mana_tap_bonus(forest, [1, 1, 0, 0, 0, 1]);
        g.move_object(source, Zone::Graveyard);
        g.move_object(travelers, Zone::Graveyard);
        g.state.players[p.index()].mana = [0; 6];
        g.finish_mana_tap_bonus(p, bonus);
        assert!(g.advance_mana_tap_bonus());
        assert_eq!(g.observe(p).mana_choices.len(), 3);
        assert!(g.observe(p.other()).mana_choices.is_empty());
        reject(
            &mut g,
            Action::ChooseMana {
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        checked(
            &mut g,
            Action::ChooseMana {
                production: [0, 0, 0, 0, 0, 1],
            },
        );
        assert!(matches!(g.state.pending, Pending::EffectMana { .. }));
        checked(
            &mut g,
            Action::ChooseMana {
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        assert_eq!(g.state.pending, Pending::Priority);
        assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 0, 1]);
        assert!(g.state.mana_lots.is_none());
    }
}
#[test]
fn lavaleaper_mixed_bonus_overflow_is_atomic_and_another_type_remains_payable() {
    let p = Player::P0;
    let mut g = hexing_setup(p);
    named(&mut g, p, "Lavaleaper", Zone::Battlefield);
    let forest = named(&mut g, p, "Forest", Zone::Battlefield);
    let bonus = g.capture_mana_tap_bonus(forest, [0, 0, 0, 1, 0, 1]);
    g.state.players[p.index()].mana = [0, 0, 0, u32::MAX, 0, 0];
    g.finish_mana_tap_bonus(p, bonus);
    assert!(g.advance_mana_tap_bonus());
    reject(
        &mut g,
        Action::ChooseMana {
            production: [0, 0, 0, 1, 0, 0],
        },
    );
    checked(
        &mut g,
        Action::ChooseMana {
            production: [0, 0, 0, 0, 0, 1],
        },
    );
    assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, u32::MAX, 0, 1]);
}
#[test]
fn lavaleaper_pending_mana_bonus_waits_for_replacements_and_defers_sbas() {
    let p = Player::P0;
    let mut g = hexing_setup(p);
    named(&mut g, p, "Lavaleaper", Zone::Battlefield);
    let forest = named(&mut g, p, "Forest", Zone::Battlefield);
    let bonus = g.capture_mana_tap_bonus(forest, [1, 1, 0, 0, 0, 0]);
    let victim = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    named(&mut g, p, "Doubling Season", Zone::Battlefield);
    named(&mut g, p, "Doubling Season", Zone::Battlefield);
    g.effect_counter(
        victim,
        CounterKind::MinusOne,
        1,
        replacements::ResumeMode::Pending,
    );
    assert!(g.advance_replacements(false));
    assert!(matches!(g.state.pending, Pending::Replacement { .. }));
    g.finish_mana_tap_bonus(p, bonus);
    for _ in 0..10 {
        if matches!(g.state.pending, Pending::EffectMana { .. }) {
            break;
        }
        advance(&mut g);
    }
    assert!(matches!(g.state.pending, Pending::EffectMana { .. }));
    assert!(
        g.on_battlefield(victim),
        "no SBA inside triggered mana resolution"
    );
    assert_eq!(g.counter_count(victim, CounterKind::MinusOne), 4);
    checked(
        &mut g,
        Action::ChooseMana {
            production: [1, 0, 0, 0, 0, 0],
        },
    );
    assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
}
#[test]
fn lavaleaper_mixed_mana_choice_resumes_an_existing_cast_payment_window() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        named(&mut g, p, "Lavaleaper", Zone::Battlefield);
        named(&mut g, p, "Lavaleaper", Zone::Battlefield);
        let forests = [
            named(&mut g, p, "Forest", Zone::Battlefield),
            named(&mut g, p, "Forest", Zone::Battlefield),
        ];
        g.state.players[p.index()].mana = [0; 6];
        let spell = named(&mut g, p, "Gigantosaurus", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        let bonus = g.capture_mana_tap_bonus(forests[0], [1, 0, 0, 0, 0, 1]);
        g.finish_mana_tap_bonus(p, bonus);
        assert!(g.advance_mana_tap_bonus());
        checked(
            &mut g,
            Action::ChooseMana {
                production: [1, 0, 0, 0, 0, 0],
            },
        );
        checked(
            &mut g,
            Action::ChooseMana {
                production: [0, 0, 0, 0, 0, 1],
            },
        );
        assert!(matches!(g.state.pending, Pending::PayCosts { .. }));
        for card in forests {
            checked(&mut g, Action::TapMana { card });
        }
        kindle_pay(&mut g);
        settle(&mut g);
        assert!(g.on_battlefield(spell));
    }
}
#[test]
fn lavaleaper_guided_wastes_activation_captures_colorless_bonus_without_stack_use() {
    for p in [Player::P0, Player::P1] {
        let mut g = hexing_setup(p);
        named(&mut g, p.other(), "Lavaleaper", Zone::Battlefield);
        let land = named(&mut g, p, "Wastes", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::AnnounceMana {
                card: land,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 1],
            },
        );
        while g.state.announcing.is_some() {
            advance(&mut g);
        }
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 2]);
        assert_eq!(g.state.pending, Pending::Priority);
        assert!(g.state.stack.is_empty());
    }
}
