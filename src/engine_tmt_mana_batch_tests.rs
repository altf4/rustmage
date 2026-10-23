//! Five complete TMT mana consumers: paid actions and owned branch contracts.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::*;
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn mature(g: &mut Game) {
    next_turn(g);
    next_turn(g);
    while g.state.step != Step::Main1 {
        checked(g, Action::Pass);
    }
    for s in &mut g.state.players {
        s.mana = [30; 6];
    }
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn mana(g: &mut Game, id: CardId, ability: u16, production: Mana, counters: Counters) {
    checked(
        g,
        Action::ActivateMana {
            card: id,
            ability,
            payment: [0; 6],
            counter_payment: counters,
            production,
        },
    );
}
fn activate(g: &mut Game, id: CardId, ability: u16, target: Target, counters: Counters) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: counters,
        },
    );
    checked(g, Action::ChooseTarget { target });
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished activation");
}
#[test]
fn tmt_mana_mona_paid_reach_sickness_and_one_color_current_power() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mona = paid(&mut g, p, "Mona Lisa, Science Geek");
        assert!(g.keywords(mona).contains(Keywords::REACH));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::ActivateMana{card,..} if *card==mona))
        );
        mature(&mut g);
        g.add_counter(mona, CounterKind::PlusOne, 3);
        settle(&mut g);
        g.state.players[p.index()].mana = [0; 6];
        reject(
            &mut g,
            Action::ActivateMana {
                card: mona,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [2, 2, 0, 0, 0, 0],
            },
        );
        mana(&mut g, mona, 0, [0, 0, 0, 0, 4, 0], Counters::default());
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 4, 0]);
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn tmt_mana_mona_nonpositive_power_adds_zero_and_funds_exact_one_color() {
    let p = Player::P0;
    let mut g = setup(p);
    let mona = paid(&mut g, p, "Mona Lisa, Science Geek");
    mature(&mut g);
    g.add_counter(mona, CounterKind::MinusOne, 2);
    settle(&mut g);
    assert_eq!(g.stats(mona).power, -1);
    g.state.players[p.index()].mana = [0; 6];
    mana(&mut g, mona, 0, [0; 6], Counters::default());
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
#[test]
fn tmt_mana_punks_restriction_allows_artifact_spell_and_any_activation_but_not_other_spells() {
    for activation in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let punk = paid(&mut g, p, "Purple Dragon Punks");
        mature(&mut g);
        let archive = if activation {
            named(&mut g, p, "Hedron Archive", Zone::Battlefield)
        } else {
            0
        };
        g.state.players[p.index()].mana = [0; 6];
        mana(&mut g, punk, 0, [0, 0, 0, 1, 0, 0], Counters::default());
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: shock })
        );
        g.state.players[p.index()].mana[5] = if activation { 1 } else { 2 };
        if activation {
            // Punks pays an ability on an artifact whose printed colors do not include red.
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: archive,
                    ability: 1,
                    counter_payment: Counters::default(),
                },
            );
            for _ in 0..6 {
                if g.state.announcing.is_none() {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                    .unwrap();
                checked(&mut g, a);
            }
            settle(&mut g);
            assert_eq!(g.state.objects[archive as usize].zone, Zone::Graveyard);
        } else {
            let maker = named(&mut g, p, "Weather Maker", Zone::Hand);
            cast(&mut g, maker, None, &[]);
            settle(&mut g);
            assert!(g.on_battlefield(maker));
        }
    }
}
#[test]
fn tmt_mana_turtle_lair_land_play_and_restricted_ninja_turtle_spell_union() {
    for (name, allowed) in [
        ("Foot Mystic", true),
        ("Aegis Turtle", true),
        ("Grizzly Bears", false),
    ] {
        let p = Player::P0;
        let mut g = setup(p);
        let lair = named(&mut g, p, "Turtle Lair", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: lair });
        g.state.players[p.index()].mana = [0; 6];
        mana(
            &mut g,
            lair,
            1,
            if name == "Foot Mystic" {
                [0, 0, 1, 0, 0, 0]
            } else if name == "Grizzly Bears" {
                [0, 0, 0, 0, 1, 0]
            } else {
                [0, 1, 0, 0, 0, 0]
            },
            Counters::default(),
        );
        let spell = named(&mut g, p, name, Zone::Hand);
        // Leave only the printed generic remainder; the restricted lot must pay a colored symbol.
        g.state.players[p.index()].mana[5] = if name == "Foot Mystic" { 3 } else { 0 };

        assert_eq!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell }),
            allowed,
            "{name}"
        );
        if allowed {
            cast(&mut g, spell, None, &[]);
            settle(&mut g);
            assert!(g.on_battlefield(spell));
        }
    }
}
#[test]
fn tmt_mana_turtle_lair_unblockable_any_controller_ninja_or_turtle_expires() {
    for name in ["Foot Ninjas", "Aegis Turtle"] {
        let p = Player::P0;
        let mut g = setup(p);
        let lair = named(&mut g, p, "Turtle Lair", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: lair });
        let target = named(&mut g, p.other(), name, Zone::Battlefield);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: lair,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        for _ in 0..6 {
            if g.state.announcing.is_none() {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, a);
        }
        settle(&mut g);
        assert!(g.keywords(target).contains(Keywords::CANT_BE_BLOCKED));
        assert!(g.state.objects[lair as usize].tapped);
        next_turn(&mut g);
        assert!(!g.keywords(target).contains(Keywords::CANT_BE_BLOCKED));
    }
}
#[test]
fn tmt_mana_groundchuck_paid_trample_owned_basic_and_nonbasic_taps_only() {
    for landname in ["Forest", "Turtle Lair"] {
        let p = Player::P0;
        let mut g = setup(p);
        let ground = paid(&mut g, p, "Groundchuck & Dirtbag");
        assert!(g.keywords(ground).contains(Keywords::TRAMPLE));
        let land = named(&mut g, p, landname, Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(a,Action::TapMana{card} if *card==land)
                    || matches!(a,Action::ActivateMana{card,ability:0,..} if *card==land)
            })
            .unwrap();
        checked(&mut g, action);
        assert_eq!(
            g.state.players[p.index()].mana[4],
            if landname == "Forest" { 2 } else { 1 }
        );
        assert!(g.state.stack.is_empty());
        g.state.players[p.index()].mana = [30; 6];
        let maker = paid(&mut g, p, "Weather Maker");
        g.state.players[p.index()].mana = [0; 6];
        mana(&mut g, maker, 0, [1, 0, 0, 0, 0, 0], Counters::default());
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
    }
}
#[test]
fn tmt_mana_weather_paid_landfall_named_charge_and_two_mana_counter_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let weather = paid(&mut g, p, "Weather Maker");
        let land = named(&mut g, p, "Forest", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: land });
        settle(&mut g);
        assert_eq!(g.counter_count(weather, CounterKind::Charge), 1);
        g.add_counter(weather, CounterKind::Charge, 1);
        g.state.players[p.index()].mana = [0; 6];
        mana(
            &mut g,
            weather,
            1,
            [0, 0, 0, 0, 0, 2],
            Counters::of(CounterKind::Charge, 2),
        );
        assert_eq!(g.counter_count(weather, CounterKind::Charge), 0);
        assert_eq!(g.state.players[p.index()].mana[5], 2);
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn tmt_mana_weather_damage_removes_three_only_charges_target_and_source_departure() {
    let p = Player::P0;
    let mut g = setup(p);
    let weather = paid(&mut g, p, "Weather Maker");
    g.add_counter(weather, CounterKind::Charge, 3);
    g.add_counter(weather, CounterKind::Lore, 3);
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: weather,
            ability: 2,
            counter_payment: Counters::of(CounterKind::Lore, 3),
        },
    );
    activate(
        &mut g,
        weather,
        2,
        Target::Player(p.other()),
        Counters::of(CounterKind::Charge, 3),
    );
    assert_eq!(g.counter_count(weather, CounterKind::Charge), 0);
    assert_eq!(g.counter_count(weather, CounterKind::Lore), 3);
    let life = g.state.players[p.other().index()].life;
    g.move_object(weather, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(g.state.players[p.other().index()].life, life - 3);
}

