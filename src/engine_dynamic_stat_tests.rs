use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, owner, card, zone)
}
fn pt(g: &Game, id: CardId) -> (i128, i128) {
    let view = g.object_view(id);
    (view.power, view.toughness)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let result = g.snapshot();
    assert_eq!(fork.snapshot(), result);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), result);
}
fn aura(g: &mut Game, name: &str, target: CardId) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    let (mut payment, generic) = g.state.objects[id as usize].card.cost();
    payment[5] += generic;
    g.state.players[0].mana = payment;
    cast(g, id, Some(Target::Card(target)));
    pass_pair(g);
    id
}
const ZONES: &[Zone] = &[
    Zone::Library,
    Zone::Hand,
    Zone::Stack,
    Zone::Battlefield,
    Zone::Graveyard,
    Zone::Exile,
    Zone::Outside,
];

#[test]
fn creature_count_characteristics_work_in_every_zone_and_include_self_only_on_battlefield() {
    for name in ["Burrowguard Mentor", "Crusader of Odric", "Snow Villiers"] {
        for &zone in ZONES {
            let mut g = scenario();
            named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
            named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
            let id = named(&mut g, Player::P0, name, zone);
            let value = 1 + i128::from(zone == Zone::Battlefield);
            assert_eq!(
                pt(&g, id),
                (value, if name == "Snow Villiers" { 3 } else { value }),
                "{name} {zone:?}"
            );
        }
    }
}

#[test]
fn land_characteristics_count_existing_own_lands_in_all_zones() {
    for name in ["Mirkwood Pathmaker", "Harmonious Grovestrider"] {
        for &zone in ZONES {
            let mut g = scenario();
            for land in ["Forest", "Wastes", "Temple Garden"] {
                named(&mut g, Player::P0, land, Zone::Battlefield);
            }
            named(&mut g, Player::P1, "Forest", Zone::Battlefield);
            let id = named(&mut g, Player::P0, name, zone);
            assert_eq!(pt(&g, id), (3, 3));
        }
    }
}

#[test]
fn graveyard_and_exile_characteristics_count_cards_and_preserve_fixed_toughness() {
    let mut g = scenario();
    let drake = named(&mut g, Player::P0, "Enigma Drake", Zone::Hand);
    let cosmo = named(&mut g, Player::P0, "Cosmogoyf", Zone::Hand);
    assert_eq!(pt(&g, drake), (0, 4));
    assert_eq!(pt(&g, cosmo), (0, 1));
    named(&mut g, Player::P0, "Opt", Zone::Graveyard);
    named(&mut g, Player::P0, "Quick Study", Zone::Graveyard);
    named(&mut g, Player::P0, "Release the Dogs", Zone::Graveyard);
    named(&mut g, Player::P0, "Forest", Zone::Graveyard);
    named(&mut g, Player::P1, "Opt", Zone::Graveyard);
    assert_eq!(pt(&g, drake), (3, 4));
    let exiled = named(&mut g, Player::P0, "Forest", Zone::Exile);
    named(&mut g, Player::P1, "Opt", Zone::Exile);
    let token = Card::catalogue().find(|c| c.is_token()).unwrap();
    spawn(&mut g, Player::P0, token, Zone::Exile);
    assert_eq!(pt(&g, cosmo), (1, 2));
    let saved = g.snapshot();
    g.move_object(exiled, Zone::Hand);
    assert_eq!(pt(&g, cosmo), (0, 1));
    g.restore(&saved).unwrap();
    assert_eq!(pt(&g, cosmo), (1, 2));
}

#[test]
fn tarmogoyf_counts_distinct_card_types_across_both_graveyards_and_excludes_tokens() {
    let mut g = scenario();
    let goyf = named(&mut g, Player::P0, "Tarmogoyf", Zone::Hand);
    assert_eq!(pt(&g, goyf), (0, 1));
    named(&mut g, Player::P0, "Three Tree Mascot", Zone::Graveyard);
    assert_eq!(pt(&g, goyf), (2, 3));
    named(&mut g, Player::P1, "Opt", Zone::Graveyard);
    named(&mut g, Player::P1, "Release the Dogs", Zone::Graveyard);
    named(&mut g, Player::P0, "Plains", Zone::Graveyard);
    named(&mut g, Player::P1, "Ethereal Armor", Zone::Graveyard);
    assert_eq!(pt(&g, goyf), (6, 7));
    named(&mut g, Player::P0, "Shock", Zone::Graveyard);
    assert_eq!(pt(&g, goyf), (6, 7));
    let token = Card::catalogue().find(|c| c.is_token()).unwrap();
    spawn(&mut g, Player::P0, token, Zone::Graveyard);
    assert_eq!(pt(&g, goyf), (6, 7));
    // Type-only fixtures do not execute or admit unsupported abilities.
    for mask in [64, 128, 256] {
        if let Some(card) = Card::catalogue().find(|c| !c.is_token() && c.kind_mask() & mask != 0) {
            let before = pt(&g, goyf).0;
            spawn(&mut g, Player::P1, card, Zone::Graveyard);
            assert_eq!(pt(&g, goyf), (before + 1, before + 2));
        }
    }
}

