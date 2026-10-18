//! Paid rule interactions for the four gated replacement/protection consumers.
use super::ability_timeline_tests::{checked, next_turn, settle, setup};
use super::tests::spawn;
use super::*;

fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    let id = spawn(g, p, card, zone);
    g.register_zone_rule_card(card);
    id
}
fn reject(g: &mut Game, action: Action) {
    let snapshot = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.revision(), revision);
}
fn finish(g: &mut Game, sacrifice: Option<CardId>) {
    for _ in 0..80 {
        if g.state.announcing.is_none()
            || matches!(
                g.state.pending,
                Pending::ZoneReplacement { .. } | Pending::ManaLotSpend { .. }
            )
        {
            return;
        }
        let actions = g.legal_actions();
        let action = actions.iter().find(|a| matches!(a, Action::ChooseAdditionalCost { object } if Some(object.card) == sacrifice))
            .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. })))
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede))).unwrap().clone();
        checked(g, action);
    }
    panic!("unfinished announcement");
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let card = fixture(g, p, name, Zone::Hand);
    checked(g, Action::AnnounceCast { card });
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    finish(g, None);
    card
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    for _ in 0..200 {
        if stop(g) {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::FinishTargets
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, action);
    }
    panic!("missing decision {:?}", g.state.pending);
}

#[test]
fn zone_competing_self_shuffle_and_death_exile_preserve_owned_cost_choice_and_current_errors() {
    assert_eq!(std::mem::size_of::<Object>(), 144);
    assert_eq!(std::mem::size_of::<Pending>(), 40);
    assert_eq!(std::mem::size_of::<ManaProduction>(), 32);
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let colossus = paid(&mut g, p, "Darksteel Colossus", &[]);
        settle(&mut g);
        assert_eq!(
            g.stats(colossus),
            Stats {
                power: 11,
                toughness: 11
            }
        );
        assert!(
            g.keywords(colossus)
                .contains(Keywords::TRAMPLE.union(Keywords::INDESTRUCTIBLE))
        );
        let ghoul = paid(&mut g, p, "Hungry Ghoul", &[]);
        settle(&mut g);
        paid(&mut g, p, "Scorching Dragonfire", &[Target::Card(colossus)]);
        settle(&mut g);
        assert_eq!(g.object_view(colossus).zone, Zone::Battlefield);
        let deaths = g.state.creatures_died_this_turn;
        let library = g.state.players[p.index()].library.len();
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: ghoul,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        finish(&mut g, Some(colossus));
        assert_eq!(g.actor(), Some(p));
        let view = g.observe(p).zone_replacement_choice.unwrap();
        assert_eq!(view.card.name(), "Darksteel Colossus");
        assert_eq!(
            view.choices,
            [
                ZoneReplacementKind::Exile,
                ZoneReplacementKind::RevealShuffle
            ]
        );
        assert!(g.observe(p.other()).zone_replacement_choice.is_none());
        assert_eq!(
            g.object_view(colossus).zone,
            Zone::Battlefield,
            "event not yet committed"
        );
        reject(
            &mut g,
            Action::ChooseZoneReplacement {
                effect: ZoneReplacementKind::Battlefield,
            },
        );
        let choice = g.snapshot();
        let mut exhausted = g.clone();
        exhausted.revision = u64::MAX;
        let current = exhausted.snapshot();
        assert!(matches!(
            exhausted.apply(
                p,
                Action::ChooseZoneReplacement {
                    effect: ZoneReplacementKind::Exile
                }
            ),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(exhausted.snapshot(), current);
        assert_eq!(exhausted.revision(), u64::MAX);
        checked(
            &mut g,
            Action::ChooseZoneReplacement {
                effect: ZoneReplacementKind::Exile,
            },
        );
        assert_eq!(g.object_view(colossus).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].library.len(), library);
        assert_eq!(g.state.creatures_died_this_turn, deaths);
        settle(&mut g);
        assert_eq!(g.counter_count(ghoul, CounterKind::PlusOne), 1);
        g.restore(&choice).unwrap();
        checked(
            &mut g,
            Action::ChooseZoneReplacement {
                effect: ZoneReplacementKind::RevealShuffle,
            },
        );
        assert_eq!(g.object_view(colossus).zone, Zone::Library);
        assert_eq!(g.state.players[p.index()].library.len(), library + 1);
        assert_eq!(g.state.creatures_died_this_turn, deaths);
        assert_eq!(
            g.observe(p.other()).zone_reveals,
            [ZoneRevealView {
                card: Card::from_name("Darksteel Colossus").unwrap(),
                owner: p,
                from: Zone::Battlefield
            }]
        );
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == colossus)
        );
        settle(&mut g);
        assert_eq!(g.counter_count(ghoul, CounterKind::PlusOne), 1);
    }
}

