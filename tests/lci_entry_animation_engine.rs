//! Entry form and counters share a transaction, while the returned card's
//! copiable values retain its ordinary physical definition.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const fn own_graveyard(kinds: u8) -> TargetRule {
    TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }
}

const RETURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Union(&[
            own_graveyard(16),
            TargetRule::Filtered {
                base: &own_graveyard(32),
                predicate: &TargetPredicate::WithoutSubtype("Aura"),
            },
        ]))),
        &[Instruction::ReturnGraveyardTargetsAnimated {
            animation: &AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Additional(&["Spirit"]),
                base: Some((1, 1)),
                keywords: Keywords::FLYING,
                duration: AnimationDuration::Permanent,
            },
            x_counters: Some(CounterKind::PlusOne),
        }],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn start_cast(game: &mut Game, player: Player, x: u32) {
    let spell = put(game, player, "Opt", Zone::Hand);
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseX { value: x });
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

#[test]
fn animated_equipment_enters_as_a_creature_with_replaced_counters_and_original_copiable_values() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &RETURN)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let opponent = player.other();
        let equipment = put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
        let aura = put(&mut game, player, "Pacifism", Zone::Graveyard);
        let foreign = put(&mut game, opponent, "Swiftfoot Boots", Zone::Graveyard);
        put(
            &mut game,
            opponent,
            "Authority of the Consuls",
            Zone::Battlefield,
        );
        put(&mut game, player, "Doubling Season", Zone::Battlefield);

        start_cast(&mut game, player, 2);
        reject(&mut game, target(aura));
        reject(&mut game, target(foreign));
        act(&mut game, target(equipment));
        pay(&mut game);
        let checkpoint = game.snapshot();
        let revision = game.revision();
        let actor = game.actor().unwrap();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == Action::Pass)
            .unwrap();
        let mut indexed = game.clone();
        let mut stale = game.clone();
        act(&mut game, Action::Pass);
        indexed.apply_index(actor, revision, index).unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());

        let original = game.object_ref(equipment);
        settle(&mut game);
        settle(&mut indexed);
        assert_eq!(game.snapshot(), indexed.snapshot());
        let entered = game.object_view(equipment);
        assert_ne!(entered.reference, original);
        assert_eq!(entered.zone, Zone::Battlefield);
        assert!(entered.tapped);
        assert_eq!(entered.kinds, 1 | 16);
        assert!(entered.has_subtype("Equipment"));
        assert!(entered.has_subtype("Spirit"));
        assert!(entered.keywords.contains(Keywords::FLYING));
        assert_eq!(entered.counters.get(CounterKind::PlusOne), 4);
        assert_eq!((entered.power, entered.toughness), (5, 5));
        assert_eq!(game.state.players[opponent.index()].life, STARTING_LIFE + 1);
        let recipe = game.copiable_recipe(entered.reference);
        assert_eq!(recipe.kinds(), 16);
        assert_eq!(
            recipe.definition,
            Card::from_name("Swiftfoot Boots").unwrap()
        );

        indexed.restore(&checkpoint).unwrap();
        settle(&mut indexed);
        assert_eq!(game.snapshot(), indexed.snapshot());
        stale.move_object(equipment, Zone::Exile);
        stale.move_object(equipment, Zone::Graveyard);
        settle(&mut stale);
        assert_eq!(stale.object_view(equipment).zone, Zone::Graveyard);

        game.move_object(equipment, Zone::Graveyard);
        game.move_object(equipment, Zone::Battlefield);
        settle(&mut game);
        let returned = game.object_view(equipment);
        assert_eq!(returned.kinds, 16);
        assert!(!returned.has_subtype("Spirit"));
        assert_eq!(returned.counters.get(CounterKind::PlusOne), 0);
    }
}

#[test]
fn zero_x_still_adds_the_entry_form_to_an_enchantment() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &RETURN)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let enchantment = put(&mut game, player, "Impact Tremors", Zone::Graveyard);
    start_cast(&mut game, player, 0);
    act(&mut game, target(enchantment));
    pay(&mut game);
    settle(&mut game);

    let entered = game.object_view(enchantment);
    assert_eq!(entered.kinds, 1 | 32);
    assert_eq!((entered.power, entered.toughness), (1, 1));
    assert!(entered.has_subtype("Spirit"));
    assert!(entered.keywords.contains(Keywords::FLYING));
    assert_eq!(entered.counters.get(CounterKind::PlusOne), 0);
    assert_eq!(
        game.state.players[player.other().index()].life,
        STARTING_LIFE - 1
    );
}
