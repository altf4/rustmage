mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const GRANT: Keywords = Keywords::TRAMPLE.union(Keywords::LIFELINK);
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::OwnTurnSourceAndKindCreatureWard {
        kinds: 32,
        keywords: GRANT,
        ward: 0,
    }],
}];
const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::Ward),
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterUnlessPay {
            cost: ManaCost {
                colored: [0; 6],
                generic: 2,
            },
            subject: CounterSubject::Related,
            exile: false,
            unpaid_skip: 0,
        }],
    }],
    ..CardRules::EMPTY
};

fn fixture() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::from_name("Grizzly Bears").unwrap(), &SOURCE),
        (Card::from_name("Summon: Fat Chocobo").unwrap(), &SOURCE),
    ])
}
fn target_from_opponent(game: &mut Game, target: CardId) -> CardId {
    let opponent = Player::P1;
    let spell = put(game, opponent, "Unsummon", Zone::Hand);
    if game.actor() != Some(opponent) {
        act(game, Action::Pass);
    }
    cast(game, spell, Some(Target::Card(target)));
    spell
}

#[test]
fn ward_and_keywords_use_one_source_or_enchantment_creature_recipient_set() {
    let _rules = fixture();
    let player = Player::P0;
    for provider_name in ["Grizzly Bears", "Summon: Fat Chocobo"] {
        let mut game = scenario(player);
        let provider = put(&mut game, player, provider_name, Zone::Battlefield);
        let enchantment = put(
            &mut game,
            player,
            "Summon: Primal Garuda",
            Zone::Battlefield,
        );
        let enemy = put(
            &mut game,
            player.other(),
            "Summon: Primal Garuda",
            Zone::Battlefield,
        );
        let vanilla = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let noncreature = put(&mut game, player, "Spelunking", Zone::Battlefield);
        for id in [provider, enchantment] {
            assert!(game.keywords(id).contains(GRANT));
        }
        for id in [enemy, vanilla, noncreature] {
            assert!(!game.keywords(id).intersects(GRANT));
        }

        for target in [provider, enchantment, vanilla] {
            let mut branch = game.clone();
            let spell = target_from_opponent(&mut branch, target);
            if target == vanilla {
                assert_eq!(branch.state.stack.len(), 1);
                continue;
            }
            assert_eq!(branch.state.stack.len(), 2);
            let ward = branch.state.stack.last().unwrap();
            assert_eq!(ward.source.unwrap().object, branch.object_ref(target));
            assert_eq!(
                ward.attachment_trigger.as_ref().unwrap().grantor,
                branch.object_ref(provider)
            );
            if target == enchantment {
                branch.move_object(provider, Zone::Exile);
                assert!(!branch.keywords(enchantment).intersects(GRANT));
                act(&mut branch, Action::Pass);
                act(&mut branch, Action::Pass);
                act(&mut branch, Action::DeclineResolutionMana);
                assert_eq!(branch.object_view(spell).zone, Zone::Graveyard);
                assert_eq!(branch.object_view(target).zone, Zone::Battlefield);
            }
        }
    }
}

#[test]
fn each_provider_grants_a_separate_ward_and_later_ability_removal_suppresses_one() {
    let _rules = fixture();
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Summon: Fat Chocobo", Zone::Battlefield);
    let target = put(
        &mut game,
        player,
        "Summon: Primal Garuda",
        Zone::Battlefield,
    );
    let mut two = game.clone();
    target_from_opponent(&mut two, target);
    assert_eq!(two.state.ordering_triggers.len(), 2);
    let mut grantors: Vec<_> = two
        .state
        .ordering_triggers
        .iter()
        .map(|ward| ward.attachment_trigger.as_ref().unwrap().grantor)
        .collect();
    grantors.sort_unstable();
    assert_eq!(grantors, [two.object_ref(first), two.object_ref(second)]);

    let blank = put(&mut game, player, "Witness Protection", Zone::Hand);
    cast(&mut game, blank, Some(Target::Card(first)));
    settle(&mut game);
    assert!(!game.keywords(first).intersects(GRANT));
    assert!(game.keywords(target).contains(GRANT));
    target_from_opponent(&mut game, target);
    assert_eq!(game.state.stack.len(), 2);
    assert_eq!(
        game.state.stack[1]
            .attachment_trigger
            .as_ref()
            .unwrap()
            .grantor,
        game.object_ref(second)
    );
}

#[test]
fn the_grants_disappear_on_the_opponents_turn() {
    let _rules = fixture();
    let player = Player::P0;
    let mut game = scenario(player.other());
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let target = put(
        &mut game,
        player,
        "Summon: Primal Garuda",
        Zone::Battlefield,
    );
    for id in [source, target] {
        assert!(!game.keywords(id).intersects(GRANT));
    }
    target_from_opponent(&mut game, target);
    assert_eq!(game.state.stack.len(), 1);
}
