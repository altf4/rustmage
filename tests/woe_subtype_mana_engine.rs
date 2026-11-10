//! A subtype grant follows effective characteristics in the layer-six and funding worlds.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const FOOD: Card = Card::from_catalogue_id(5246);

const FOOD_GRANT: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledSubtypeTapMana {
            subtype: "Food",
            production: ManaProduction::Fixed(GREEN),
        }],
    }],
    ..CardRules::EMPTY
};
const FOOD_CREATURE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    ..CardRules::EMPTY
};
const BLANK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::RemoveTargetAbilitiesUntilEndOfTurn],
    ),
    ..CardRules::EMPTY
};

fn grantor_card() -> Card {
    Card::from_name("A Realm Reborn").unwrap()
}

fn tap_food(grantor: ObjectRef, card: CardId) -> Action {
    Action::TapGrantedMana {
        card,
        grantor,
        production: GREEN,
    }
}

#[test]
fn gained_food_subtype_supplies_mana_during_a_paid_announcement() {
    let _rules = fixture_rules::activate(&[(grantor_card(), &FOOD_GRANT)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let grantor = put(&mut game, player, "A Realm Reborn", Zone::Hand);
        let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
        let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let opposing_food = spawn(&mut game, player.other(), FOOD, Zone::Battlefield);
        let ragost = put(&mut game, player, "Ragost, Deft Gastronaut", Zone::Hand);
        let elves = put(&mut game, player, "Llanowar Elves", Zone::Hand);
        game.state.objects[boots as usize].summoning_sick = true;

        cast(&mut game, grantor, None);
        settle(&mut game);
        let grantor = game.object_ref(grantor);
        game.state.players[player.index()].mana = [0; 6];
        for card in [boots, bear, opposing_food, grantor.card] {
            reject(&mut game, tap_food(grantor, card));
        }
        let announce = Action::AnnounceCast { card: elves };
        reject(&mut game, announce.clone());

        game.state.players[player.index()].mana = [8; 6];
        cast(&mut game, ragost, None);
        settle(&mut game);
        assert!(game.has_subtype(boots, "Food"));
        assert!(!game.has_subtype(bear, "Food"));

        // The printed equipment acquires the live subtype from Ragost. It
        // loses the granted mana route as soon as that type effect ends.
        let mut without_subtype = game.clone();
        let murder = put(&mut without_subtype, player.other(), "Murder", Zone::Hand);
        act(&mut without_subtype, Action::Pass);
        cast(&mut without_subtype, murder, Some(Target::Card(ragost)));
        settle(&mut without_subtype);
        assert!(!without_subtype.has_subtype(boots, "Food"));
        without_subtype.state.players[player.index()].mana = [0; 6];
        reject(&mut without_subtype, tap_food(grantor, boots));
        reject(&mut without_subtype, announce.clone());

        game.state.players[player.index()].mana = [0; 6];
        act(&mut game, announce);
        reject(&mut game, tap_food(grantor, bear));
        let tap = tap_food(grantor, boots);
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == tap)
            .unwrap();
        let mut indexed = game.clone();
        act(&mut game, tap);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert_eq!(game.state.players[player.index()].mana, GREEN);
        pay(&mut game);
        settle(&mut game);
        assert!(game.object_view(boots).tapped);
        assert_eq!(game.object_view(elves).zone, Zone::Battlefield);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}

#[test]
fn food_mana_respects_recipient_ability_order_sickness_and_control() {
    let cookie = Card::from_name("Tough Cookie").unwrap();
    let _rules = fixture_rules::activate(&[
        (grantor_card(), &FOOD_GRANT),
        (cookie, &FOOD_CREATURE),
        (Card::from_name("Opt").unwrap(), &BLANK),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let first_grant = put(&mut game, player, "A Realm Reborn", Zone::Hand);
    let later_grant = put(&mut game, player, "A Realm Reborn", Zone::Hand);
    let ready = spawn(&mut game, player, cookie, Zone::Battlefield);
    let sick = spawn(&mut game, player, cookie, Zone::Battlefield);
    let borrowed = spawn(&mut game, player.other(), cookie, Zone::Battlefield);
    let blank = put(&mut game, player, "Opt", Zone::Hand);
    let theft = put(&mut game, player, "Goatnap", Zone::Hand);
    let elves = put(&mut game, player, "Llanowar Elves", Zone::Hand);
    game.state.objects[sick as usize].summoning_sick = true;

    cast(&mut game, first_grant, None);
    settle(&mut game);
    let first_grant = game.object_ref(first_grant);
    assert!(game.legal_actions().contains(&tap_food(first_grant, ready)));
    reject(&mut game, tap_food(first_grant, sick));
    reject(&mut game, tap_food(first_grant, borrowed));

    cast(&mut game, blank, Some(Target::Card(ready)));
    settle(&mut game);
    assert!(game.has_subtype(ready, "Food"));
    reject(&mut game, tap_food(first_grant, ready));
    let mut without_mana = game.clone();
    without_mana.state.players[player.index()].mana = [0; 6];
    reject(&mut without_mana, Action::AnnounceCast { card: elves });

    // A later grant applies after the blanking effect, with a distinct origin.
    cast(&mut game, later_grant, None);
    settle(&mut game);
    reject(&mut game, tap_food(first_grant, ready));
    let later_grant = game.object_ref(later_grant);
    assert!(game.legal_actions().contains(&tap_food(later_grant, ready)));

    // A Food the opponent owns can tap if you control it and its tap cost is legal.
    cast(&mut game, theft, Some(Target::Card(borrowed)));
    settle(&mut game);
    assert_eq!(game.object_view(borrowed).owner, player.other());
    assert_eq!(game.object_view(borrowed).controller, player);
    let prior_green = game.state.players[player.index()].mana[4];
    act(&mut game, tap_food(first_grant, borrowed));
    assert_eq!(game.state.players[player.index()].mana[4], prior_green + 1);
    assert!(game.object_view(borrowed).tapped);
    reject(&mut game, tap_food(later_grant, borrowed));
}
