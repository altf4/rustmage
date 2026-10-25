//! Whole Undercover Skrull: conditional source layers, real mana and current controller.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn mana(card: CardId, color: usize) -> Action {
    let mut production = [0; 6];
    production[color] = 1;
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}
fn corpses(g: &mut Game, p: Player) -> [CardId; 2] {
    [
        named(g, p, "Bear Cub", Zone::Graveyard),
        named(g, p, "Hill Giant", Zone::Graveyard),
    ]
}
fn assert_form(g: &Game, id: CardId, high: bool) {
    let v = g.object_view(id);
    assert_eq!((v.power, v.toughness), if high { (3, 3) } else { (1, 1) });
    for typ in ["Skrull", "Shapeshifter", "Villain"] {
        assert!(v.has_subtype(typ));
    }
    for typ in ["Elf", "Dragon", "Hero", "Time Lord"] {
        assert_eq!(v.has_subtype(typ), high, "{typ}");
    }
    assert!(!v.has_subtype("Equipment") && !v.has_subtype("Forest"));
}
fn rubble(g: &mut Game, p: Player, fuel: CardId) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let card = named(g, p, "Rubble Rouser", Zone::Battlefield);
    checked(
        g,
        Action::AnnounceMana {
            card,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 1, 0, 0],
        },
    );
    let object = g.object_ref(fuel);
    checked(g, Action::ChooseAdditionalCost { object });
    for _ in 0..10 {
        if g.state.announcing.is_none() {
            break;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    settle(g);
}
#[test]
fn msh_skrull_paid_body_all_five_mana_colors_sickness_and_atomic_invalid_production() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
            let id = paid(&mut g, p, "Undercover Skrull", &[]);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_form(&g, id, false);
            assert_eq!(g.object_view(id).colors, 16);
            reject(&mut g, mana(id, color));
            next_turn(&mut g);
            next_turn(&mut g);
            reject(&mut g, mana(id, 5));
            reject(
                &mut g,
                Action::ActivateMana {
                    card: id,
                    ability: 0,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production: [1, 1, 0, 0, 0, 0],
                },
            );
            checked(&mut g, mana(id, color));
            let mut expected = [0; 6];
            expected[color] = 1;
            assert_eq!(g.state.players[p.index()].mana, expected);
            assert!(g.object_view(id).tapped && g.state.stack.is_empty());
            reject(&mut g, mana(id, color));
        }
    }
}
#[test]
fn msh_skrull_threshold_only_current_controller_creature_cards_not_tokens_or_other_zones() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=2 {
            let mut g = setup(p);
            corpses(&mut g, p.other());
            named(&mut g, p, "Shock", Zone::Graveyard);
            named(
                &mut g,
                p,
                "@token/cf0227c0984ee9114e6c311f",
                Zone::Graveyard,
            );
            named(&mut g, p, "Bear Cub", Zone::Hand);
            named(&mut g, p, "Hill Giant", Zone::Exile);
            for _ in 0..count {
                named(&mut g, p, "Bear Cub", Zone::Graveyard);
            }
            let id = paid(&mut g, p, "Undercover Skrull", &[]);
            assert_form(&g, id, count == 2);
            let secret = named(&mut g, p, "Shock", Zone::Hand);
            assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == secret));
            for zone in [Zone::Hand, Zone::Graveyard, Zone::Exile] {
                g.move_object(id, zone);
                assert_form(&g, id, false);
            }
        }
    }
}
#[test]
fn msh_skrull_actual_paid_death_and_graveyard_mana_exile_toggle_both_parts() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Hill Giant", Zone::Graveyard);
        let id = paid(&mut g, p, "Undercover Skrull", &[]);
        assert_form(&g, id, false);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        paid(&mut g, p, "Shock", &[Target::Card(bear)]);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_form(&g, id, true);
        rubble(&mut g, p, bear);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        assert_form(&g, id, false);
    }
}
#[test]
fn msh_skrull_type_effect_continues_bonus_through_later_printed_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fuel = corpses(&mut g, p)[0];
        let id = paid(&mut g, p, "Undercover Skrull", &[]);
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(id)]);
        let v = g.object_view(id);
        assert_eq!((v.power, v.toughness), (3, 3));
        assert!(v.has_subtype("Citizen") && !v.has_subtype("Elf") && !v.has_subtype("Skrull"));
        assert!(!g.printed_abilities_available(id));
        reject(&mut g, mana(id, 1));
        rubble(&mut g, p, fuel);
        assert_eq!((g.stats(id).power, g.stats(id).toughness), (1, 1));
        named(&mut g, p, "Bear Cub", Zone::Graveyard);
        assert_eq!((g.stats(id).power, g.stats(id).toughness), (3, 3));
        g.move_object(aura, Zone::Graveyard);
        assert_form(&g, id, true);
    }
}
#[test]
fn msh_skrull_noncreature_land_form_has_no_creature_types_then_restores() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        corpses(&mut g, p);
        let id = paid(&mut g, p, "Undercover Skrull", &[]);
        let aura = paid(&mut g, p, "Imprisoned in the Moon", &[Target::Card(id)]);
        let v = g.object_view(id);
        assert_eq!(v.kinds & 3, 2);
        for typ in ["Elf", "Dragon", "Skrull", "Villain"] {
            assert!(!v.has_subtype(typ));
        }
        assert!(!g.printed_abilities_available(id));
        g.state.players[p.index()].mana = [0; 6];
        let grantor = g.object_ref(aura);
        checked(
            &mut g,
            Action::TapGrantedMana {
                card: id,
                grantor,
                production: [0, 0, 0, 0, 0, 1],
            },
        );
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
        g.move_object(aura, Zone::Graveyard);
        assert_form(&g, id, true);
    }
}
#[test]
fn msh_skrull_source_theft_uses_new_controller_graveyard_and_cleanup_restores_relation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        corpses(&mut g, p);
        let id = paid(&mut g, p, "Undercover Skrull", &[]);
        assert_form(&g, id, true);
        g.state.active = p.other();
        g.state.priority = p.other();
        paid(
            &mut g,
            p.other(),
            "Involuntary Employment",
            &[Target::Card(id)],
        );
        assert_eq!(g.object_view(id).controller, p.other());
        assert_form(&g, id, false);
        g.state.players[p.other().index()].mana = [0; 6];
        checked(&mut g, mana(id, 2));
        assert_eq!(g.state.players[p.other().index()].mana, [0, 0, 1, 0, 0, 0]);
        corpses(&mut g, p.other());
        assert_form(&g, id, true);
        g.finish_cleanup();
        assert_eq!(g.object_view(id).controller, p);
        assert_form(&g, id, true);
    }
}
#[test]
fn msh_skrull_type_setter_timestamp_order_and_restarted_source_effect() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        corpses(&mut g, p);
        let id = paid(&mut g, p, "Undercover Skrull", &[]);
        let packet = g.activation_item(p, id, 0);
        // Existing production type replacement is a later layer-four effect.
        g.set_creature_types(&packet, Subject::Source, &["Hero"]);
        assert_eq!(g.stats(id).power, 3);
        assert!(g.has_subtype(id, "Hero"));
        assert!(!g.has_subtype(id, "Elf"));
        // A copy effect restarting this exact source uses this production clock.
        g.retimestamp_live_static_source(g.object_ref(id));
        assert!(g.has_subtype(id, "Elf") && g.has_subtype(id, "Dragon"));
        assert_eq!(g.stats(id).power, 3);
    }
}
#[test]
fn msh_skrull_other_elf_modifier_and_real_subtype_scaled_mana_follow_live_threshold() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fuel = corpses(&mut g, p)[0];
        let lord = paid(&mut g, p, "Elvish Archdruid", &[]);
        let id = paid(&mut g, p, "Undercover Skrull", &[]);
        assert_eq!((g.stats(id).power, g.stats(id).toughness), (4, 4));
        g.state.objects[lord as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::ActivateMana {
                card: lord,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 2, 0],
            },
        );
        rubble(&mut g, p, fuel);
        assert_form(&g, id, false);
        g.state.objects[lord as usize].tapped = false;
        checked(
            &mut g,
            Action::ActivateMana {
                card: lord,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            },
        );
    }
}
#[test]
fn msh_skrull_copies_recompute_for_their_controller_and_zone_changes_clear_stale_types() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        corpses(&mut g, p);
        let id = paid(&mut g, p, "Undercover Skrull", &[]);
        let def = g.object_view(id).card;
        let copy = g.create_copy_groups_inner(
            p.other(),
            &[TokenGroupView {
                token: def,
                amount: 1,
                tapped: false,
            }],
            &[Some(CopyRecipe::new(def))],
            None,
        )[0]
        .card;
        g.advance_entries();
        assert_form(&g, id, true);
        assert_form(&g, copy, false);
        corpses(&mut g, p.other());
        assert_form(&g, copy, true);
        let old = g.object_ref(id);
        g.move_object(id, Zone::Hand);
        assert_form(&g, id, false);
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        assert_ne!(old, g.object_ref(id));
        assert_form(&g, id, true);
    }
}