#[test]
fn zone_dryad_hybrid_and_chosen_adventure_counter_use_pre_departure_spell_facts() {
    for p in [Player::P0, Player::P1] {
        for color in [0, 4] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.index()].mana[color] = 1;
            let dryad = paid(&mut g, p, "Dryad Militant", &[]);
            settle(&mut g);
            assert_eq!(
                g.stats(dryad),
                Stats {
                    power: 2,
                    toughness: 1
                }
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            g.add_mana(p, [30; 6]);
            let opt = paid(&mut g, p, "Opt", &[]);
            settle(&mut g);
            assert_eq!(g.object_view(opt).zone, Zone::Exile);
            let wrath = paid(&mut g, p, "Day of Judgment", &[]);
            settle(&mut g);
            assert_eq!(g.object_view(dryad).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(wrath).zone,
                Zone::Graveyard,
                "Dryad left before the resolving spell"
            );
        }
        let mut g = setup(p.other());
        fixture(&mut g, p, "Dryad Militant", Zone::Battlefield);
        let adventure = fixture(
            &mut g,
            p.other(),
            "Gollum, Silent Slinker // Meager Meal",
            Zone::Hand,
        );
        checked(&mut g, Action::AnnounceAdventure { card: adventure });
        finish(&mut g, None);
        checked(&mut g, Action::Pass);
        paid(&mut g, p, "Negate", &[Target::Spell(adventure)]);
        settle(&mut g);
        assert_eq!(g.object_view(adventure).zone, Zone::Exile);
        assert!(g.object_view(adventure).card.is_creature());
        assert!(!g.can_play_adventure(p.other(), adventure));
    }
}

#[test]
fn zone_self_shuffle_from_library_and_opponent_discard_reveal_without_slot_identity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let card = fixture(&mut g, p, "Darksteel Colossus", Zone::Library);
        let original = g.object_ref(card);
        let before = g.object_view(card);
        let event = g.transition_object(before, Zone::Graveyard, false);
        assert!(!event.moved);
        assert_eq!(event.destination, Zone::Library);
        assert_eq!(g.object_ref(card), original);
        let view = g.observe(p.other());
        assert_eq!(view.zone_reveals.len(), 1);
        assert!(!view.objects.iter().any(|o| o.id == card));
        assert!(
            serde_json::to_value(&view.zone_reveals).unwrap()[0]
                .get("object")
                .is_none()
        );
        let mut g = setup(p.other());
        let card = fixture(&mut g, p, "Darksteel Colossus", Zone::Hand);
        paid(&mut g, p.other(), "Burglar Rat", &[]);
        until(
            &mut g,
            |g| matches!(g.state.pending, Pending::EffectDiscard { player, .. } if player == p),
        );
        checked(&mut g, Action::Discard { card });
        assert_eq!(g.object_view(card).zone, Zone::Library);
        assert_eq!(g.observe(p.other()).zone_reveals.len(), 1);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == card));
    }
}

#[test]
fn zone_simultaneous_provider_departures_use_frozen_world_in_both_member_orders() {
    for p in [Player::P0, Player::P1] {
        for reverse in [false, true] {
            let mut g = setup(p);
            let dryad = fixture(&mut g, p, "Dryad Militant", Zone::Battlefield);
            let spell = fixture(&mut g, p.other(), "Shock", Zone::Stack);
            let ids = if reverse {
                [spell, dryad]
            } else {
                [dryad, spell]
            };
            let events = g.move_objects_raw(&ids, Zone::Graveyard);
            g.finish_zone_events(&events, &[]);
            assert_eq!(g.object_view(dryad).zone, Zone::Graveyard);
            assert_eq!(g.object_view(spell).zone, Zone::Exile);
        }
    }
}

