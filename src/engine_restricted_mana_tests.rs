//! Restricted mana: exact purpose, live source facts, paid whole cards and owned replay.
use super::super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
use crate::ManaRetention;
fn announce(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}
fn mana(card: CardId, ability: u16, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), a.clone()),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn pay(g: &mut Game) {
    for _ in 0..20 {
        if g.state.announcing.is_none() && !matches!(g.state.pending, Pending::ManaLotSpend { .. })
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::ChooseManaLotSpend { .. }
                )
            })
            .unwrap_or_else(|| panic!("unpaid {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("unpaid");
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..90 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled");
}
fn helper(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Hydraulic Helper", Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    g.state.objects[id as usize].summoning_sick = false;
    id
}
fn tower(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Avengers Tower", Zone::Hand);
    checked(g, Action::PlayLand { card: id });
    id
}
fn types(g: &mut Game, id: CardId, types: &'static [&'static str]) {
    let r = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    let roles = g.state.roles.get_or_insert_default();
    roles.creature_types.retain(|e| e.recipient != r);
    roles
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient: r,
            source: r,
            types,
            additional: false,
            timestamp,
        });
}
fn quality() -> mana_lots::Quality {
    mana_lots::Quality {
        restriction: ManaRestrictionView::SubtypeSpellOrActivation("Hero"),
        spell_bonus: ManaSpellBonusView::None,
        retention: ManaRetention::Step,
        copy_source: None,
    }
}

#[test]
fn restricted_helper_defender_sickness_artifact_payment_and_nonartifact_rejection() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Hydraulic Helper", Zone::Hand);
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        assert_eq!(
            (
                g.object_view(id).power,
                g.object_view(id).toughness,
                g.object_view(id).kinds
            ),
            (2, 3, 17)
        );
        reject(&mut g, mana(id, 0, [0, 1, 0, 0, 0, 0]));
        g.state.objects[id as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::Attack { card: id });
        g.state.step = Step::Main1;
        g.state.pending = Pending::Priority;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(&mut g, mana(id, 0, [0, 1, 0, 0, 0, 0]));
        assert_eq!(
            g.mana_lots(p)[0].quality.restriction,
            ManaRestrictionView::ExceptNonArtifactSpells
        );
        let bad = named(&mut g, p, "Divination", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: bad });
        let boots = named(&mut g, p, "Swiftfoot Boots", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: boots });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 0, 1],
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(boots).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
        assert!(g.mana_lots(p).is_empty());
    }
}

#[test]
fn restricted_tower_all_colors_hero_activations_hero_spell_and_ordinary_colorless() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = setup(p);
            let t = tower(&mut g, p);
            let hero = named(&mut g, p, "Reptil, Dinomorpher", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            let mut production = [0; 6];
            production[color] = 1;
            reject(&mut g, mana(t, 1, [0, 0, 0, 0, 0, 1]));
            checked(&mut g, mana(t, 1, production));
            assert_eq!(
                g.mana_lots(p)[0].quality.restriction,
                ManaRestrictionView::SubtypeSpellOrActivation("Hero")
            );
            checked(&mut g, announce(hero, 0));
            pay(&mut g);
            settle(&mut g);
            assert_eq!(
                (g.object_view(hero).power, g.object_view(hero).toughness),
                (3, 5)
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
        let mut g = setup(p);
        let t = tower(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, mana(t, 1, [0, 0, 0, 0, 1, 0]));
        let hero = named(&mut g, p, "Reptil, Dinomorpher", Zone::Hand);
        cast(&mut g, hero, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(hero).zone, Zone::Battlefield);
        let mut g = setup(p);
        let t = tower(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(&mut g, mana(t, 0, [0, 0, 0, 0, 0, 1]));
        let boots = named(&mut g, p, "Swiftfoot Boots", Zone::Hand);
        cast(&mut g, boots, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(boots).zone, Zone::Battlefield);
        assert!(g.mana_lots(p).is_empty());
    }
}

#[test]
fn restricted_tower_nonhero_spell_and_ability_rejected_including_nested_conversion() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = tower(&mut g, p);
        let cave = named(&mut g, p, "Captivating Cave", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, mana(t, 1, [0, 1, 0, 0, 0, 0]));
        reject(
            &mut g,
            Action::ActivateMana {
                card: cave,
                ability: 1,
                payment: [0, 1, 0, 0, 0, 0],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: shock });
        // Exclude Cave's separate ordinary colorless ability from the next probe.
        g.state.objects[cave as usize].tapped = true;
        g.state.players[p.index()].mana[5] = 1;
        let boots = named(&mut g, p, "Swiftfoot Boots", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: boots });
    }
}

