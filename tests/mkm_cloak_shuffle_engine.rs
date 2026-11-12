//! Concealed cohorts preserve actual moves and hide their old public identities.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const DISGUISE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(ManaCost {
            generic: 2,
            ..ManaCost::ZERO
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const CLOAK_PILE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ExileFaceUpDisguiseCreatures,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 0,
                optional_skip: None,
            },
            Instruction::CloakShuffledRememberedExile,
        ],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn open_pile_choice(game: &mut Game, player: Player) {
    let spell = put(game, player, "Opt", Zone::Hand);
    cast(game, spell, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.pending, Pending::EffectChoice { player });
}

fn select(object: ObjectRef) -> Action {
    Action::ChooseEffectObject { object }
}

struct ResetPrevention;
impl Drop for ResetPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(None));
    }
}

#[test]
fn a_shuffled_pile_cloaks_only_actual_cards_and_hides_the_correspondence() {
    let _rules = fixture_rules::activate(&[
        (named("Opt"), &CLOAK_PILE),
        (Card::GrizzlyBears, &DISGUISE),
        (named("Bear Cub"), &DISGUISE),
    ]);
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let copy = put(&mut game, other, "Forest", Zone::Battlefield);
    let held = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let blanked = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opponent = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
    let ineligible = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let copy_object = game.object_ref(copy);
    game.record_control(copy_object, player, None);
    game.apply_temporary_copy(
        copy_object,
        CopyRecipe::new(Card::GrizzlyBears),
        CopyDeadline::EndOfTurn,
    );
    let token = CardId::try_from(game.state.objects.len()).unwrap();
    game.stage_copy_creation(player, CopyRecipe::new(Card::GrizzlyBears), 1);
    assert!(!game.advance_replacements(false));
    assert!(!game.advance_entries());
    assert!(game.object_view(token).is_token());
    let blank = put(&mut game, player, "Witness Protection", Zone::Hand);
    cast(&mut game, blank, Some(Target::Card(blanked)));
    settle(&mut game);
    game.cloak_library_top(player, player);
    assert!(!game.advance_entries());
    let already_cloaked = game.observe(player).face_down_permanents[0].object;

    open_pile_choice(&mut game, player);
    for id in [blanked, opponent, ineligible, already_cloaked.card] {
        let object = game.object_ref(id);
        reject(&mut game, select(object));
    }
    let mut empty = game.clone();
    act(&mut empty, Action::FinishEffectChoice);
    assert_eq!(empty.observe(player).face_down_permanents.len(), 1);
    assert_eq!(empty.object_view(bear).zone, Zone::Battlefield);

    for id in [held, token, copy, bear] {
        let object = game.object_ref(id);
        act(&mut game, select(object));
        reject(&mut game, select(object));
    }
    let prevented = game.object_ref(held);
    let _reset = ResetPrevention;
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(Some(prevented)));
    let checkpoint = game.snapshot();
    let revision = game.revision();
    let finish = Action::FinishEffectChoice;
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == finish)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, finish.clone());
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_ref(held), prevented);
    assert_eq!(game.object_view(held).zone, Zone::Battlefield);
    assert_eq!(game.object_view(bear).zone, Zone::Outside);
    assert_eq!(game.object_view(copy).zone, Zone::Outside);
    assert_eq!(game.object_view(token).zone, Zone::Outside);

    let public = game.observe(other);
    let private = game.observe(player);
    assert_eq!(public.face_down_permanents.len(), 3);
    assert!(
        public
            .face_down_permanents
            .iter()
            .all(|row| row.underlying.is_none())
    );
    let mut printed = Vec::new();
    for row in private.face_down_permanents {
        if row.object == already_cloaked {
            continue;
        }
        assert_eq!(row.cause, FaceDownCause::Cloak);
        assert!(![bear, copy, held, token].contains(&row.object.card));
        assert_eq!(row.object.generation, 1);
        let object = game.object_view(row.object.card);
        assert_eq!(object.card, named("@face-down/disguise"));
        assert_eq!((object.power, object.toughness), (2, 2));
        assert_eq!(object.controller, player);
        let card = row.underlying.unwrap();
        assert_eq!(
            object.owner,
            if card == Card::Forest { other } else { player }
        );
        printed.push(card);
    }
    assert_eq!(printed.len(), 2);
    assert!(printed.contains(&Card::GrizzlyBears));
    assert!(printed.contains(&Card::Forest));

    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, finish);
    assert_eq!(indexed.snapshot(), game.snapshot());
}

#[test]
fn a_prevented_cloak_restores_the_exact_face_down_exile_card() {
    let _rules =
        fixture_rules::activate(&[(named("Opt"), &CLOAK_PILE), (Card::GrizzlyBears, &DISGUISE)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let before = game.object_ref(bear);
    open_pile_choice(&mut game, player);
    act(&mut game, select(before));
    let temporary = ObjectRef {
        card: CardId::try_from(game.state.objects.len()).unwrap(),
        generation: 0,
    };
    let _reset = ResetPrevention;
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(Some(temporary)));
    act(&mut game, Action::FinishEffectChoice);
    let exiled = game.object_ref(bear);
    assert_eq!(exiled.generation, before.generation + 1);
    assert_eq!(game.object_view(bear).zone, Zone::Exile);
    assert!(game.exiled_face_down(exiled));
    assert_eq!(game.object_view(temporary.card).zone, Zone::Outside);
    for viewer in [player, player.other()] {
        let view = game.observe(viewer);
        assert!(view.face_down_permanents.is_empty());
        assert!(
            view.objects
                .iter()
                .all(|object| object.id != bear && object.id != temporary.card)
        );
    }
}

#[test]
fn a_hidden_hand_card_enters_with_a_fresh_public_handle() {
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let first = put(&mut game, player, "Opt", Zone::Hand);
    let second = put(&mut game, player, "Shock", Zone::Hand);
    let first = game.object_ref(first);
    let second = game.object_ref(second);
    game.remember_public_hand(first);
    game.remember_public_hand(second);
    assert_eq!(game.observe(other).objects.len(), 2);

    game.obscure_private_hand_selection(player);
    game.stage_hidden_cloaks(player, [second]);
    assert!(!game.advance_entries());
    let public = game.observe(other);
    assert_eq!(public.objects.len(), 1);
    let row = public.face_down_permanents[0];
    assert!(![first.card, second.card].contains(&row.object.card));
    assert!(row.underlying.is_none());
    assert_eq!(
        game.observe(player).face_down_permanents[0].underlying,
        Some(named("Shock"))
    );
    assert_eq!(game.object_view(first.card).zone, Zone::Hand);
}
