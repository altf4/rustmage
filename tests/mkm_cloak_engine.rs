//! Private hand and library choices produce exact cloaked entries.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const HAND: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::CloakFromHand]),
    ..CardRules::EMPTY
};
const LIBRARY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 4,
                count: LibrarySelectionCount::Fixed(2),
                filter: LibrarySelectionFilter::Any,
                optional: false,
                reveal: false,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(4),
            destination: LibraryPickDestination::Cloak,
            public: false,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};
const ENTRY_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersBatchYou {
            predicate: EntryPredicate::FaceDown,
            other: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn start(game: &mut Game, player: Player, spell: Card) {
    let cast_id = spawn(game, player, spell, Zone::Hand);
    cast(game, cast_id, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn hand_cloak_accepts_noncreature_cards_and_keeps_the_choice_private() {
    let spell = Card::from_name("Opt").unwrap();
    let _scope = fixture_rules::activate(&[(spell, &HAND)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let aura = put(&mut game, player, "Unable to Scream", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    let opposing = put(&mut game, player.other(), "Shock", Zone::Hand);
    let aura_ref = game.object_ref(aura);
    let land_ref = game.object_ref(land);
    game.state.known_hand.extend([aura_ref, land_ref]);
    start(&mut game, player, spell);
    assert_eq!(game.actor(), Some(player));
    assert!(game.observe(player).effect_choice.is_some());
    assert!(game.observe(player.other()).effect_choice.is_none());
    let other = game.object_ref(opposing);
    reject(&mut game, Action::ChooseEffectObject { object: other });

    let choose_aura = Action::ChooseEffectObject {
        object: game.object_ref(aura),
    };
    let saved = game.snapshot();
    let mut branch = game.clone();
    let index = branch
        .legal_actions()
        .iter()
        .position(|action| *action == choose_aura)
        .unwrap();
    act(&mut game, choose_aura);
    branch
        .apply_index(player, branch.revision(), index)
        .unwrap();
    assert_eq!(game.snapshot(), branch.snapshot());
    assert_eq!(game.object_view(land).zone, Zone::Hand);
    let public = &game.observe(player.other()).face_down_permanents[0];
    assert_ne!(public.object.card, aura);
    assert_ne!(public.object.card, land);
    assert_eq!(stats(&game, public.object.card), (2, 2));
    assert_eq!(public.cause, FaceDownCause::Cloak);
    assert_eq!(public.underlying, None);
    assert_eq!(
        game.observe(player).face_down_permanents[0].underlying,
        Card::from_name("Unable to Scream")
    );

    branch.restore(&saved).unwrap();
    act(&mut branch, Action::ChooseEffectObject { object: land_ref });
    assert_eq!(branch.object_view(aura).zone, Zone::Hand);
    assert_eq!(
        branch.observe(player).face_down_permanents[0].underlying,
        Some(Card::Forest)
    );
    let public = |game: &Game| {
        let mut view = game.observe(player.other());
        view.revision = 0;
        serde_json::to_value(view).unwrap()
    };
    assert_eq!(public(&branch), public(&game));
}

#[test]
fn two_selected_library_cards_cloak_separately_then_the_rest_go_on_bottom() {
    let spell = Card::from_name("Opt").unwrap();
    let observer = Card::from_name("Grizzly Bears").unwrap();
    let _scope = fixture_rules::activate(&[(spell, &LIBRARY), (observer, &ENTRY_OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    for id in game.state.players[player.index()].library.clone() {
        game.move_object(id, Zone::Exile);
    }
    let outside = put(&mut game, player, "Forest", Zone::Library);
    let first = put(&mut game, player, "Island", Zone::Library);
    let rest1 = put(&mut game, player, "Forest", Zone::Library);
    let second = put(&mut game, player, "Shock", Zone::Library);
    let rest2 = put(&mut game, player, "Plains", Zone::Library);
    spawn(&mut game, player, observer, Zone::Battlefield);
    let life = game.state.players[player.index()].life;
    start(&mut game, player, spell);
    assert_eq!(
        game.observe(player)
            .library_selection
            .unwrap()
            .candidates
            .len(),
        4
    );
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: outside });
    act(&mut game, Action::SelectLibrary { card: second });
    act(&mut game, Action::SelectLibrary { card: first });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, life + 2);
    let hidden = game.observe(player.other()).face_down_permanents;
    assert_eq!(hidden.len(), 2);
    assert!(
        hidden
            .iter()
            .all(|row| ![first, second].contains(&row.object.card))
    );
    let private = game.observe(player).face_down_permanents;
    assert_eq!(
        private.iter().map(|row| row.underlying).collect::<Vec<_>>(),
        [Card::from_name("Shock"), Some(Card::Island)]
    );
    assert!(
        hidden
            .iter()
            .all(|row| row.cause == FaceDownCause::Cloak && row.underlying.is_none())
    );
    let library = &game.state.players[player.index()].library;
    assert_eq!(library.last(), Some(&outside));
    assert!(library[..2].contains(&rest1) && library[..2].contains(&rest2));
}

#[test]
fn private_cloak_choices_cannot_be_matched_to_a_previously_revealed_library_card() {
    let spell = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &LIBRARY)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Mountain", Zone::Library);
    let alternate = put(&mut game, player, "Plains", Zone::Library);
    let common = put(&mut game, player, "Forest", Zone::Library);
    let revealed = put(&mut game, player, "Shock", Zone::Library);
    let scout = put(&mut game, player, "Cenote Scout", Zone::Hand);
    cast(&mut game, scout, None);
    for _ in 0..4 {
        act(&mut game, Action::Pass);
    }
    let revealed = game.object_ref(revealed);
    act(
        &mut game,
        Action::ChooseExplore {
            object: revealed,
            graveyard: false,
        },
    );
    assert_eq!(
        game.observe(opponent).known_library_top[player.index()]
            .as_ref()
            .unwrap()
            .object,
        revealed
    );

    start(&mut game, player, spell);
    let mut omitted = game.clone();
    for (branch, first) in [(&mut game, revealed.card), (&mut omitted, alternate)] {
        for card in [first, common] {
            act(branch, Action::SelectLibrary { card });
        }
        if branch
            .legal_actions()
            .contains(&Action::FinishLibrarySelection)
        {
            act(branch, Action::FinishLibrarySelection);
        }
        settle(branch);
    }

    let public = |game: &Game| {
        let mut view = game.observe(opponent);
        view.revision = 0;
        assert!(view.known_library_top[player.index()].is_none());
        assert!(
            view.face_down_permanents
                .iter()
                .all(|row| { row.underlying.is_none() && row.object.card != revealed.card })
        );
        serde_json::to_value(view).unwrap()
    };
    assert_eq!(public(&game), public(&omitted));
    assert_eq!(
        game.observe(player).face_down_permanents[0].underlying,
        Card::from_name("Shock")
    );
    assert_eq!(
        omitted.observe(player).face_down_permanents[0].underlying,
        Some(Card::Plains)
    );
    assert!(omitted.state.players[player.index()].library[..2].contains(&revealed.card));
}

#[test]
fn a_skipped_library_cloak_restores_the_exact_original_library_slot() {
    let player = Player::P0;
    let mut game = scenario(player);
    let original = put(&mut game, player, "Island", Zone::Library);
    let top = put(&mut game, player, "Plains", Zone::Library);
    let library = game.state.players[player.index()].library.clone();
    let exact = game.object_ref(original);
    let mut entry = game.hidden_cloak_entry(exact, player).unwrap();
    assert_ne!(entry.before.id, original);
    assert_eq!(
        game.state.players[player.index()].library.last(),
        Some(&top)
    );
    entry.skipped = true;
    game.stage_entries(vec![entry], None);
    assert!(!game.advance_entries());
    assert_eq!(game.state.players[player.index()].library, library);
    assert_eq!(game.object_ref(original), exact);
    assert_eq!(game.object_view(original).card, Card::Island);
    assert!(game.observe(player).face_down_permanents.is_empty());
}