#[test]
fn restricted_helper_nested_mana_funds_nonartifact_source_then_nonartifact_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = helper(&mut g, p);
        let cave = named(&mut g, p, "Captivating Cave", Zone::Battlefield);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnounceCast { card: shock });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(&mut g, mana(h, 0, [0, 1, 0, 0, 0, 0]));
        // A fully concrete mana cost uses the direct payment action inside
        // the outer spell's mana window; guided nesting is for external costs.
        checked(
            &mut g,
            Action::ActivateMana {
                card: cave,
                ability: 1,
                counter_payment: Counters::default(),
                payment: [0, 1, 0, 0, 0, 0],
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.object_view(h).tapped && g.object_view(cave).tapped);
    }
}

#[test]
fn restricted_ward_is_allowed_for_helper_but_not_tower_activated_only_mana() {
    for p in [Player::P0, Player::P1] {
        for allowed in [false, true] {
            let mut g = setup(p);
            let producer = if allowed {
                helper(&mut g, p)
            } else {
                tower(&mut g, p)
            };
            let target = named(&mut g, p.other(), "Colorstorm Stallion", Zone::Battlefield);
            // A Hero ward source still has a triggered, not activated, cost.
            types(&mut g, target, &["Hero"]);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
            checked(
                &mut g,
                mana(producer, u16::from(!allowed), [0, 1, 0, 0, 0, 0]),
            );
            let spell = named(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, spell, None, &[Target::Card(target)]);
            until(&mut g, |g| {
                g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::DeclineResolutionMana))
            });
            let payment = Action::PayResolutionMana {
                payment: [0, 1, 0, 0, 0, 0],
            };
            if allowed {
                checked(&mut g, payment);
                pay(&mut g);
            } else {
                reject(&mut g, payment);
                checked(&mut g, Action::DeclineResolutionMana);
            }
            settle(&mut g);
            assert_eq!(g.object_view(target).damage, if allowed { 2 } else { 0 });
        }
    }
}

#[test]
fn restricted_adventure_face_uses_spell_types_instead_of_artifact_front() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = helper(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        checked(&mut g, mana(h, 0, [0, 1, 0, 0, 0, 0]));
        let artifact = named(&mut g, p, "Glamdring, Foe-hammer", Zone::Hand);
        reject(&mut g, Action::AnnounceAdventure { card: artifact });
        checked(&mut g, Action::AnnounceCast { card: artifact });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 0, 1],
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
    }
}

#[test]
fn restricted_lot_allocation_preserves_ordinary_blue_for_later_nonhero_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = tower(&mut g, p);
        let hero = named(&mut g, p, "Reptil, Dinomorpher", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        checked(&mut g, mana(t, 1, [0, 1, 0, 0, 0, 0]));
        checked(&mut g, announce(hero, 0));
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 0, 2],
            },
        );
        assert!(matches!(g.state.pending, Pending::ManaLotSpend { .. }));
        assert!(g.observe(p).mana_lot_spend.is_some());
        assert!(g.observe(p.other()).mana_lot_spend.is_none());
        reject(&mut g, Action::ChooseManaLotSpend { amount: 2 });
        checked(&mut g, Action::ChooseManaLotSpend { amount: 1 });
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 0, 0]);
        assert!(g.mana_lots(p).is_empty());
        let cave = named(&mut g, p, "Captivating Cave", Zone::Battlefield);
        checked(
            &mut g,
            Action::ActivateMana {
                card: cave,
                ability: 1,
                payment: [0, 1, 0, 0, 0, 0],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}

#[test]
fn restricted_lot_rules_survive_producer_control_and_zone_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = tower(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, mana(t, 1, [0, 0, 0, 0, 1, 0]));
        let object = g.object_ref(t);
        g.change_controller(object, p.other());
        g.move_object(t, Zone::Graveyard);
        assert_eq!(
            serde_json::to_string(&g.observe(p).mana_lots).unwrap(),
            serde_json::to_string(&g.observe(p.other()).mana_lots).unwrap()
        );
        let hero = named(&mut g, p, "Reptil, Dinomorpher", Zone::Hand);
        cast(&mut g, hero, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(hero).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn restricted_speculative_current_source_types_agree_with_actual_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = tower(&mut g, p);
        let hero = named(&mut g, p, "Reptil, Dinomorpher", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(&mut g, mana(t, 1, [0, 1, 0, 0, 0, 0]));
        let reference = g.object_ref(hero);
        let definition = g.state.objects[hero as usize].card;
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 3,
            reserve: None,
            source: Some((reference, 0, Counters::default(), definition, None)),
        };
        assert!(g.can_fund(goal.clone()));
        let before = g.snapshot();
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        world
            .entry_projection
            .as_mut()
            .unwrap()
            .roles
            .creature_types
            .push(subtypes::CreatureTypesEffectView {
                recipient: reference,
                source: reference,
                types: &["Citizen"],
                additional: false,
                timestamp: 100,
            });
        assert!(
            !world.reachable(&goal, &mut HashSet::new()),
            "virtual Hero loss invalidates Tower funding"
        );
        assert_eq!(g.snapshot(), before);
        types(&mut g, hero, &["Citizen"]);
        reject(&mut g, announce(hero, 0));
        types(&mut g, hero, &["Hero"]);
        checked(&mut g, announce(hero, 0));
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(hero).power, 3);
    }
}

