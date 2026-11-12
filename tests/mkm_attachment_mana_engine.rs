//! Attached land mana triggers preserve color choices in execution and funding.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::{cards::*, engine::*};
use std::collections::{BTreeSet, HashSet};
use support::*;

const WHITE: Mana = [1, 0, 0, 0, 0, 0];
const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const BLACK: Mana = [0, 0, 1, 0, 0, 0];
const RED: Mana = [0, 0, 0, 1, 0, 0];
const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];

const AURA: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 2,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    })),
    aura: Some(AuraSpec {
        kinds: 2,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttachmentManaBonus(
            ManaProduction::any_color(1),
        )],
    }],
    ..CardRules::EMPTY
};

const DRAW: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};

fn attach(game: &mut Game, controller: Player, land: CardId) -> CardId {
    let aura = put(game, controller, "Pacifism", Zone::Hand);
    cast(game, aura, Some(Target::Card(land)));
    settle(game);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(land))
    );
    aura
}

fn mana_choices(game: &Game) -> BTreeSet<Mana> {
    game.legal_actions()
        .into_iter()
        .filter_map(|action| match action {
            Action::ChooseMana { production } => Some(production),
            _ => None,
        })
        .collect()
}

#[test]
fn the_enchanted_lands_controller_chooses_any_color_and_keeps_the_base_output() {
    let aura_card = Card::from_name("Pacifism").unwrap();
    let _rules = fixture_rules::activate(&[(aura_card, &AURA)]);
    for land_controller in [Player::P0, Player::P1] {
        let aura_controller = land_controller.other();
        let mut game = scenario(aura_controller);
        let land = put(&mut game, land_controller, "Forest", Zone::Battlefield);
        let aura = attach(&mut game, aura_controller, land);
        game.state.priority = land_controller;
        game.state.players[land_controller.index()].mana = [0; 6];
        game.state.players[aura_controller.index()].mana = [0; 6];

        act(&mut game, Action::TapMana { card: land });
        assert_eq!(game.actor(), Some(land_controller));
        assert_eq!(game.state.players[land_controller.index()].mana, GREEN);
        assert_eq!(
            game.observe(land_controller).mana_tap_bonus_source,
            Some(game.object_ref(aura))
        );
        assert_eq!(
            mana_choices(&game),
            BTreeSet::from([WHITE, BLUE, BLACK, RED, GREEN])
        );
        reject(
            &mut game,
            Action::ChooseMana {
                production: COLORLESS,
            },
        );

        let action = Action::ChooseMana { production: BLUE };
        let index = game
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .unwrap();
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        indexed
            .apply_index(land_controller, game.revision(), index)
            .unwrap();
        act(&mut game, action);
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert_eq!(
            game.state.players[land_controller.index()].mana,
            [0, 1, 0, 0, 1, 0]
        );
        assert_eq!(game.state.players[aura_controller.index()].mana, [0; 6]);
        assert_eq!(game.state.pending, Pending::Priority);
        assert!(game.state.stack.is_empty());

        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, Action::ChooseMana { production: RED });
        assert_eq!(
            indexed.state.players[land_controller.index()].mana,
            [0, 0, 0, 1, 1, 0]
        );
    }
}

#[test]
fn two_attached_choices_fund_distinct_colored_pips_and_resume_the_enclosing_cast() {
    let aura_card = Card::from_name("Pacifism").unwrap();
    let spell_card = Card::from_name("No More Lies").unwrap(); // {W}{U}
    let _rules = fixture_rules::activate(&[(aura_card, &AURA), (spell_card, &DRAW)]);
    let payer = Player::P0;
    let mut game = scenario(payer.other());
    let land = put(&mut game, payer, "Forest", Zone::Battlefield);
    let first = attach(&mut game, payer.other(), land);
    game.state.active = payer;
    game.state.priority = payer;
    game.state.players[payer.index()].mana = [0; 6];
    let spell = spawn(&mut game, payer, spell_card, Zone::Hand);
    let goal = FundingGoal {
        actor: payer,
        required: [1, 1, 0, 0, 0, 0],
        generic: 0,
        source: None,
        reserve: None,
    };
    let one = FundingState::from_game(&game, &goal);
    assert!(!one.reachable(&goal, &mut HashSet::new()));
    reject(&mut game, Action::AnnounceCast { card: spell });

    let second = put(&mut game, payer.other(), "Pacifism", Zone::Battlefield);
    game.state.objects[second as usize].attached_to = Some(game.object_ref(land));
    let both = FundingState::from_game(&game, &goal);
    assert!(both.reachable(&goal, &mut HashSet::new()));
    let tap = both
        .moves(&goal)
        .into_iter()
        .find(|step| both.resources[step.resource].id == land)
        .unwrap();
    let mut projected = BTreeSet::new();
    both.visit_applied(&tap, &mut |next| {
        projected.insert(next.mana);
        false
    });
    assert_eq!(projected.len(), 15);
    assert!(projected.contains(&[1, 1, 0, 0, 1, 0]));
    assert!(projected.iter().all(|mana| mana[5] == 0));

    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::TapMana { card: land });
    assert_eq!(
        game.observe(payer).mana_tap_bonus_source,
        Some(game.object_ref(first))
    );
    assert_eq!(mana_choices(&game), BTreeSet::from([WHITE, BLUE]));
    reject(&mut game, Action::ChooseMana { production: RED });
    act(&mut game, Action::ChooseMana { production: BLUE });
    assert_eq!(
        game.observe(payer).mana_tap_bonus_source,
        Some(game.object_ref(second))
    );
    assert_eq!(mana_choices(&game), BTreeSet::from([WHITE]));
    act(&mut game, Action::ChooseMana { production: WHITE });
    assert!(matches!(game.state.pending, Pending::PayCosts { .. }));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(payer), 1);
    assert_eq!(game.state.players[payer.index()].mana, GREEN);
}

#[test]
fn the_bonus_requires_the_exact_affected_land_incarnation() {
    let aura_card = Card::from_name("Pacifism").unwrap();
    let _rules = fixture_rules::activate(&[(aura_card, &AURA)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    attach(&mut game, player, land);
    let enchanted = game.object_ref(land);
    game.move_object(land, Zone::Exile);
    game.move_object(land, Zone::Battlefield);
    assert_ne!(game.object_ref(land), enchanted);
    game.state.players[player.index()].mana = [0; 6];
    act(&mut game, Action::TapMana { card: land });
    assert_eq!(game.state.pending, Pending::Priority);
    assert_eq!(game.state.players[player.index()].mana, GREEN);
    assert!(game.observe(player).mana_tap_bonus_source.is_none());
}
