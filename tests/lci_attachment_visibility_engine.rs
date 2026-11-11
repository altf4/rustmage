//! A library-top look belongs to the exact attached ability's recipient.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const LANTERN: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentLookAtLibraryTop,
            StaticAbilityFragment::AttachmentTrigger(0),
        ],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::GrantedAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::Explore(Subject::Source)],
    }],
    ..CardRules::EMPTY
};
const CAST_TOP: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CastFromLibraryTop(
            CastFilter::InstantSorcery,
        )],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn visible(game: &Game, viewer: Player, owner: Player) -> Option<ObjectRef> {
    game.observe(viewer).known_library_top[owner.index()].map(|top| top.object)
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn only_the_live_bearers_controller_gets_the_private_look() {
    let _rules = fixture_rules::activate(&[(named("Short Bow"), &LANTERN)]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let own_top = put(&mut game, player, "Shock", Zone::Library);
        let foreign_top = put(&mut game, other, "Opt", Zone::Library);
        let lantern = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let spare = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
        let provider = game.object_ref(lantern);
        let bearer = game.object_ref(creature);
        assert!(visible(&game, player, player).is_none());
        assert!(game.attach_equipment(provider, bearer));
        assert_eq!(
            visible(&game, player, player),
            Some(game.object_ref(own_top))
        );
        assert!(visible(&game, other, player).is_none());

        game.record_control(bearer, other, None);
        assert_eq!(game.actor(), Some(player));
        assert_eq!(
            visible(&game, other, other),
            Some(game.object_ref(foreign_top))
        );
        assert!(visible(&game, player, other).is_none());
        assert!(visible(&game, player, player).is_none());

        game.remove_abilities_exact(&[bearer], provider, other, AnimationDuration::Permanent);
        assert!(visible(&game, other, other).is_none());
        assert!(game.attach_equipment(provider, game.object_ref(spare)));
        assert!(game.attach_equipment(provider, bearer));
        assert_eq!(
            visible(&game, other, other),
            Some(game.object_ref(foreign_top))
        );

        let mut departed = game.clone();
        departed.move_object(creature, Zone::Graveyard);
        departed.move_object(creature, Zone::Battlefield);
        assert_ne!(departed.object_ref(creature), bearer);
        assert!(visible(&departed, other, other).is_none());
        game.remove_abilities_exact(&[provider], bearer, player, AnimationDuration::Permanent);
        assert!(visible(&game, other, other).is_none());
    }
}

#[test]
fn paying_for_the_top_spell_defers_the_next_private_look() {
    let _rules = fixture_rules::activate(&[
        (named("Short Bow"), &LANTERN),
        (named("Bear Cub"), &CAST_TOP),
    ]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut root = scenario(player);
        let lantern = put(&mut root, other, "Short Bow", Zone::Battlefield);
        let bearer = put(&mut root, player, "Bear Cub", Zone::Battlefield);
        let next = put(&mut root, player, "Opt", Zone::Library);
        let top = put(&mut root, player, "Shock", Zone::Library);
        assert!(visible(&root, player, player).is_none());
        assert!(root.attach_equipment(root.object_ref(lantern), root.object_ref(bearer)));
        assert_eq!(visible(&root, player, player), Some(root.object_ref(top)));

        let action = Action::AnnounceCast { card: top };
        let index = root
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let revision = root.revision();
        let saved = root.snapshot();
        let mut raw = root.clone();
        let mut indexed = root.clone();
        drop(root);

        act(&mut raw, action);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        for branch in [&mut raw, &mut indexed] {
            assert!(visible(branch, player, player).is_none());
            assert!(visible(branch, other, player).is_none());
            act(
                branch,
                Action::ChooseTarget {
                    target: Target::Player(other),
                },
            );
            reject(branch, Action::PayCosts { payment: [0; 6] });
            assert!(visible(branch, player, player).is_none());
            pay(branch);
            assert_eq!(
                visible(branch, player, player),
                Some(branch.object_ref(next))
            );
            assert!(visible(branch, other, player).is_none());
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        assert_eq!(
            visible(&indexed, player, player),
            Some(indexed.object_ref(top))
        );
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
    }
}

#[test]
fn a_queued_attack_explores_with_the_bearer_after_the_grantor_leaves() {
    let _rules = fixture_rules::activate(&[(named("Short Bow"), &LANTERN)]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let lantern = put(&mut game, other, "Short Bow", Zone::Battlefield);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let top = put(&mut game, player, "Mountain", Zone::Library);
        let provider = game.object_ref(lantern);
        let bearer = game.object_ref(creature);
        assert!(game.attach_equipment(provider, bearer));

        pass_pair(&mut game);
        pass_pair(&mut game);
        act(&mut game, Action::Attack { card: creature });
        act(&mut game, Action::FinishAttackers);
        let trigger = game.state.stack.last().unwrap();
        assert_eq!(trigger.source.unwrap().object, bearer);
        assert_eq!(
            trigger.attachment_trigger.as_ref().unwrap().grantor,
            provider
        );

        game.move_object(lantern, Zone::Graveyard);
        assert!(visible(&game, player, player).is_none());
        settle(&mut game);
        assert_eq!(game.object_view(top).zone, Zone::Hand);
    }
}
