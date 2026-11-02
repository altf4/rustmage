//! Actual land entries, max-speed activations, chosen mana and cycling.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn announce_ability(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn mana(g: &mut Game, source: CardId, ability: u16) {
    choose(
        g,
        |a| matches!(a, Action::ActivateMana { card, ability: n, .. } if *card == source && *n == ability),
    );
}

fn drain(g: &mut Game) {
    for _ in 0..48 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("bounded land resolution");
}

#[test]
fn all_three_raceways_really_start_engines_and_supply_unconditional_colorless_mana() {
    for name in ["Amonkhet Raceway", "Avishkar Raceway", "Muraganda Raceway"] {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p);
            let land = put(&mut g, p, name, Zone::Hand);
            assert_eq!(g.state.players[p.index()].speed, 0);
            act(&mut g, Action::PlayLand { card: land });
            assert_eq!(g.object_view(land).zone, Zone::Battlefield);
            assert!(!g.object_view(land).tapped);
            assert_eq!(g.state.players[p.index()].speed, 1, "{name}");
            assert!(g.state.stack.is_empty());
            g.state.players[p.index()].mana = [0; 6];
            mana(&mut g, land, 0);
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
            assert!(g.object_view(land).tapped);
        }
    }
}

#[test]
fn muraganda_max_speed_mana_is_two_colorless_and_uses_one_tap_at_instant_speed() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Muraganda Raceway", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        g.state.players[p.index()].speed = 3;
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ActivateMana {card, ability:1,..} if *card==land))
        );
        g.state.players[p.index()].speed = 4;
        g.state.step = Step::Upkeep;
        mana(&mut g, land, 1);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 2]);
        assert!(g.object_view(land).tapped);
        assert!(g.state.stack.is_empty());
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ActivateMana {card,..} if *card==land))
        );
    }
}

#[test]
fn amonkhet_haste_targets_either_player_and_survives_source_departure_until_cleanup() {
    for p in [Player::P0, Player::P1] {
        for own in [false, true] {
            let mut g = scenario(p);
            let land = put(&mut g, p, "Amonkhet Raceway", Zone::Battlefield);
            let creature = put(
                &mut g,
                if own { p } else { p.other() },
                "Grizzly Bears",
                Zone::Battlefield,
            );
            let artifact = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
            g.state.players[p.index()].speed = 3;
            reject(&mut g, announce_ability(land, 1));
            g.state.players[p.index()].speed = 4;
            act(&mut g, announce_ability(land, 1));
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(artifact),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            assert!(g.object_view(land).tapped);
            let mut stale = g.clone();
            stale.move_object(creature, Zone::Exile);
            stale.move_object(creature, Zone::Battlefield);
            drain(&mut stale);
            assert!(!stale.keywords(creature).contains(Keywords::HASTE));
            g.move_object(land, Zone::Graveyard);
            drain(&mut g);
            assert!(g.keywords(creature).contains(Keywords::HASTE));
            g.state.step = Step::End;
            g.advance_step();
            assert!(!g.keywords(creature).contains(Keywords::HASTE));
        }
    }
}

#[test]
fn avishkar_pays_three_taps_and_discards_before_drawing_with_no_empty_hand_route() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Avishkar Raceway", Zone::Battlefield);
        g.state.players[p.index()].speed = 4;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        reject(&mut g, announce_ability(land, 1));
        let discard = put(&mut g, p, "Forest", Zone::Hand);
        let foreign = put(&mut g, p.other(), "Forest", Zone::Hand);
        g.state.players[p.index()].speed = 3;
        reject(&mut g, announce_ability(land, 1));
        g.state.players[p.index()].speed = 4;
        let library_before = g.state.players[p.index()].library.len();
        let draws_before = g.state.players[p.index()].cards_drawn_this_turn;
        act(&mut g, announce_ability(land, 1));
        assert!(g.observe(p.other()).additional_cost.is_none());
        let object = g.object_ref(foreign);
        reject(&mut g, Action::ChooseAdditionalCost { object });
        let object = g.object_ref(discard);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert!(g.object_view(land).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.state.players[p.index()].cards_drawn_this_turn,
            draws_before
        );
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), library_before - 1);
        assert_eq!(
            g.state.players[p.index()].cards_drawn_this_turn,
            draws_before + 1
        );
        assert_eq!(g.object_view(foreign).zone, Zone::Hand);
    }
}

