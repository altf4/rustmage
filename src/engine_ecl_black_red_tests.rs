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
