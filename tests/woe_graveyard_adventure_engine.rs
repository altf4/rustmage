//! Exact Adventure-only permission follows the recipient's next actual turn.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const TRAIN_TROOPS: Card = Card::from_catalogue_id(5199);
const ADVENTURER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantDepartedAdventureUntilNextTurn],
    }],
    extras: &CardExtras {
        adventure_identity: Some("Imodane's Recruiter // Train Troops"),
        adventure_spell: Some(TRAIN_TROOPS),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const FREE_SPELLS: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::FreeSpellCast(CastFilter::Any)],
    }],
    ..CardRules::EMPTY
};

fn adventurer() -> Card {
    Card::from_name("Imodane's Recruiter").unwrap()
}

fn setup(player: Player) -> (Game, CardId, CardId) {
    let mut game = scenario(player);
    let source = put(&mut game, player, "Imodane's Recruiter", Zone::Battlefield);
    let removal = put(&mut game, player, "Murder", Zone::Hand);
    (game, source, removal)
}

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn expect_permission(game: &Game, player: Player, source: CardId, expires_turn: Option<u32>) {
    let row = GraveyardCastPermissionView {
        object: game.object_ref(source),
        player,
        kind: GraveyardCastKind::Adventure,
        expires_turn,
    };
    for viewer in [player, player.other()] {
        assert_eq!(game.observe(viewer).graveyard_cast_permissions, [row]);
    }
}