#[test]
fn tmt_mana_groundchuck_funding_exact_paid_cast_uses_one_forest_for_two_green() {
    let p = Player::P0;
    let mut g = setup(p);
    paid(&mut g, p, "Groundchuck & Dirtbag");
    let land = named(&mut g, p, "Forest", Zone::Battlefield);
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Hand);
    g.state.players[p.index()].mana = [0; 6];
    checked(&mut g, Action::AnnounceCast { card: bear });
    checked(&mut g, Action::TapMana { card: land });
    let payment = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(&mut g, payment);
    settle(&mut g);
    assert!(g.on_battlefield(bear));
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
#[test]
fn tmt_mana_restriction_views_distinguish_spell_and_activation_and_other_payments() {
    use mana_lots::{ManaRestrictionView as R, Purpose, Quality, Source};
    let spell = |name: &str| {
        let card = Card::from_name(name).unwrap();
        Source {
            card,
            kinds: card.kind_mask(),
            subtypes: GrantedSubtypes::default(),
            foreign_stash: false,
            exile_any_mana: false,
            mana_value: card.fixed_mana_value(),
        }
    };
    let quality = |restriction| Quality {
        restriction,
        spell_bonus: mana_lots::ManaSpellBonusView::None,
        retention: crate::cards::ManaRetention::Step,
        copy_source: None,
    };
    let punk = quality(R::ArtifactSpellOrActivation);
    assert!(punk.eligible(Purpose::Spell(spell("Weather Maker"))));
    assert!(!punk.eligible(Purpose::Spell(spell("Shock"))));
    assert!(punk.eligible(Purpose::Activation(spell("Grizzly Bears"))));
    assert!(!punk.eligible(Purpose::Other));
    let turtle = quality(R::SpellSubtypeAny(&["Ninja", "Turtle"]));
    for name in ["Foot Mystic", "Aegis Turtle"] {
        assert!(turtle.eligible(Purpose::Spell(spell(name))));
        assert!(!turtle.eligible(Purpose::Activation(spell(name))));
    }
    assert!(!turtle.eligible(Purpose::Other));
    assert!(!turtle.eligible(Purpose::Spell(spell("Grizzly Bears"))));
}
#[test]
fn tmt_mana_groundchuck_only_controls_own_lands_and_disappearing_source_stops_bonus() {
    let p = Player::P0;
    let mut g = setup(p);
    let ground = paid(&mut g, p, "Groundchuck & Dirtbag");
    let own = named(&mut g, p, "Forest", Zone::Battlefield);
    let opposing = named(&mut g, p.other(), "Forest", Zone::Battlefield);
    assert_eq!(g.mana_tap_bonus(own), [0, 0, 0, 0, 1, 0]);
    assert_eq!(g.mana_tap_bonus(opposing), [0; 6]);
    let before = g.snapshot();
    g.move_object(ground, Zone::Hand);
    assert_eq!(g.mana_tap_bonus(own), [0; 6]);
    g.restore(&before).unwrap();
    assert_eq!(g.mana_tap_bonus(own), [0, 0, 0, 0, 1, 0]);
}
#[test]
fn tmt_mana_weather_landfall_source_incarnation_and_illegal_target_fizzle() {
    let p = Player::P0;
    let mut g = setup(p);
    let weather = paid(&mut g, p, "Weather Maker");
    let land = named(&mut g, p, "Forest", Zone::Hand);
    checked(&mut g, Action::PlayLand { card: land });
    g.move_object(weather, Zone::Exile);
    g.move_object(weather, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.counter_count(weather, CounterKind::Charge), 0);
    g.add_counter(weather, CounterKind::Charge, 3);
    let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    activate(
        &mut g,
        weather,
        2,
        Target::Card(bear),
        Counters::of(CounterKind::Charge, 3),
    );
    g.move_object(bear, Zone::Hand);
    settle(&mut g);
    assert_eq!(g.state.objects[bear as usize].zone, Zone::Hand);
    assert_eq!(g.counter_count(weather, CounterKind::Charge), 0);
}