#[test]
fn restricted_source_departure_lki_does_not_rebind_and_hero_source_need_not_be_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = tower(&mut g, p);
        let source = named(&mut g, p, "Reckless Lackey", Zone::Battlefield);
        types(&mut g, source, &["Hero"]);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(&mut g, mana(t, 1, [0, 0, 0, 1, 0, 0]));
        checked(&mut g, announce(source, 0));
        pay(&mut g);
        let item = g.state.stack.last().unwrap().clone();
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert!(quality().eligible(g.purpose_for_item(&item)));
        let mut triggered = item.clone();
        triggered.kind = StackKind::TriggeredAbility { ability: 0 };
        assert!(!quality().eligible(g.purpose_for_item(&triggered)));
        g.move_object(source, Zone::Battlefield);
        assert!(!g.has_subtype(source, "Hero"));
        assert!(quality().eligible(g.purpose_for_item(&item)));
        settle(&mut g);
    }
    let source = mana_lots::Source {
        card: Card::from_name("Reptil, Dinomorpher").unwrap(),
        kinds: 16,
        subtypes: GrantedSubtypes::default(),
        foreign_stash: false,
        spend_any_mana: false,
        equip: false,
    };
    assert!(quality().eligible(mana_lots::Purpose::Activation(source)));
    assert!(!quality().eligible(mana_lots::Purpose::Other));
}

#[test]
fn restricted_tower_private_top_three_optional_revealed_hero_and_arbitrary_bottom() {
    for p in [Player::P0, Player::P1] {
        for take in [false, true] {
            let mut g = setup(p);
            let t = tower(&mut g, p);
            let one = named(&mut g, p, "Forest", Zone::Library);
            let hero = named(&mut g, p, "Reptil, Dinomorpher", Zone::Library);
            let two = named(&mut g, p, "Hill Giant", Zone::Library);
            let hand = g.hand_count(p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
            checked(&mut g, announce(t, 2));
            reject(&mut g, Action::PayCosts { payment: [0; 6] });
            pay(&mut g);
            assert!(g.object_view(t).tapped);
            g.move_object(t, Zone::Graveyard);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
            });
            assert!(g.observe(p).library_selection.is_some());
            assert!(g.observe(p.other()).library_selection.is_none());
            reject(&mut g, Action::SelectLibrary { card: one });
            if take {
                checked(&mut g, Action::SelectLibrary { card: hero });
            }
            checked(&mut g, Action::FinishLibrarySelection);
            let order = if take {
                vec![one, two]
            } else {
                vec![hero, one, two]
            };
            for &card in &order {
                if g.legal_actions()
                    .contains(&Action::OrderLibraryBottom { card })
                {
                    checked(&mut g, Action::OrderLibraryBottom { card });
                }
            }
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand + usize::from(take));
            let bottom: Vec<_> = order.into_iter().rev().collect();
            assert_eq!(
                &g.state.players[p.index()].library[..bottom.len()],
                bottom.as_slice()
            );
            assert_eq!(
                g.object_view(hero).zone,
                if take { Zone::Hand } else { Zone::Library }
            );
            assert_eq!(
                g.observe(p.other()).objects.iter().any(|o| o.id == hero),
                take
            );
            assert!(
                !g.observe(p.other())
                    .objects
                    .iter()
                    .any(|o| o.id == one || o.id == two)
            );
        }
    }
}

