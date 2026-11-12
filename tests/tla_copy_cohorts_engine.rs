//! Copiable type exceptions and simultaneous copies of successful graveyard exiles.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const VISIONS: CardRules = CardRules {
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
            max: usize::MAX,
        }),
        &[
            Instruction::TargetsExactlyX,
            Instruction::ExileGraveyardTargetsCopyUntilOwnEndStep,
        ],
    ),
    extras: &CardExtras {
        waterbend_additional: Some(WaterbendAdditional::RequiredX),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const EXILE_PROHIBITION: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::GraveyardCardsCannotBeExiled],
    }],
    ..CardRules::EMPTY
};

const MODES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Copy a controlled creature",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
            program: &[Instruction::CreateCopyTokenWithExceptions {
                subject: Subject::Targets,
                amount: 1,
                exceptions: CopyExceptions {
                    base: Some((4, 4)),
                    add_hero_subtype: true,
                    remove_legendary: true,
                    ..CopyExceptions::NONE
                },
            }],
        },
        SpellMode {
            text: "Copy an opposing creature",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::CreateCopyTokenWithExceptions {
                subject: Subject::Targets,
                amount: 1,
                exceptions: CopyExceptions {
                    base: Some((2, 2)),
                    add_coward_subtype: true,
                    remove_legendary: true,
                    ..CopyExceptions::NONE
                },
            }],
        },
    ]),
    ..CardRules::EMPTY
};

fn target(id: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(id),
    }
}

fn tokens(game: &Game) -> Vec<ObjectView> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| object.zone == Zone::Battlefield && object.is_token())
        .map(|(id, _)| game.object_view(id as CardId))
        .collect()
}

#[test]
fn the_nonlegendary_base_and_added_subtype_are_copiable_values() {
    let spell = Card::from_name("Quick Study").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &MODES)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = spawn(&mut game, player, spell, Zone::Hand);
        let own = put(
            &mut game,
            player,
            "Malcolm, Alluring Scoundrel",
            Zone::Battlefield,
        );
        let opposing = put(
            &mut game,
            player.other(),
            "Malcolm, Alluring Scoundrel",
            Zone::Battlefield,
        );
        for (mode, subject, rejected, size, subtype, other_type) in [
            (0, own, opposing, 4, "Hero", "Coward"),
            (1, opposing, own, 2, "Coward", "Hero"),
        ] {
            let mut branch = game.clone();
            act(&mut branch, Action::AnnounceCast { card: spell });
            act(&mut branch, Action::ChooseMode { mode });
            reject(&mut branch, target(rejected));
            act(&mut branch, target(subject));
            pay(&mut branch);
            settle(&mut branch);
            let token = tokens(&branch).into_iter().next().unwrap();
            let recipe = branch.copiable_recipe(token.reference);
            branch.stage_copy_creation(player, recipe, 1);
            branch.state_based_actions();
            let copies = tokens(&branch);
            assert_eq!(copies.len(), 2);
            for copy in copies {
                assert_eq!(copy.controller, player);
                assert_eq!((copy.power, copy.toughness), (size, size));
                assert!(!copy.is_legendary());
                assert!(copy.has_subtype(subtype));
                assert!(!copy.has_subtype(other_type));
                assert!(copy.has_subtype("Siren"));
                assert!(copy.has_subtype("Pirate"));
                assert!(copy.keywords.contains(Keywords::FLYING));
            }
            assert!(branch.object_view(own).is_legendary());
        }
    }
}

fn start_end_step(game: &mut Game, player: Player) {
    game.state.active = player;
    game.begin_step(Step::End);
    game.give_priority();
    game.prepare_triggers();
    settle(game);
}

#[test]
fn only_actual_exiled_targets_form_one_delayed_copy_cohort() {
    let spell = Card::from_name("Divination").unwrap();
    let prohibition = Card::from_name("Short Bow").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &VISIONS), (prohibition, &EXILE_PROHIBITION)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let cast = spawn(&mut game, player, spell, Zone::Hand);
        let bear = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        let giant = put(&mut game, player.other(), "Hill Giant", Zone::Graveyard);
        put(&mut game, player, "Doubling Season", Zone::Battlefield);
        act(&mut game, Action::AnnounceCast { card: cast });
        reject(&mut game, Action::ChooseX { value: 3 });

        let mut zero = game.clone();
        act(&mut zero, Action::ChooseX { value: 0 });
        pay(&mut zero);
        settle(&mut zero);
        assert!(tokens(&zero).is_empty());
        assert!(zero.observe(player).delayed_sacrifices.is_empty());

        act(&mut game, Action::ChooseX { value: 2 });
        act(&mut game, target(bear));
        reject(&mut game, target(bear));
        reject(&mut game, Action::FinishTargets);
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        let action = target(giant);
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut game, action);
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert!(indexed.legal_actions().contains(&target(giant)));
        pay(&mut game);

        let mut stale = game.clone();
        stale.move_object(bear, Zone::Exile);
        stale.move_object(bear, Zone::Graveyard);
        settle(&mut stale);
        assert_eq!(stale.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(stale.object_view(giant).zone, Zone::Exile);
        assert!(
            tokens(&stale)
                .iter()
                .all(|copy| copy.card == Card::HillGiant)
        );
        assert_eq!(tokens(&stale).len(), 2);

        let mut prohibited = game.clone();
        spawn(
            &mut prohibited,
            player.other(),
            prohibition,
            Zone::Battlefield,
        );
        settle(&mut prohibited);
        assert_eq!(prohibited.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(prohibited.object_view(giant).zone, Zone::Graveyard);
        assert!(tokens(&prohibited).is_empty());

        settle(&mut game);
        let cohort = game.observe(player).delayed_sacrifices;
        assert_eq!(cohort.len(), 1);
        assert!(cohort[0].own_end_step);
        assert_eq!(cohort[0].tokens.len(), 4);
        let copies = tokens(&game);
        assert_eq!(copies.len(), 4);
        assert!(copies.iter().all(|copy| !copy.tapped));
        let controlled_by_opponent = copies[0].reference;
        game.change_controller(controlled_by_opponent, player.other());
        let unrelated = CopyRecipe::new(Card::HillGiant);
        game.stage_copy_creation(player, unrelated, 1);
        game.advance_replacements(true);
        assert_eq!(tokens(&game).len(), 6);
        start_end_step(&mut game, player.other());
        assert_eq!(tokens(&game).len(), 6);
        start_end_step(&mut game, player);
        let remaining = tokens(&game);
        assert_eq!(remaining.len(), 3);
        assert!(
            remaining
                .iter()
                .any(|copy| copy.reference == controlled_by_opponent)
        );
        assert!(game.observe(player).delayed_sacrifices.is_empty());
    }
}