#[test]
fn zone_wilt_hybrid_two_colors_and_opponent_discard_use_real_entry_triggers() {
    for p in [Player::P0, Player::P1] {
        for white in 0..=3 {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [white, 0, 0, 0, 3 - white, 1];
            let green = fixture(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            g.state.objects[green as usize].tapped = true;
            let white_creature = fixture(&mut g, p, "Savannah Lions", Zone::Battlefield);
            let both = fixture(&mut g, p, "Dryad Militant", Zone::Battlefield);
            let enemy = fixture(&mut g, p.other(), "Dryad Militant", Zone::Battlefield);
            let liege = paid(&mut g, p, "Wilt-Leaf Liege", &[]);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.stats(liege),
                Stats {
                    power: 4,
                    toughness: 4
                }
            );
            assert_eq!(
                g.stats(green),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
            assert_eq!(
                g.stats(white_creature),
                Stats {
                    power: 3,
                    toughness: 2
                }
            );
            assert_eq!(
                g.stats(both),
                Stats {
                    power: 4,
                    toughness: 3
                }
            );
            assert_eq!(
                g.stats(enemy),
                Stats {
                    power: 2,
                    toughness: 1
                }
            );
        }
        let mut g = setup(p.other());
        fixture(&mut g, p, "Good-Fortune Unicorn", Zone::Battlefield);
        fixture(&mut g, p, "Doubling Season", Zone::Battlefield);
        fixture(
            &mut g,
            p.other(),
            "Authority of the Consuls",
            Zone::Battlefield,
        );
        let liege = fixture(&mut g, p, "Wilt-Leaf Liege", Zone::Hand);
        paid(&mut g, p.other(), "Burglar Rat", &[]);
        until(
            &mut g,
            |g| matches!(g.state.pending, Pending::EffectDiscard { player, .. } if player == p),
        );
        checked(&mut g, Action::Discard { card: liege });
        settle(&mut g);
        assert_eq!(g.object_view(liege).zone, Zone::Battlefield);
        assert!(g.object_view(liege).tapped);
        assert_eq!(g.counter_count(liege, CounterKind::PlusOne), 2);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
        assert_eq!(g.state.creatures_died_this_turn, 0);
        let mut g = setup(p);
        let liege = fixture(&mut g, p, "Wilt-Leaf Liege", Zone::Hand);
        let thrill = fixture(&mut g, p, "Thrill of Possibility", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: thrill });
        finish(&mut g, Some(liege));
        assert_eq!(
            g.object_view(liege).zone,
            Zone::Graveyard,
            "own discard cost is not an opposing effect"
        );
    }
}

#[test]
fn zone_protection_targets_blocks_attachments_damage_and_countering_keep_distinct_rules() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let progenitus = paid(&mut g, p, "Progenitus", &[]);
        checked(&mut g, Action::Pass);
        paid(
            &mut g,
            p.other(),
            "Essence Scatter",
            &[Target::Spell(progenitus)],
        );
        settle(&mut g);
        assert_eq!(
            g.object_view(progenitus).zone,
            Zone::Library,
            "protection does not protect a creature spell"
        );
        let progenitus = paid(&mut g, p, "Progenitus", &[]);
        settle(&mut g);
        let growth = fixture(&mut g, p, "Giant Growth", Zone::Hand);
        reject(
            &mut g,
            Action::Cast {
                card: growth,
                target: Some(Target::Card(progenitus)),
                payment: [0, 0, 0, 0, 1, 0],
            },
        );
        let enemy = fixture(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let equipment = fixture(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
        assert!(!g.attach_equipment(g.object_ref(equipment), g.object_ref(progenitus)));
        let aura = fixture(&mut g, p, "Ethereal Armor", Zone::Hand);
        assert!(!g.aura_can_enchant(
            g.state.objects[aura as usize].card,
            p,
            g.object_ref(progenitus)
        ));
        let source = g.damage_source(enemy);
        g.apply_damage_batch(&[DamageEvent {
            preventable: true,
            source,
            target: Target::Card(progenitus),
            amount: 5,
            combat: false,
        }]);
        g.advance_replacements(false);
        assert_eq!(g.object_view(progenitus).damage, 0);
        g.apply_damage_batch(&[DamageEvent {
            preventable: false,
            source,
            target: Target::Card(progenitus),
            amount: 1,
            combat: false,
        }]);
        assert_eq!(g.object_view(progenitus).damage, 1);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p && g.state.pending == Pending::Attackers
        });
        checked(&mut g, Action::Attack { card: progenitus });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        assert!(!g.can_block_pair(enemy, progenitus));
        until(&mut g, |g| {
            g.state.active == p
                && g.state.step == Step::Main2
                && g.state.pending == Pending::Priority
        });
        g.add_mana(p, [30; 6]);
        paid(&mut g, p, "Day of Judgment", &[]);
        settle(&mut g);
        assert_eq!(
            g.object_view(progenitus).zone,
            Zone::Library,
            "non-targeted destruction still works"
        );
    }
}