#[test]
fn resolving_instant_reaches_graveyard_before_tarmogoyf_damage_sba() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Forest", Zone::Graveyard);
    named(&mut g, Player::P0, "Release the Dogs", Zone::Graveyard);
    let goyf = named(&mut g, Player::P0, "Tarmogoyf", Zone::Battlefield);
    assert_eq!(pt(&g, goyf), (2, 3));
    let strike = named(&mut g, Player::P1, "Lightning Strike", Zone::Hand);
    g.state.priority = Player::P1;
    g.state.players[1].mana[3] = 2;
    cast(&mut g, strike, Some(Target::Card(goyf)));
    let saved = g.snapshot();
    pass_pair(&mut g);
    assert_eq!(pt(&g, goyf), (3, 4));
    assert_eq!(g.object_view(goyf).damage, 3);
    assert_eq!(g.object_view(goyf).zone, Zone::Battlefield);
    let expected = g.snapshot();
    g.restore(&saved).unwrap();
    pass_pair(&mut g);
    assert_eq!(g.snapshot(), expected);
    g.move_object(strike, Zone::Exile);
    g.state_based_actions();
    assert_eq!(g.object_view(goyf).zone, Zone::Graveyard);
    assert_eq!(pt(&g, goyf), (3, 4));
}

#[test]
fn simultaneous_departures_capture_pre_event_characteristics_before_graveyard_counts_change() {
    for reverse in [false, true] {
        let mut g = scenario();
        let goyf = named(&mut g, Player::P0, "Tarmogoyf", Zone::Battlefield);
        let robot = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
        assert_eq!(pt(&g, goyf), (0, 1));
        let ids = if reverse {
            vec![robot, goyf]
        } else {
            vec![goyf, robot]
        };
        let events = g.move_objects_raw(&ids, Zone::Graveyard);
        let before = &events.iter().find(|e| e.before.id == goyf).unwrap().before;
        assert_eq!((before.power, before.toughness), (0, 1));
        assert_eq!(pt(&g, goyf), (2, 3));
    }
}

#[test]
fn additive_scaling_tracks_subtypes_equipment_artifacts_and_basic_land_status() {
    let mut g = scenario();
    let steiner = named(&mut g, Player::P0, "Adelbert Steiner", Zone::Battlefield);
    let goblin = named(&mut g, Player::P0, "Boneclub Berserker", Zone::Battlefield);
    let guide = named(
        &mut g,
        Player::P0,
        "Guidelight Synergist",
        Zone::Battlefield,
    );
    let djinn = named(&mut g, Player::P0, "Tempest Djinn", Zone::Battlefield);
    assert_eq!(pt(&g, steiner), (2, 1));
    assert_eq!(pt(&g, goblin), (2, 4));
    assert_eq!(pt(&g, guide), (1, 4));
    let sword = named(&mut g, Player::P0, "Short Bow", Zone::Battlefield); // Unattached Equipment still counts.
    let changeling = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    named(&mut g, Player::P1, "Short Bow", Zone::Battlefield);
    named(&mut g, Player::P0, "Island", Zone::Battlefield);
    named(&mut g, Player::P0, "Hallowed Fountain", Zone::Battlefield);
    named(&mut g, Player::P0, "Wastes", Zone::Battlefield);
    named(&mut g, Player::P1, "Island", Zone::Battlefield);
    assert_eq!(pt(&g, steiner), (3, 2));
    assert_eq!(pt(&g, goblin), (4, 4));
    assert_eq!(pt(&g, guide), (3, 4));
    assert_eq!(pt(&g, djinn), (1, 4));
    let saved = g.snapshot();
    g.move_object(changeling, Zone::Hand);
    g.move_object(sword, Zone::Hand);
    assert_eq!(pt(&g, steiner), (2, 1));
    assert_eq!(pt(&g, goblin), (2, 4));
    assert_eq!(pt(&g, guide), (1, 4));
    g.restore(&saved).unwrap();
    assert_eq!(pt(&g, goblin), (4, 4));
    g.move_object(goblin, Zone::Hand);
    assert_eq!(pt(&g, goblin), (2, 4)); // Not a CDA.
}

