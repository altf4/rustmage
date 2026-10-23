use super::ability_timeline_tests::{checked, named, settle, setup};
use super::tmt_departure_tests::{candidate, cast_candidate};
use super::*;
#[test]
fn tmt_final_stomp_uncast_kicker_needs_real_controlled_artifact_or_creature() {
    let p = Player::P0;
    let mut g = setup(p);
    let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let spell = candidate(&mut g, p, "Stomped by the Foot", Zone::Hand);
    start(&mut g, spell);
    assert!(
        !g.legal_actions()
            .contains(&Action::ChooseKicker { pay: true })
    );
    checked(&mut g, Action::ChooseKicker { pay: false });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    for _ in 0..4 {
        if g.state.announcing.is_none() {
            break;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(&mut g, a);
    }
    settle(&mut g);
    assert_eq!(g.stats(enemy).toughness, 1);
}
#[test]
fn tmt_final_brilliance_returns_nontoken_artifact_as_permanent_flying_robot() {
    let p = Player::P0;
    let mut g = setup(p);
    let artifact = named(&mut g, p, "Hedron Archive", Zone::Graveyard);
    let spell = candidate(&mut g, p, "Brilliance Unleashed", Zone::Hand);
    start(&mut g, spell);
    checked(&mut g, Action::ChooseMode { mode: 1 });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    for _ in 0..4 {
        if g.state.announcing.is_none() {
            break;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(&mut g, a);
    }
    settle(&mut g);
    assert!(g.is_creature(artifact));
    assert!(g.has_subtype(artifact, "Robot"));
    assert!(g.keywords(artifact).contains(Keywords::FLYING));
    assert_eq!(
        (g.stats(artifact).power, g.stats(artifact).toughness),
        (3, 3)
    );
}
#[test]
fn tmt_final_aura_paid_casts_attach_and_retro_preserves_colors_and_types() {
    for (name, retro) in [("Retro-Mutation", true), ("Uneasy Alliance", false)] {
        let p = Player::P0;
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let aura = candidate(&mut g, p, name, Zone::Hand);
        cast_candidate(&mut g, aura, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(
            g.state.objects[aura as usize].attached_to,
            Some(g.object_ref(bear))
        );
        assert!(g.combat_restricted(bear, true));
        if retro {
            assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (0, 1));
            assert!(g.has_subtype(bear, "Turtle"));
            assert_eq!(g.current_kinds(bear), 1);
        } else {
            assert!(g.combat_restricted(bear, false));
        }
    }
}

fn start(g: &mut Game, id: CardId) {
    if g.state.objects[id as usize].card.is_supported() {
        checked(g, Action::AnnounceCast { card: id });
    } else {
        let item = g.cast_item(g.actor().unwrap(), id, [0; 6]);
        g.open_payment_announcement(item, Counters::default(), None);
    }
}
fn finish(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap_or_else(|| panic!("payment {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("unfinished payment");
}
fn brilliance(g: &mut Game, mode: u16, targets: &[CardId]) -> CardId {
    let p = g.actor().unwrap();
    let id = candidate(g, p, "Brilliance Unleashed", Zone::Hand);
    start(g, id);
    checked(g, Action::ChooseMode { mode });
    for &t in targets {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(t),
            },
        );
    }
    finish(g);
    id
}
fn stomp(g: &mut Game, target: CardId, donor: Option<CardId>) -> CardId {
    let p = g.actor().unwrap();
    let id = candidate(g, p, "Stomped by the Foot", Zone::Hand);
    start(g, id);
    checked(
        g,
        Action::ChooseKicker {
            pay: donor.is_some(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if let Some(id) = donor {
        let object = g.object_ref(id);
        checked(g, Action::ChooseAdditionalCost { object });
    }
    finish(g);
    id
}
fn aura(g: &mut Game, name: &str, target: CardId) -> CardId {
    let p = g.actor().unwrap();
    let id = candidate(g, p, name, Zone::Hand);
    cast_candidate(g, id, &[Target::Card(target)]);
    settle(g);
    id
}
fn activate(g: &mut Game, id: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    finish(g);
}
#[test]
fn tmt_final_stomp_kicker_sacrifices_artifact_creature_or_token_and_pays_same_mana() {
    for p in [Player::P0, Player::P1] {
        for donor in [
            "Hedron Archive",
            "Grizzly Bears",
            "@token/283fd83a9aec83a51b736926",
        ] {
            let mut g = setup(p);
            let victim = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
            let donor = named(&mut g, p, donor, Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
            stomp(&mut g, victim, Some(donor));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(!g.on_battlefield(donor));
            assert_eq!(
                g.state.stack.last().unwrap().cast.unwrap().kicker,
                Some(true)
            );
            settle(&mut g);
            assert_eq!(g.stats(victim).toughness, 5);
        }
    }
}
#[test]
fn tmt_final_stomp_kicker_reserves_cost_object_and_rejects_wrong_costs_atomically() {
    let p = Player::P0;
    let mut g = setup(p);
    let donor = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let land = named(&mut g, p, "Forest", Zone::Battlefield);
    let enchantment = named(&mut g, p, "Anthem of Champions", Zone::Battlefield);
    let spell = candidate(&mut g, p, "Stomped by the Foot", Zone::Hand);
    start(&mut g, spell);
    checked(&mut g, Action::ChooseKicker { pay: true });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    for id in [land, enchantment, victim] {
        let object = g.object_ref(id);
        super::tmt_departure_tests::reject(&mut g, Action::ChooseAdditionalCost { object });
    }
    let object = g.object_ref(donor);
    checked(&mut g, Action::ChooseAdditionalCost { object });
    g.move_object(donor, Zone::Hand);
    g.move_object(donor, Zone::Battlefield);
    super::tmt_departure_tests::reject(
        &mut g,
        Action::PayCosts {
            payment: [0, 0, 1, 0, 0, 1],
        },
    );
}
#[test]
fn tmt_final_stomp_sacrificed_target_fizzles_and_expiring_minus_stats_preserve_counters() {
    let p = Player::P0;
    let mut g = setup(p);
    let donor = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    stomp(&mut g, donor, Some(donor));
    settle(&mut g);
    assert_eq!(g.state.objects[donor as usize].zone, Zone::Graveyard);
    let victim = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
    g.set_counters(victim, Counters::of(CounterKind::PlusOne, 1));
    stomp(&mut g, victim, None);
    settle(&mut g);
    assert_eq!(g.stats(victim).toughness, 9);
    super::ability_timeline_tests::next_turn(&mut g);
    assert_eq!(g.stats(victim).toughness, 11);
    assert_eq!(g.counter_count(victim, CounterKind::PlusOne), 1);
}
#[test]
fn tmt_final_brilliance_all_modes_and_partial_illegal_targets() {
    for p in [Player::P0, Player::P1] {
        for invalid in [0, 1, 2] {
            let mut g = setup(p);
            let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let artifact = named(&mut g, p, "Hedron Archive", Zone::Graveyard);
            brilliance(&mut g, 2, &[victim, artifact]);
            if invalid == 1 {
                g.move_object(victim, Zone::Hand);
            }
            if invalid == 2 {
                g.move_object(artifact, Zone::Exile);
            }
            settle(&mut g);
            if invalid != 1 {
                assert_eq!(g.state.objects[victim as usize].zone, Zone::Graveyard);
            }
            if invalid != 2 {
                assert!(g.on_battlefield(artifact));
                assert_eq!(g.stats(artifact).power, 3);
            }
        }
    }
    let p = Player::P0;
    let mut g = setup(p);
    let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    brilliance(&mut g, 0, &[victim]);
    settle(&mut g);
    assert_eq!(g.state.objects[victim as usize].zone, Zone::Graveyard);
}
#[test]
fn tmt_final_brilliance_artifact_creature_unchanged_and_animation_not_copiable() {
    let p = Player::P0;
    let mut g = setup(p);
    let mascot = named(&mut g, p, "Three Tree Mascot", Zone::Graveyard);
    brilliance(&mut g, 1, &[mascot]);
    settle(&mut g);
    assert_eq!((g.stats(mascot).power, g.stats(mascot).toughness), (2, 1));
    assert!(!g.keywords(mascot).contains(Keywords::FLYING));
    let archive = named(&mut g, p, "Hedron Archive", Zone::Graveyard);
    brilliance(&mut g, 1, &[archive]);
    settle(&mut g);
    let image = named(&mut g, p, "Omni-Changeling", Zone::Hand);
    super::ability_timeline_tests::cast(&mut g, image, None, &[]);
    for _ in 0..20 {
        if g.legal_actions().contains(&Action::ChooseEntryCopy {
            object: Some(g.object_ref(archive)),
        }) {
            break;
        }
        checked(&mut g, Action::Pass);
    }
    let reference = g.object_ref(archive);
    checked(
        &mut g,
        Action::ChooseEntryCopy {
            object: Some(reference),
        },
    );
    settle(&mut g);
    assert!(!g.is_creature(image));
    assert_eq!(
        g.state.objects[image as usize].card,
        Card::from_name("Hedron Archive").unwrap()
    );
}
#[test]
fn tmt_final_brilliance_stale_target_does_not_animate_reentered_graveyard_incarnation() {
    let p = Player::P0;
    let mut g = setup(p);
    let artifact = named(&mut g, p, "Hedron Archive", Zone::Graveyard);
    brilliance(&mut g, 1, &[artifact]);
    g.move_object(artifact, Zone::Hand);
    g.move_object(artifact, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(g.state.objects[artifact as usize].zone, Zone::Graveyard);
    assert!(
        g.observe(p)
            .animations
            .iter()
            .all(|r| r.object.card != artifact)
    );
}
#[test]
fn tmt_final_retro_flash_artifact_type_name_colors_and_later_keyword_grants() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mascot = named(&mut g, p.other(), "Three Tree Mascot", Zone::Battlefield);
        let old_name = g.object_name(mascot);
        let old_colors = super::ability_world::CharacteristicWorld::colors(&g, mascot);
        g.set_counters(mascot, Counters::of(CounterKind::PlusOne, 2));
        g.state.active = p.other();
        let e = aura(&mut g, "Retro-Mutation", mascot);
        assert_eq!(g.current_kinds(mascot), 17);
        assert_eq!(g.object_name(mascot), old_name);
        assert_eq!(
            super::ability_world::CharacteristicWorld::colors(&g, mascot),
            old_colors
        );
        assert_eq!((g.stats(mascot).power, g.stats(mascot).toughness), (2, 3));
        assert!(!g.has_subtype(mascot, "Shapeshifter"));
        let source = g.activation_item(p, mascot, 0);
        g.grant_keyword_effects(&[mascot], source.object, Keywords::FLYING);
        assert!(g.keywords(mascot).contains(Keywords::FLYING));
        assert!(g.combat_restricted(mascot, true));
        g.move_object(e, Zone::Hand);
        assert_eq!(g.stats(mascot).power, 4);
        assert!(g.has_subtype(mascot, "Shapeshifter"));
    }
}
#[test]
fn tmt_final_uneasy_sorcery_sacrifice_lki_exile_and_ninja_creation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let e = aura(&mut g, "Uneasy Alliance", victim);
        g.state.step = Step::End;
        super::tmt_departure_tests::reject(
            &mut g,
            Action::AnnounceActivate {
                card: e,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
        activate(&mut g, e);
        assert_eq!(g.state.objects[e as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.on_battlefield(victim));
        settle(&mut g);
        assert_eq!(g.state.objects[victim as usize].zone, Zone::Exile);
        assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
            && o.controller == p
            && o.card == Card::from_name("@token/ninja_1_1_black").unwrap()));
    }
}
#[test]
fn tmt_final_uneasy_departed_recipient_still_creates_ninja_and_old_incarnation_stays_safe() {
    let p = Player::P0;
    let mut g = setup(p);
    let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let e = aura(&mut g, "Uneasy Alliance", victim);
    activate(&mut g, e);
    g.move_object(victim, Zone::Hand);
    g.move_object(victim, Zone::Battlefield);
    settle(&mut g);
    assert!(g.on_battlefield(victim));
    assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
        && o.controller == p
        && o.card == Card::from_name("@token/ninja_1_1_black").unwrap()));
}
#[test]
fn tmt_final_aura_restrictions_survive_later_recipient_ability_removal_and_fall_off_departure() {
    for name in ["Retro-Mutation", "Uneasy Alliance"] {
        let p = Player::P0;
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        aura(&mut g, name, bear);
        let witness = named(&mut g, p, "Witness Protection", Zone::Hand);
        super::ability_timeline_tests::cast(&mut g, witness, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert!(g.combat_restricted(bear, true));
        if name == "Uneasy Alliance" {
            assert!(g.combat_restricted(bear, false));
        }
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        super::tmt_departure_tests::reject(&mut g, Action::Attack { card: bear });
    }
}

#[test]
fn tmt_final_stomp_kicker_cannot_double_spend_a_sacrificed_mana_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        named(
            &mut g,
            p,
            "@token/b60f6cd3b44b22bd289fe4e2",
            Zone::Battlefield,
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        let spell = candidate(&mut g, p, "Stomped by the Foot", Zone::Hand);
        start(&mut g, spell);
        assert!(
            g.legal_actions()
                .contains(&Action::ChooseKicker { pay: false })
        );
        super::tmt_departure_tests::reject(&mut g, Action::ChooseKicker { pay: true });
    }
}
#[test]
fn tmt_final_stomp_kicker_can_tap_then_sacrifice_the_same_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.state.objects[elf as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        let spell = candidate(&mut g, p, "Stomped by the Foot", Zone::Hand);
        start(&mut g, spell);
        checked(&mut g, Action::ChooseKicker { pay: true });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        let object = g.object_ref(elf);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        checked(&mut g, Action::TapMana { card: elf });
        finish(&mut g);
        assert_eq!(g.state.objects[elf as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.stats(victim).toughness, 5);
    }
}