#[test]
fn zone_nested_mana_choice_preserves_controller_owner_and_exact_replay() {
    for owner in [Player::P0, Player::P1] {
        for amount in [0, 1] {
            for effect in [
                ZoneReplacementKind::Exile,
                ZoneReplacementKind::RevealShuffle,
            ] {
                let controller = owner.other();
                let mut g = setup(controller);
                // A controlled permanent keeps its owner's library destination.
                let colossus = fixture(&mut g, owner, "Darksteel Colossus", Zone::Battlefield);
                g.state.objects[colossus as usize].controller = controller;
                let ghoul = paid(&mut g, controller, "Hungry Ghoul", &[]);
                settle(&mut g);
                paid(
                    &mut g,
                    controller,
                    "Scorching Dragonfire",
                    &[Target::Card(colossus)],
                );
                settle(&mut g);
                let orb = fixture(
                    &mut g,
                    controller,
                    "Carnelian Orb of Dragonkind",
                    Zone::Battlefield,
                );
                g.state.players[controller.index()].mana = [0, 0, 0, 1, 0, 0];
                checked(
                    &mut g,
                    Action::ActivateMana {
                        card: orb,
                        ability: 0,
                        production: [0, 0, 0, 1, 0, 0],
                        payment: [0; 6],
                        counter_payment: Counters::default(),
                    },
                );
                let libraries = [
                    g.state.players[0].library.len(),
                    g.state.players[1].library.len(),
                ];
                let deaths = g.state.creatures_died_this_turn;
                checked(
                    &mut g,
                    Action::AnnounceActivate {
                        card: ghoul,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                finish(&mut g, Some(colossus));
                assert!(
                    matches!(g.state.pending, Pending::ManaLotSpend { player } if player == controller)
                );
                assert_eq!(g.object_view(colossus).zone, Zone::Battlefield);
                checked(&mut g, Action::ChooseManaLotSpend { amount });
                assert!(
                    matches!(g.state.pending, Pending::ZoneReplacement { player } if player == controller)
                );
                assert_eq!(
                    g.observe(controller).zone_replacement_choice.unwrap().owner,
                    owner
                );
                assert!(g.observe(owner).zone_replacement_choice.is_none());
                checked(&mut g, Action::ChooseZoneReplacement { effect });
                assert_eq!(
                    g.object_view(colossus).zone,
                    if effect == ZoneReplacementKind::Exile {
                        Zone::Exile
                    } else {
                        Zone::Library
                    }
                );
                assert_eq!(
                    g.state.players[owner.index()].library.len(),
                    libraries[owner.index()]
                        + usize::from(effect == ZoneReplacementKind::RevealShuffle)
                );
                assert_eq!(
                    g.state.players[controller.index()].library.len(),
                    libraries[controller.index()]
                );
                assert_eq!(g.state.players[controller.index()].mana, [0, 0, 0, 1, 0, 0]);
                assert_eq!(g.state.creatures_died_this_turn, deaths);
                settle(&mut g);
                assert_eq!(g.counter_count(ghoul, CounterKind::PlusOne), 1);
            }
        }
    }
}

#[test]
fn zone_protection_prevention_orders_with_doubling_and_excludes_unpreventable_damage() {
    for p in [Player::P0, Player::P1] {
        for prevent_first in [false, true] {
            let mut g = setup(p);
            let progenitus = paid(&mut g, p, "Progenitus", &[]);
            settle(&mut g);
            let dealer = fixture(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let doubler = fixture(&mut g, p.other(), "Twinflame Tyrant", Zone::Battlefield);
            let source = g.damage_source(dealer);
            g.apply_damage_batch(&[DamageEvent {
                preventable: true,
                source,
                target: Target::Card(progenitus),
                amount: 2,
                combat: false,
            }]);
            assert!(g.advance_replacements(false));
            assert_eq!(g.actor(), Some(p));
            let view = g.observe(p).replacement_choice.unwrap();
            assert_eq!(view.effects.len(), 2);
            assert_eq!(view.damage.unwrap().amount, 2);
            let chosen = if prevent_first { progenitus } else { doubler };
            let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::ChooseReplacement { source, .. } if source.card == chosen)).unwrap();
            checked(&mut g, action);
            assert!(g.state.replacements.is_none());
            assert_eq!(g.object_view(progenitus).damage, 0);
            assert!(!g.object_view(progenitus).damaged_this_turn);
            assert!(!g.object_view(dealer).dealt_damage_this_turn);
            g.apply_damage_batch(&[DamageEvent {
                preventable: false,
                source,
                target: Target::Card(progenitus),
                amount: 1,
                combat: false,
            }]);
            assert!(!g.advance_replacements(false));
            assert_eq!(g.object_view(progenitus).damage, 2);
            assert!(g.object_view(dealer).dealt_damage_this_turn);
        }
    }
}