#[test]
fn summed_and_excluded_graveyard_counts_are_distinct_from_permanent_counts() {
    let mut g = scenario();
    let adherent = named(
        &mut g,
        Player::P0,
        "Moon-Vigil Adherents",
        Zone::Battlefield,
    );
    let xande = named(&mut g, Player::P0, "Xande, Dark Mage", Zone::Battlefield);
    assert_eq!(pt(&g, adherent), (2, 2));
    assert_eq!(pt(&g, xande), (3, 3));
    named(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    named(&mut g, Player::P0, "Opt", Zone::Graveyard);
    named(&mut g, Player::P0, "Short Bow", Zone::Graveyard);
    named(&mut g, Player::P0, "Forest", Zone::Graveyard);
    named(&mut g, Player::P1, "Short Bow", Zone::Graveyard);
    assert_eq!(pt(&g, adherent), (3, 3));
    assert_eq!(pt(&g, xande), (5, 5));
}

#[test]
fn characteristic_nonland_and_mana_value_counts_include_source_and_ignore_opponents() {
    let mut g = scenario();
    let bunnicorn = named(&mut g, Player::P0, "Regal Bunnicorn", Zone::Battlefield);
    let kraven = named(
        &mut g,
        Player::P0,
        "Kraven, Proud Predator",
        Zone::Battlefield,
    );
    named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    named(&mut g, Player::P1, "Gigantosaurus", Zone::Battlefield);
    assert_eq!(pt(&g, bunnicorn), (2, 2));
    assert_eq!(pt(&g, kraven), (3, 4));
    let gear = named(&mut g, Player::P0, "Hedron Archive", Zone::Battlefield);
    assert_eq!(pt(&g, bunnicorn), (3, 3));
    assert_eq!(pt(&g, kraven), (4, 4));
    g.move_object(gear, Zone::Hand);
    assert_eq!(pt(&g, kraven), (3, 4));
}

#[test]
fn attached_scaling_counts_itself_and_recomputes_controller_land_counts() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    named(&mut g, Player::P0, "Temple Garden", Zone::Battlefield);
    named(&mut g, Player::P1, "Forest", Zone::Battlefield);
    let blanch = aura(&mut g, "Blanchwood Armor", bear);
    assert_eq!(pt(&g, bear), (4, 4));
    let ethereal = aura(&mut g, "Ethereal Armor", bear);
    assert_eq!(pt(&g, bear), (6, 6));
    assert!(g.keywords(bear).contains(Keywords::FIRST_STRIKE));
    let saved = g.snapshot();
    g.move_object(blanch, Zone::Hand);
    assert_eq!(pt(&g, bear), (3, 3));
    assert_eq!(
        g.object_view(ethereal).attached_to,
        Some(g.object_ref(bear))
    );
    g.restore(&saved).unwrap();
    assert_eq!(pt(&g, bear), (6, 6));
    let other = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    // Own Aura on an opposing creature still counts the Aura controller's lands.
    let own = aura(&mut g, "Blanchwood Armor", other);
    assert_eq!(pt(&g, other), (4, 4));
    g.move_object(other, Zone::Hand);
    g.state_based_actions();
    assert_eq!(g.object_view(own).zone, Zone::Graveyard);
}

#[test]
fn characteristic_base_precedes_counters_temporary_growth_and_static_anthems() {
    let mut g = scenario();
    let goyf = named(&mut g, Player::P0, "Tarmogoyf", Zone::Battlefield);
    g.state.objects[goyf as usize].counters.plus_one = 2;
    g.state.objects[goyf as usize].counters.minus_one = 1;
    named(&mut g, Player::P0, "Anthem of Champions", Zone::Battlefield);
    assert_eq!(pt(&g, goyf), (2, 3));
    let grow = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    g.state.players[0].mana[4] = 1;
    replay(
        &mut g,
        Action::Cast {
            card: grow,
            target: Some(Target::Card(goyf)),
            payment: [0, 0, 0, 0, 1, 0],
        },
    );
    pass_pair(&mut g);
    assert_eq!(pt(&g, goyf), (6, 7));
    g.move_object(goyf, Zone::Graveyard);
    assert_eq!(pt(&g, goyf), (2, 3));
}