#[test]
fn restricted_guided_nested_mana_keeps_outer_hero_purpose_and_private_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = tower(&mut g, p);
        let hero = named(&mut g, p, "Reptil, Dinomorpher", Zone::Battlefield);
        let rubble = named(&mut g, p, "Rubble Rouser", Zone::Battlefield);
        g.state.objects[rubble as usize].summoning_sick = false;
        let fuel = named(&mut g, p, "Forest", Zone::Graveyard);
        let foreign = named(&mut g, p.other(), "Forest", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(&mut g, mana(t, 1, [0, 1, 0, 0, 0, 0]));
        checked(&mut g, announce(hero, 0));
        checked(
            &mut g,
            Action::AnnounceMana {
                card: rubble,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        assert_eq!(g.observe(p).suspended_mana_announcements.len(), 1);
        assert!(g.observe(p).suspended_mana_announcements[0].cost.is_some());
        assert!(
            g.observe(p.other()).suspended_mana_announcements[0]
                .cost
                .is_none()
        );
        let bad = g.object_ref(foreign);
        reject(&mut g, Action::ChooseAdditionalCost { object: bad });
        let object = g.object_ref(fuel);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(fuel).zone, Zone::Exile);
        assert_eq!(g.object_view(foreign).zone, Zone::Graveyard);
        assert_eq!(g.object_view(hero).power, 3);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.other().index()].life, 19);
        assert!(g.state.mana_announcement_stack.is_none());
    }
}

#[test]
fn restricted_virtual_departure_freezes_last_live_source_types_and_keys() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = tower(&mut g, p);
        let dragon = named(&mut g, p, "Shivan Dragon", Zone::Battlefield);
        types(&mut g, dragon, &["Hero"]);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, mana(t, 1, [0, 0, 0, 1, 0, 0]));
        checked(&mut g, announce(dragon, 0));
        let goal = g.funding_goal(g.state.announcing.as_ref().unwrap());
        let reference = g.object_ref(dragon);
        let before = g.snapshot();
        let mut keys = Vec::new();
        for hero_at_departure in [false, true] {
            let mut world = FundingState::from_game(&g, &goal);
            world.prepare_entry_projection();
            // A virtual type change after announcement must be captured at
            // departure, rather than recycling announcement-time facts.
            let roles = &mut world.entry_projection.as_mut().unwrap().roles;
            roles
                .creature_types
                .retain(|row| row.recipient != reference);
            roles
                .creature_types
                .push(subtypes::CreatureTypesEffectView {
                    recipient: reference,
                    source: reference,
                    types: if hero_at_departure {
                        &["Hero"]
                    } else {
                        &["Citizen"]
                    },
                    additional: false,
                    timestamp: 100,
                });
            assert_eq!(
                quality().eligible(world.payment_purpose(&goal)),
                hero_at_departure
            );
            let mut visits = 0;
            world.cost_departures(&[reference], Zone::Graveyard, &mut |departed| {
                visits += 1;
                assert_eq!(departed.object(dragon).zone, Zone::Graveyard);
                assert_ne!(departed.reference(dragon), reference);
                assert!(
                    !departed
                        .subtypes(dragon)
                        .has(departed.object(dragon).card, "Hero")
                );
                assert_eq!(
                    quality().eligible(departed.payment_purpose(&goal)),
                    hero_at_departure
                );
                assert_eq!(
                    departed.reachable(&goal, &mut HashSet::new()),
                    hero_at_departure
                );
                keys.push(departed.key(&goal));
                false
            });
            assert_eq!(visits, 1);
        }
        // Same post-departure object/pool facts can have different saved types.
        assert!(keys[0] != keys[1]);
        assert_eq!(g.snapshot(), before);
    }
}

#[test]
fn restricted_funding_after_real_source_departure_and_blink_uses_announcement_lki() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let t = tower(&mut g, p);
            let dragon = named(&mut g, p, "Shivan Dragon", Zone::Battlefield);
            types(&mut g, dragon, &["Hero"]);
            g.state.players[p.index()].mana = [0; 6];
            checked(&mut g, mana(t, 1, [0, 0, 0, 1, 0, 0]));
            checked(&mut g, announce(dragon, 0));
            let goal = g.funding_goal(g.state.announcing.as_ref().unwrap());
            let old = g.object_ref(dragon);
            g.move_object(dragon, Zone::Graveyard);
            if blink {
                g.move_object(dragon, Zone::Battlefield);
            }
            assert_ne!(g.object_ref(dragon), old);
            assert!(!g.has_subtype(dragon, "Hero"));
            let snapshot = g.snapshot();
            let mut branch = {
                let parent = g.clone();
                parent.clone()
            };
            assert!(g.can_fund(goal.clone()));
            assert!(branch.can_fund(goal.clone()));
            let world = FundingState::from_game(&g, &goal);
            assert!(quality().eligible(world.payment_purpose(&goal)));
            assert!(world.reachable(&goal, &mut HashSet::new()));
            assert_eq!(g.snapshot(), snapshot);
            pay(&mut g);
            pay(&mut branch);
            settle(&mut g);
            settle(&mut branch);
            assert_eq!(g.snapshot(), branch.snapshot());
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(g.mana_lots(p).is_empty());
            if blink {
                assert_eq!(g.object_view(dragon).power, 5);
            }
        }
    }
}
