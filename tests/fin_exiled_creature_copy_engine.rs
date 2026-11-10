//! Copy only the exact creature card whose exile actually succeeded.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        &[Instruction::ExileAndCopyCreatureAs {
            base: (5, 5),
            color: ChoiceColor::Black,
            subtype: "Demon",
        }],
    ),
    ..CardRules::EMPTY
};

struct PreventMove;

impl Drop for PreventMove {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
    }
}

fn copy_fixture() -> (Game, CardId, CardId) {
    let player = Player::P0;
    let mut game = scenario(player);
    let original = put(&mut game, player.other(), "Meteor Golem", Zone::Graveyard);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    (game, original, spell)
}

fn check_form(game: &Game, copy: CardId) {
    let view = game.object_view(copy);
    assert!(view.is_token());
    assert_eq!(view.name, "Meteor Golem");
    assert_eq!(view.kinds, 1);
    assert_eq!(view.colors, 1 << 2);
    assert!(view.has_subtype("Demon"));
    assert!(!view.has_subtype("Golem"));
    assert_eq!(stats(game, copy), (5, 5));
}

#[test]
fn successful_exile_creates_a_copiable_replacement_form_with_its_printed_trigger() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &COPY)]);
    let player = Player::P0;
    let opponent = player.other();
    let (mut game, original, spell) = copy_fixture();
    let victim = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let later_victim = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let first_copy = game.state.objects.len() as CardId;

    cast(&mut game, spell, Some(Target::Card(original)));
    act(&mut game, Action::Pass);
    let mut indexed = game.clone();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == Action::Pass)
        .unwrap();
    let revision = game.revision();
    act(&mut game, Action::Pass);
    indexed.apply_index(opponent, revision, index).unwrap();
    assert_eq!(indexed.snapshot(), game.snapshot());

    assert_eq!(game.object_view(original).zone, Zone::Exile);
    assert_eq!(game.object_view(first_copy).controller, player);
    check_form(&game, first_copy);
    assert_eq!(game.object_view(first_copy).card.fixed_mana_value(), 7);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);

    let recipe = game.copiable_recipe(game.object_ref(first_copy));
    let second_copy = game.state.objects.len() as CardId;
    game.stage_copy_creation(opponent, recipe, 1);
    assert!(!game.advance_entries());
    game.prepare_triggers();
    check_form(&game, second_copy);
    assert_eq!(game.actor(), Some(opponent));
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(later_victim),
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(later_victim).zone, Zone::Graveyard);
}

#[test]
fn failed_stale_and_unselected_exile_do_not_create_a_token() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &COPY)]);
    let (mut game, original, spell) = copy_fixture();
    let before = game.state.objects.len();

    let mut skipped = game.clone();
    cast(&mut skipped, spell, None);
    settle(&mut skipped);
    assert_eq!(skipped.state.objects.len(), before);
    assert_eq!(skipped.object_view(original).zone, Zone::Graveyard);

    cast(&mut game, spell, Some(Target::Card(original)));
    let mut stale = game.clone();
    stale.move_object(original, Zone::Exile);
    stale.move_object(original, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.state.objects.len(), before);
    assert_eq!(stale.object_view(original).zone, Zone::Graveyard);

    let _prevention = PreventMove;
    zone_replacements::TEST_PREVENT_ZONE_MOVE
        .with(|rule| rule.set(Some(game.object_ref(original))));
    settle(&mut game);
    assert_eq!(game.state.objects.len(), before);
    assert_eq!(game.object_view(original).zone, Zone::Graveyard);
}