#[test]
fn dynamic_toughness_changes_participate_in_repeated_state_based_checks() {
    let mut g = scenario();
    let mentor = named(&mut g, Player::P0, "Burrowguard Mentor", Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(pt(&g, mentor), (2, 2));
    g.state.objects[mentor as usize].damage = 1;
    g.state.objects[bear as usize].damage = 2;
    let saved = g.snapshot();
    g.state_based_actions();
    let expected = g.snapshot();
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    assert_eq!(g.object_view(mentor).zone, Zone::Graveyard);
    g.restore(&saved).unwrap();
    g.state_based_actions();
    assert_eq!(g.snapshot(), expected);
    let forest = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    let path = named(&mut g, Player::P0, "Mirkwood Pathmaker", Zone::Battlefield);
    assert_eq!(pt(&g, path), (1, 1));
    g.move_object(forest, Zone::Hand);
    g.state_based_actions();
    assert_eq!(g.object_view(path).zone, Zone::Graveyard);
    assert_eq!(pt(&g, path), (0, 0));
}

#[test]
fn all_scaling_sources_are_indexed_and_round_trip_source_removal() {
    let mut g = scenario();
    let cards: Vec<_> = Card::catalogue()
        .filter(|c| !c.scaling_modifiers().is_empty())
        .collect();
    for name in [
        "Magitek Infantry",
        "Outcaster Greenblade",
        "Armory Mice",
        "Grand Ball Guest",
        "Tuinvale Guide",
        "Stingerback Terror",
        "Machinist's Arsenal",
        "Angel of Vitality",
    ] {
        assert!(cards.contains(&Card::from_name(name).unwrap()), "{name}");
    }
    for card in cards {
        let id = if card.is_token() {
            // Tokens cannot enter from a hand; exercise their actual creation path.
            g.create_tokens(Player::P0, card, 1, false);
            g.state
                .objects
                .iter()
                .enumerate()
                .find(|(_, o)| o.card == card && o.zone == Zone::Battlefield)
                .unwrap()
                .0 as CardId
        } else {
            let id = spawn(&mut g, Player::P0, card, Zone::Hand);
            assert!(!g.state.static_sources.contains(&id));
            // Source-index structural fixture, not a claim of legal unattached Aura play.
            g.move_object(id, Zone::Battlefield);
            id
        };
        assert_eq!(
            g.state.static_sources.iter().filter(|&&s| s == id).count(),
            1
        );
        let saved = g.snapshot();
        g.move_object(id, Zone::Hand);
        assert!(!g.state.static_sources.contains(&id));
        g.restore(&saved).unwrap();
        assert!(g.state.static_sources.contains(&id));
        g.move_object(id, Zone::Hand);
    }
}

#[test]
fn normal_card_type_masks_distinguish_types_from_supertypes_and_subtypes() {
    for card in Card::catalogue() {
        if card.info().type_line.contains("//") {
            continue;
        }
        let types: Vec<_> = card
            .info()
            .type_line
            .split(" — ")
            .next()
            .unwrap()
            .split_whitespace()
            .collect();
        let expected = [
            ("Creature", 1),
            ("Land", 2),
            ("Sorcery", 4),
            ("Instant", 8),
            ("Artifact", 16),
            ("Enchantment", 32),
            ("Planeswalker", 64),
            ("Kindred", 128),
            ("Battle", 256),
        ]
        .into_iter()
        .filter(|(name, _)| types.contains(name))
        .fold(0u16, |mask, (_, bit)| mask | bit);
        assert_eq!(card.kind_mask(), expected, "{}", card.name());
    }
}

#[test]
fn newly_admitted_characteristic_source_keeps_its_printed_ward_trigger() {
    let mut g = scenario();
    for _ in 0..3 {
        named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    }
    let source = named(
        &mut g,
        Player::P0,
        "Harmonious Grovestrider",
        Zone::Battlefield,
    );
    let murder = named(&mut g, Player::P1, "Murder", Zone::Hand);
    g.state.priority = Player::P1;
    g.state.players[1].mana[2] = 3;
    cast(&mut g, murder, Some(Target::Card(source)));
    pass_pair(&mut g);
    assert_eq!(
        g.state.pending,
        Pending::ResolutionMana { player: Player::P1 }
    );
    replay(&mut g, Action::DeclineResolutionMana);
    assert_eq!(g.object_view(murder).zone, Zone::Graveyard);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    assert_eq!(pt(&g, source), (3, 3));
}
