//! CR 608.2c applies two selected destroy instructions in printed order.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentKeywords {
            kinds: 16,
            keywords: Keywords::INDESTRUCTIBLE,
        }],
    }],
    ..CardRules::EMPTY
};
const ARTIFACT: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 16,
    relation: -1,
    other: false,
    nonland: true,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const MODAL: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy a creature",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Destroy an artifact",
            targets: TargetPlan::Single(TargetSpec::one(ARTIFACT)),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Both",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Creature),
                TargetSpec::one(ARTIFACT),
            ]),
            program: &[
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::Destroy,
            ],
        },
    ]),
    ..CardRules::EMPTY
};
const REPEATED: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Creature),
            TargetSpec::one(TargetRule::Creature),
        ]),
        &[
            Instruction::Destroy,
            Instruction::SelectGroup(1),
            Instruction::Destroy,
        ],
    ),
    ..CardRules::EMPTY
};

#[test]
fn destroying_the_first_target_changes_protection_before_the_second_instruction() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &MODAL),
        (Card::from_name("Grizzly Bears").unwrap(), &PROVIDER),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        let provider = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let artifact = put(
            &mut game,
            player.other(),
            "Basilisk Collar",
            Zone::Battlefield,
        );
        assert!(game.keywords(artifact).contains(Keywords::INDESTRUCTIBLE));
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseMode { mode: 2 });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(provider),
            },
        );
        let saved = game.snapshot();
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(provider),
            },
        );
        let mut indexed = game.clone();
        let choice = Action::ChooseTarget {
            target: Target::Card(artifact),
        };
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == choice)
            .unwrap();
        act(&mut game, choice.clone());
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        assert_eq!(indexed.snapshot(), game.snapshot());
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(provider).zone, Zone::Graveyard);
        assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);

        indexed.restore(&saved).unwrap();
        act(&mut indexed, choice);
        pay(&mut indexed);
        let bounce = put(&mut indexed, player, "Unsummon", Zone::Hand);
        cast(&mut indexed, bounce, Some(Target::Card(provider)));
        settle(&mut indexed);
        assert_eq!(indexed.object_view(provider).zone, Zone::Hand);
        assert_eq!(indexed.object_view(artifact).zone, Zone::Graveyard);
    }
}

#[test]
fn one_legal_object_can_fill_both_distinct_target_groups() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &REPEATED)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    let target = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.add_counter(target, CounterKind::Shield, 1);
    announce(
        &mut game,
        spell,
        &[Target::Card(target), Target::Card(target)],
    );
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Graveyard);
}