#[test]
fn night_market_chooses_every_color_as_it_enters_tapped_then_produces_only_that_color() {
    for p in [Player::P0, Player::P1] {
        for (color, choice) in ChoiceColor::ALL.into_iter().enumerate() {
            let mut g = scenario(p);
            let land = put(&mut g, p, "Night Market", Zone::Hand);
            act(&mut g, Action::PlayLand { card: land });
            assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
            act(
                &mut g,
                Action::ChooseEntryValue {
                    value: EntryChoiceValue::Color(choice),
                },
            );
            drain(&mut g);
            assert_eq!(g.object_view(land).zone, Zone::Battlefield);
            assert!(g.object_view(land).tapped);
            assert_eq!(
                g.object_view(land).entry_choice,
                Some(EntryChoiceValue::Color(choice))
            );
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ActivateMana{card,..}if *card==land))
            );
            g.state.objects[land as usize].tapped = false;
            g.state.players[p.index()].mana = [0; 6];
            mana(&mut g, land, 0);
            let mut expected = [0; 6];
            expected[color] = 1;
            assert_eq!(g.state.players[p.index()].mana, expected);
            assert!(g.object_view(land).tapped);
        }
    }
}

#[test]
fn night_market_entry_choice_owns_independent_raw_index_and_restored_branches() {
    let mut parent = scenario(Player::P0);
    let land = put(&mut parent, Player::P0, "Night Market", Zone::Hand);
    act(&mut parent, Action::PlayLand { card: land });
    let saved = parent.snapshot();
    let action = Action::ChooseEntryValue {
        value: EntryChoiceValue::Color(ChoiceColor::Blue),
    };
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut other = parent.clone();
    let at = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let actor = indexed.actor().unwrap();
    let revision = indexed.revision();
    act(&mut raw, action);
    indexed.apply_index(actor, revision, at).unwrap();
    drop(parent);
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    other.restore(&saved).unwrap();
    act(
        &mut other,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::Color(ChoiceColor::Green),
        },
    );
    drain(&mut other);
    assert_eq!(
        raw.object_view(land).entry_choice,
        Some(EntryChoiceValue::Color(ChoiceColor::Blue))
    );
    assert_eq!(
        other.object_view(land).entry_choice,
        Some(EntryChoiceValue::Color(ChoiceColor::Green))
    );
    raw.move_object(land, Zone::Exile);
    assert!(raw.object_view(land).entry_choice.is_none());
    raw.move_object(land, Zone::Battlefield);
    assert!(matches!(raw.state.pending, Pending::EntryValue { .. }));
    act(
        &mut raw,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::Color(ChoiceColor::Red),
        },
    );
    assert_eq!(
        raw.object_view(land).entry_choice,
        Some(EntryChoiceValue::Color(ChoiceColor::Red))
    );
}

#[test]
fn night_market_cycles_for_three_at_instant_speed_without_entering_or_choosing_a_color() {
    let p = Player::P1;
    let mut g = scenario(Player::P0);
    let land = put(&mut g, p, "Night Market", Zone::Hand);
    g.state.priority = p;
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    reject(&mut g, announce_ability(land, 1));
    g.state.players[p.index()].mana[5] = 3;
    let before = g.state.players[p.index()].library.len();
    act(&mut g, announce_ability(land, 1));
    pay(&mut g);
    assert_eq!(g.object_view(land).zone, Zone::Graveyard);
    assert!(g.object_view(land).entry_choice.is_none());
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.state.players[p.index()].library.len(), before);
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].library.len(), before - 1);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    assert_eq!(g.state.players[p.index()].speed, 0);
}
