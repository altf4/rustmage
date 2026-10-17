//! Shared entry-choice rules regressions, independent of admission flags.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::tests::spawn;
use super::*;
use crate::cards::{ChoiceColor, CreatureType};
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    spawn(g, p, card, zone)
}
fn color(c: ChoiceColor) -> EntryChoiceValue {
    EntryChoiceValue::Color(c)
}
fn typ(name: &str) -> EntryChoiceValue {
    EntryChoiceValue::CreatureType(CreatureType::from_name(name).unwrap())
}
fn choose(g: &mut Game, value: EntryChoiceValue) {
    checked(g, Action::ChooseEntryValue { value });
}
fn paid_entry(g: &mut Game, id: CardId, value: EntryChoiceValue) {
    cast(g, id, None, &[]);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
    choose(g, value);
    settle(g);
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
#[test]
fn entry_choices_paid_land_all_colors_and_public_preentry_values_are_atomic() {
    for p in [Player::P0, Player::P1] {
        for chosen in ChoiceColor::ALL {
            let mut g = setup(p);
            let haven = fixture(&mut g, p, "Uncharted Haven", Zone::Hand);
            assert!(g.state.entry_values.is_none());
            checked(&mut g, Action::PlayLand { card: haven });
            assert_eq!(g.object_view(haven).zone, Zone::Hand);
            assert_eq!(
                g.legal_actions()
                    .iter()
                    .filter(|a| matches!(a, Action::ChooseEntryValue { .. }))
                    .count(),
                5
            );
            for viewer in [p, p.other()] {
                assert_eq!(
                    g.observe(viewer).entry_value.unwrap().card,
                    Card::from_name("Uncharted Haven").unwrap()
                );
            }
            reject(&mut g, Action::Pass);
            reject(&mut g, Action::ChooseEntryValue { value: typ("Bear") });
            choose(&mut g, color(chosen));
            assert!(g.object_view(haven).tapped);
            assert_eq!(g.object_view(haven).entry_choice, Some(color(chosen)));
            g.state.objects[haven as usize].tapped = false;
            let activation = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a,Action::ActivateMana {card,..} if *card==haven))
                .expect("chosen mana ability");
            let before = g.state.players[p.index()].mana;
            checked(&mut g, activation);
            let mut expected = before;
            expected[chosen.index()] += 1;
            assert_eq!(g.state.players[p.index()].mana, expected);
        }
    }
    assert!(serde_json::from_str::<EntryChoiceValue>(r#"{"Color":"Colorless"}"#).is_err());
    assert!(serde_json::from_str::<CreatureType>(r#""Equipment""#).is_err());
    let taxonomy: serde_json::Value =
        serde_json::from_str(include_str!("../data/creature-types.json")).unwrap();
    let expected: Vec<&str> = taxonomy["types"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();
    assert_eq!(crate::CREATURE_TYPES, expected);
    assert!(CreatureType::from_name("Time Lord").is_some());
}
#[test]
fn entry_choices_paid_mare_cast_event_uses_choice_controller_and_departure_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mare = fixture(&mut g, p, "Diamond Mare", Zone::Hand);
        paid_entry(&mut g, mare, color(ChoiceColor::Red));
        let life = g.state.players[p.index()].life;
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, bear, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        let frozen = g.object_view(mare);
        assert_eq!(frozen.entry_choice, Some(color(ChoiceColor::Red)));
        // The already queued cast trigger is independent; removing its exact
        // source before resolution must not lose its chosen-color qualification.
        g.move_object(mare, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 1);
        assert_eq!(g.object_view(mare).entry_choice, None);
        assert_eq!(frozen.entry_choice, Some(color(ChoiceColor::Red)));
    }
}
#[test]
fn entry_choices_paid_banner_own_color_bonus_mana_and_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let banner = fixture(&mut g, p, "Heraldic Banner", Zone::Hand);
        paid_entry(&mut g, banner, color(ChoiceColor::Green));
        assert_eq!(g.stats(own).power, 3);
        assert_eq!(g.stats(enemy).power, 2);
        g.state.players[p.index()].mana = [0; 6];
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: elf }),
            "funding sees chosen green mana"
        );
        // Removal can apply to an animated artifact in future; the shared source
        // availability path is directly covered on Automaton below.
        let destroy = named(&mut g, p, "Disenchant", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        cast(&mut g, destroy, None, &[Target::Card(banner)]);
        settle(&mut g);
        assert_eq!(g.stats(own).power, 2);
        assert!(g.state.entry_values.is_none());
    }
}
#[test]
fn entry_choices_automaton_arbitrary_types_layers_removal_control_and_reentry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let automaton = fixture(&mut g, p, "Adaptive Automaton", Zone::Hand);
        paid_entry(&mut g, automaton, typ("Bear"));
        assert!(g.has_subtype(automaton, "Construct"));
        assert!(g.has_subtype(automaton, "Bear"));
        assert_eq!(g.stats(automaton).power, 2);
        assert_eq!(g.stats(bear).power, 3);
        let removal = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, removal, None, &[Target::Card(automaton)]);
        settle(&mut g);
        assert!(!g.printed_abilities_available(automaton));
        assert!(
            g.has_subtype(automaton, "Bear"),
            "layer4 type addition precedes layer6 removal"
        );
        assert_eq!(g.stats(bear).power, 2);
        let destroy = named(&mut g, p, "Disenchant", Zone::Hand);
        cast(&mut g, destroy, None, &[Target::Card(removal)]);
        settle(&mut g);
        g.state.active = p.other();
        g.state.priority = p.other();
        let steal = named(&mut g, p.other(), "Involuntary Employment", Zone::Hand);
        cast(&mut g, steal, None, &[Target::Card(automaton)]);
        settle(&mut g);
        assert_eq!(g.object_view(automaton).controller, p.other());
        assert_eq!(g.object_view(automaton).entry_choice, Some(typ("Bear")));
        assert_eq!(g.stats(bear).power, 2);
        g.state.active = p;
        g.state.priority = p;
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(automaton)]);
        settle(&mut g);
        assert_eq!(g.object_view(automaton).entry_choice, None);
        paid_entry(&mut g, automaton, typ("Time Lord"));
        assert!(g.has_subtype(automaton, "Time Lord"));
        assert!(!g.has_subtype(automaton, "Bear"));
        assert_eq!(g.stats(bear).power, 2);
        // A newer resolution-created type replacement overwrites the older
        // chosen-type addition, but does not erase the underlying public choice.
        let source = named(&mut g, p, "Shock", Zone::Hand);
        let mut item = g.cast_item(p, source, [0; 6]);
        item.targets.push(BoundTarget {
            group: 0,
            target: Target::Card(automaton),
            object: Some(g.object_ref(automaton)),
        });
        g.set_creature_types(&item, Subject::Targets, &["Werewolf"]);
        assert!(g.has_subtype(automaton, "Werewolf"));
        assert!(!g.has_subtype(automaton, "Time Lord"));
        assert!(!g.has_subtype(automaton, "Construct"));
        assert_eq!(
            g.object_view(automaton).entry_choice,
            Some(typ("Time Lord"))
        );
    }
}
#[test]
fn entry_choices_kinship_counts_preentry_creatures_and_doubles_named_entry_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
        named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let banner = fixture(&mut g, p, "Banner of Kinship", Zone::Hand);
        paid_entry(&mut g, banner, typ("Bear"));
        assert_eq!(g.counter_count(banner, CounterKind::Fellowship), 4);
        assert_eq!(g.stats(bear).power, 6);
        let mut counters = g.counter_amounts(banner).to_owned();
        counters.set(CounterKind::Fellowship, 1);
        g.set_counters(banner, counters);
        assert_eq!(g.stats(bear).power, 3);
        let frozen = g.snapshot();
        let clone = g.clone();
        assert!(
            g.state
                .entry_values
                .as_ref()
                .unwrap()
                .ptr_eq(clone.state.entry_values.as_ref().unwrap())
        );
        let card = named(&mut g, p, "Unsummon", Zone::Hand);
        g.move_object(card, Zone::Graveyard);
        assert!(
            g.state
                .entry_values
                .as_ref()
                .unwrap()
                .ptr_eq(clone.state.entry_values.as_ref().unwrap()),
            "unrelated move does not detach cold choices"
        );
        g.restore(&frozen).unwrap();
    }
}
#[test]
fn entry_choices_simultaneous_apnap_private_bounds_and_preentry_counts() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let existing = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let banner = fixture(&mut g, p, "Banner of Kinship", Zone::Graveyard);
        let incoming = fixture(&mut g, p, "Adaptive Automaton", Zone::Graveyard);
        let private = fixture(&mut g, p.other(), "Diamond Mare", Zone::Hand);
        let moves = [private, banner, incoming]
            .into_iter()
            .map(|id| g.entry_move(id, Zone::Battlefield, false))
            .collect();
        g.stage_entries(moves, None);
        assert!(g.advance_entries());
        assert_eq!(g.actor(), Some(p));
        choose(&mut g, typ("Bear"));
        assert!(g.observe(p).entry_value.is_some());
        choose(&mut g, typ("Bear"));
        assert_eq!(g.actor(), Some(p.other()));
        assert!(
            g.observe(p).entry_value.is_none(),
            "private incoming identity and choice remain hidden from foreign viewer"
        );
        assert_eq!(g.object_view(incoming).zone, Zone::Graveyard);
        assert_eq!(g.object_view(banner).zone, Zone::Graveyard);
        choose(&mut g, color(ChoiceColor::White));
        assert!(
            !matches!(g.state.pending, Pending::EntryTimestamp { .. }),
            "commuting layer-seven choice bonuses need no timeline registration"
        );
        settle(&mut g);
        assert_eq!(
            g.counter_count(banner, CounterKind::Fellowship),
            1,
            "simultaneous incoming chosen-type creature is not already controlled"
        );
        assert_eq!(
            g.stats(existing).power,
            4,
            "post-entry world sees both independent lord effects"
        );
        assert_eq!(g.stats(incoming).power, 3);
        for viewer in [p, p.other()] {
            let wire = serde_json::to_value(g.observe(viewer)).unwrap();
            assert!(
                wire["objects"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|o| o["id"] == incoming && o["entry_choice"]["CreatureType"] == "Bear")
            );
        }
    }
}

#[test]
fn entry_choices_cold_layout_and_empty_root_share_cost_are_bounded() {
    let g = setup(Player::P0);
    assert!(g.state.entry_values.is_none());
    assert!(
        std::mem::size_of::<GrantedSubtypes>()
            <= std::mem::size_of::<(u16, Option<&'static [&'static str]>)>()
    );
    eprintln!(
        "choice layout: State={} Object={} GrantedSubtypes={} ChoiceRow={} EntryMove={} initial_bytes={}",
        std::mem::size_of::<State>(),
        std::mem::size_of::<Object>(),
        std::mem::size_of::<GrantedSubtypes>(),
        std::mem::size_of::<entry_choices::ChoiceRow>(),
        std::mem::size_of::<entries::EntryMove>(),
        g.state_bytes()
    );
}
