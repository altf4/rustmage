//! Public name choices and bounded private three-zone search share owned resolution state.
mod support;
use rustmage::{Card, CardName};
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const SEARCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        &[
            Instruction::ChooseCardName { nonland: false },
            Instruction::ExileTargetNamedCards { maximum: 4 },
        ],
    ),
    ..CardRules::EMPTY
};
const NONLAND_ENTRY: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::NonlandCardName),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn reach_name(g: &mut Game) {
    for _ in 0..12 {
        if matches!(
            g.state.pending,
            Pending::CardName { .. } | Pending::EntryValue { .. }
        ) {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("name choice missing");
}
fn choose_name(g: &mut Game, name: &str) {
    act(
        g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CardName(CardName::from_name(name).unwrap()),
        },
    );
}
fn exile_pick(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::SelectResolutionExile { object });
}
fn begin(g: &mut Game, p: Player) {
    let spell = put(g, p, "Divination", Zone::Hand);
    cast(g, spell, Some(Target::Player(p.other())));
    reach_name(g);
}

#[test]
fn nonland_entry_choice_rejects_land_names_and_accepts_names_outside_playable_pool() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &NONLAND_ENTRY)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Short Bow", Zone::Hand);
        cast(&mut g, source, None);
        reach_name(&mut g);
        assert_eq!(
            g.object_view(source).zone,
            Zone::Stack,
            "choice precedes entry"
        );
        assert!(
            g.observe(p)
                .entry_value
                .as_ref()
                .unwrap()
                .opponent_hand
                .is_none()
        );
        reject(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CardName(CardName::from_name("Forest").unwrap()),
            },
        );
        let choice = CardName::from_name("Ancestral Recall").unwrap();
        act(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CardName(choice),
            },
        );
        settle(&mut g);
        assert_eq!(
            g.entry_value(g.object_ref(source)),
            Some(EntryChoiceValue::CardName(choice))
        );
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        g.move_object(source, Zone::Hand);
        assert_eq!(g.entry_value(g.object_ref(source)), None);
    }
}

#[test]
fn named_search_shares_one_four_card_cap_across_all_zones_without_hand_replacement_draws() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &SEARCH)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let cards: Vec<_> = [
            Zone::Graveyard,
            Zone::Graveyard,
            Zone::Hand,
            Zone::Library,
            Zone::Library,
        ]
        .into_iter()
        .map(|zone| put(&mut g, p.other(), "Grizzly Bears", zone))
        .collect();
        let own = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
        let wrong = put(&mut g, p.other(), "Llanowar Elves", Zone::Hand);
        begin(&mut g, p);
        choose_name(&mut g, "Grizzly Bears");
        for viewer in [p, p.other()] {
            assert_eq!(
                g.observe(viewer).chosen_card_name.unwrap().as_str(),
                "Grizzly Bears"
            );
        }
        let wrong_ref = g.object_ref(wrong);
        reject(&mut g, Action::SelectResolutionExile { object: wrong_ref });
        let own_ref = g.object_ref(own);
        reject(&mut g, Action::SelectResolutionExile { object: own_ref });
        for &id in cards.iter().take(4) {
            exile_pick(&mut g, id);
        }
        settle(&mut g);
        assert_eq!(
            cards
                .iter()
                .filter(|&&id| g.object_view(id).zone == Zone::Exile)
                .count(),
            4
        );
        assert_eq!(g.object_view(cards[4]).zone, Zone::Library);
        assert_eq!(g.object_view(own).zone, Zone::Graveyard);
        assert_eq!(g.object_view(wrong).zone, Zone::Hand);
        assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
        assert_eq!(g.observe(p).chosen_card_name, None);
    }
}

#[test]
fn search_is_optional_for_public_graveyard_copies_and_always_shuffles_even_at_zero() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &SEARCH)]);
    for available in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let public = available.then(|| put(&mut g, p.other(), "Grizzly Bears", Zone::Graveyard));
        begin(&mut g, p);
        choose_name(&mut g, "Grizzly Bears");
        let rng = g.state.rng.clone();
        assert!(matches!(g.state.pending, Pending::EffectChoice { .. }));
        act(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_ne!(g.state.rng, rng, "zero selection still shuffles library");
        if let Some(id) = public {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn search_inspection_is_private_and_no_hidden_matches_are_required_to_continue() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &SEARCH)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let hand = put(&mut g, p.other(), "Grizzly Bears", Zone::Hand);
    let library = put(&mut g, p.other(), "Llanowar Elves", Zone::Library);
    begin(&mut g, p);
    assert!(!g.observe(p).objects.iter().any(|o| o.id == hand));
    choose_name(&mut g, "Ancestral Recall");
    let view = g.observe(p);
    let inspected = view
        .effect_choice
        .as_ref()
        .unwrap()
        .inspected_cards
        .as_ref()
        .unwrap();
    assert!(
        inspected
            .iter()
            .any(|o| o.id == hand && o.card.name() == "Grizzly Bears")
    );
    assert!(
        inspected
            .iter()
            .any(|o| o.id == library && o.card.name() == "Llanowar Elves")
    );
    assert!(view.effect_choice.as_ref().unwrap().candidates.is_empty());
    assert!(g.observe(p.other()).effect_choice.is_none());
    act(&mut g, Action::FinishSearch);
    settle(&mut g);
    assert!(g.observe(p).effect_choice.is_none());
    assert!(
        !g.observe(p)
            .objects
            .iter()
            .any(|o| o.id == hand || o.id == library)
    );
}