fn finish_turn(game: &mut Game) {
    let turn = game.state.turn;
    for _ in 0..48 {
        if game.state.turn != turn {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("turn did not end");
}

fn enter_main(game: &mut Game) {
    for _ in 0..12 {
        if game.state.step == Step::Main1 {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("main phase did not start");
}

#[test]
fn the_grant_binds_the_exact_death_object_and_allows_the_paid_adventure() {
    let _rules = fixture_rules::activate(&[(adventurer(), &ADVENTURER)]);
    for player in [Player::P0, Player::P1] {
        let (mut game, source, removal) = setup(player);
        let announce = Action::AnnounceAdventure { card: source };
        cast(&mut game, removal, Some(Target::Card(source)));
        resolve_top(&mut game);
        assert_eq!(game.object_view(source).zone, Zone::Graveyard);
        assert!(game.observe(player).graveyard_cast_permissions.is_empty());

        // A death trigger on the stack cannot grant a right on a newer
        // graveyard incarnation of the same physical card.
        let mut stale_trigger = game.clone();
        stale_trigger.move_object(source, Zone::Exile);
        stale_trigger.move_object(source, Zone::Graveyard);
        settle(&mut stale_trigger);
        reject(&mut stale_trigger, announce.clone());

        settle(&mut game);
        expect_permission(&game, player, source, None);
        reject(&mut game, Action::AnnounceCast { card: source });

        let mut stale_grant = game.clone();
        stale_grant.move_object(source, Zone::Exile);
        stale_grant.move_object(source, Zone::Graveyard);
        reject(&mut stale_grant, announce.clone());
        assert!(
            stale_grant
                .observe(player)
                .graveyard_cast_permissions
                .is_empty()
        );

        let snapshot = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == announce)
            .unwrap();
        let mut raw = game.clone();
        let mut indexed = game;
        act(&mut raw, announce.clone());
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&snapshot).unwrap();
        expect_permission(&indexed, player, source, None);
        act(&mut indexed, announce.clone());
        assert_eq!(indexed.snapshot(), raw.snapshot());

        // Train Troops requires the full {4}{W} when using the ordinary route.
        reject(
            &mut raw,
            Action::PayCosts {
                payment: [1, 0, 0, 0, 0, 0],
            },
        );
        act(
            &mut raw,
            Action::PayCosts {
                payment: [1, 0, 0, 0, 4, 0],
            },
        );
        let characteristics = raw.spell_characteristics().pop().unwrap();
        assert_eq!(characteristics.face, CastFace::Adventure);
        assert_eq!(characteristics.name, "Train Troops");
        assert_eq!(
            raw.state.stack.last().unwrap().cast.unwrap().origin,
            Zone::Graveyard
        );
        settle(&mut raw);
        assert_eq!(raw.object_view(source).zone, Zone::Exile);
        assert_eq!(
            raw.state
                .objects
                .iter()
                .filter(|object| object.zone == Zone::Battlefield
                    && object.controller == player
                    && object.is_token())
                .count(),
            2,
        );
        assert!(raw.observe(player).graveyard_cast_permissions.is_empty());
        reject(&mut raw, announce);

        // Resolving the Adventure established the usual normal-only exile right.
        act(&mut raw, Action::AnnounceExile { card: source });
        pay(&mut raw);
        settle(&mut raw);
        assert_eq!(raw.object_view(source).zone, Zone::Battlefield);
    }
}

#[test]
fn the_deadline_follows_extra_and_skipped_turns_of_both_players() {
    let _rules = fixture_rules::activate(&[(adventurer(), &ADVENTURER)]);
    let player = Player::P1;
    let (mut ordinary, source, removal) = setup(player);
    let announce = Action::AnnounceAdventure { card: source };
    cast(&mut ordinary, removal, Some(Target::Card(source)));
    settle(&mut ordinary);

    let mut extra = ordinary.clone();
    extra.add_extra_turn(player, false);
    finish_turn(&mut extra);
    assert_eq!(extra.state.active, player);
    expect_permission(&extra, player, source, Some(extra.state.turn));
    enter_main(&mut extra);
    extra.state.players[player.index()].mana = [8; 6];
    assert!(extra.legal_actions().contains(&announce));
    finish_turn(&mut extra);
    assert!(extra.observe(player).graveyard_cast_permissions.is_empty());

    // Wait through an opponent's ordinary and extra turns, then a skipped
    // recipient turn. Only the recipient's next turn that actually starts counts.
    finish_turn(&mut ordinary);
    assert_eq!(ordinary.state.active, player.other());
    expect_permission(&ordinary, player, source, None);
    enter_main(&mut ordinary);
    ordinary.state.players[player.index()].mana = [8; 6];
    act(&mut ordinary, Action::Pass);
    assert_eq!(ordinary.actor(), Some(player));
    reject(&mut ordinary, announce.clone());
    ordinary.add_extra_turn(player.other(), false);
    let mut skips = Box::new([0; 2]);
    skips[player.index()] = 1;
    ordinary.state.turn_skips = Some(skips);

    for _ in 0..2 {
        finish_turn(&mut ordinary);
        assert_eq!(ordinary.state.active, player.other());
        expect_permission(&ordinary, player, source, None);
    }
    finish_turn(&mut ordinary);
    assert_eq!(ordinary.state.active, player);
    expect_permission(&ordinary, player, source, Some(ordinary.state.turn));
    enter_main(&mut ordinary);
    ordinary.state.players[player.index()].mana = [8; 6];
    assert!(ordinary.legal_actions().contains(&announce));
    finish_turn(&mut ordinary);
    assert!(
        ordinary
            .observe(player)
            .graveyard_cast_permissions
            .is_empty()
    );
}

#[test]
fn an_alternative_cost_combines_only_with_the_permitted_face() {
    let _rules = fixture_rules::activate(&[
        (adventurer(), &ADVENTURER),
        (Card::from_name("Omniscience").unwrap(), &FREE_SPELLS),
    ]);
    let player = Player::P0;
    let (mut game, source, removal) = setup(player);
    put(&mut game, player, "Omniscience", Zone::Battlefield);
    let counter = put(&mut game, player.other(), "Cancel", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(source)));
    settle(&mut game);
    game.state.players[player.index()].mana = [0; 6];
    let free = |face| Action::AnnounceFree {
        card: source,
        face,
        flash_surcharge: false,
    };
    reject(&mut game, free(CastFace::Normal));
    reject(&mut game, Action::AnnounceAdventure { card: source });
    act(&mut game, free(CastFace::Adventure));
    act(&mut game, Action::PayCosts { payment: [0; 6] });
    act(&mut game, Action::Pass);
    cast(&mut game, counter, Some(Target::Spell(source)));
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert!(game.observe(player).graveyard_cast_permissions.is_empty());
    reject(&mut game, free(CastFace::Adventure));
}

#[test]
fn a_stolen_adventurer_does_not_grant_a_right_to_either_player() {
    let _rules = fixture_rules::activate(&[(adventurer(), &ADVENTURER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player.other(),
        "Imodane's Recruiter",
        Zone::Battlefield,
    );
    let theft = put(&mut game, player, "Goatnap", Zone::Hand);
    let removal = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, theft, Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(game.object_view(source).controller, player);
    cast(&mut game, removal, Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(game.object_view(source).owner, player.other());
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    for viewer in [player, player.other()] {
        assert!(game.observe(viewer).graveyard_cast_permissions.is_empty());
    }
}