#[test]
fn named_search_can_name_land_and_stop_below_cap_without_exiling_other_matches() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &SEARCH)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let public = put(&mut g, p.other(), "Forest", Zone::Graveyard);
    begin(&mut g, p);
    choose_name(&mut g, "Forest");
    exile_pick(&mut g, public);
    act(&mut g, Action::FinishSearch);
    settle(&mut g);
    assert_eq!(g.object_view(public).zone, Zone::Exile);
    assert_eq!(g.state.players[p.other().index()].library.len(), 8);
}

#[test]
fn private_name_search_raw_index_and_restore_keep_selection_and_shuffle_owned() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &SEARCH)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let card = put(&mut g, p.other(), "Grizzly Bears", Zone::Hand);
    begin(&mut g, p);
    choose_name(&mut g, "Grizzly Bears");
    let saved = g.snapshot();
    let action = Action::SelectResolutionExile {
        object: g.object_ref(card),
    };
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let rev = indexed.revision();
    act(&mut raw, action.clone());
    indexed.apply_index(p, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    reject(&mut raw, action);
    act(&mut raw, Action::FinishSearch);
    settle(&mut raw);
    indexed.restore(&saved).unwrap();
    drop(g);
    act(&mut indexed, Action::FinishSearch);
    settle(&mut indexed);
    assert_eq!(raw.object_view(card).zone, Zone::Exile);
    assert_eq!(indexed.object_view(card).zone, Zone::Hand);
    let mut restored = indexed.clone();
    restored.restore(&saved).unwrap();
    exile_pick(&mut restored, card);
    act(&mut restored, Action::FinishSearch);
    settle(&mut restored);
    assert_eq!(raw.snapshot(), restored.snapshot());
}

#[test]
fn split_and_room_cards_match_either_individual_outside_zone_name() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &SEARCH)]);
    for (identity, names) in [
        ("Cease // Desist", ["Cease", "Desist"]),
        (
            "Bottomless Pool // Locker Room",
            ["Bottomless Pool", "Locker Room"],
        ),
    ] {
        let card = Card::from_name(identity).unwrap();
        for name in names {
            for zone in [Zone::Hand, Zone::Library, Zone::Graveyard] {
                let mut g = scenario(Player::P0);
                // A named search reads card metadata even while that card's own
                // gameplay remains gated; it never casts or resolves the card.
                let id = put(&mut g, Player::P1, "Grizzly Bears", zone);
                g.state.objects[id as usize].card = card;
                begin(&mut g, Player::P0);
                choose_name(&mut g, name);
                exile_pick(&mut g, id);
                act(&mut g, Action::FinishSearch);
                settle(&mut g);
                assert_eq!(
                    g.object_view(id).zone,
                    Zone::Exile,
                    "{identity}, {name}, {zone:?}"
                );
            }
        }
    }
}

#[test]
fn double_faced_cards_match_the_front_name_only_outside_battlefield() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &SEARCH)]);
    let mut g = scenario(Player::P0);
    let id = put(&mut g, Player::P1, "Grizzly Bears", Zone::Graveyard);
    g.state.objects[id as usize].card = Card::from_name("Braided Net").unwrap();
    begin(&mut g, Player::P0);
    choose_name(&mut g, "Braided Quipu");
    let reference = g.object_ref(id);
    reject(&mut g, Action::SelectResolutionExile { object: reference });
    act(&mut g, Action::FinishSearch);
    settle(&mut g);
    begin(&mut g, Player::P0);
    choose_name(&mut g, "Braided Net");
    exile_pick(&mut g, id);
    act(&mut g, Action::FinishSearch);
    settle(&mut g);
    assert_eq!(g.object_view(id).zone, Zone::Exile);
}
