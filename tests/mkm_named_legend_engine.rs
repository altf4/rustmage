//! Exact named legendary forms and controller-relative acquired statics.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const NAME: &str = "Mileva, the Stalwart";

const fn form(program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
const EVOLUTION: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    activated: &[
        form(&[Instruction::ChangeSourceCreatureForm(AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Named(&["Human", "Detective"]),
            base: Some((4, 4)),
            keywords: Keywords::VIGILANCE,
            duration: AnimationDuration::Permanent,
        })]),
        form(&[
            Instruction::IfSubjectSubtype {
                subject: Subject::Source,
                subtype: "Detective",
                skip: 1,
            },
            Instruction::BecomeNamedLegendaryCreature {
                name: NAME,
                base: (5, 5),
                ability: SelfGrantedAbilityKind::OtherControlledCreatureKeywords(
                    Keywords::INDESTRUCTIBLE,
                ),
            },
        ]),
    ],
    ..CardRules::EMPTY
};
const BLANK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::RemoveAbilities {
            subject: Subject::Targets,
            duration: AnimationDuration::EndOfTurn,
        }],
    ),
    ..CardRules::EMPTY
};

fn evolve(game: &mut Game, card: CardId, ability: u16) {
    let action = Action::Activate {
        card,
        ability,
        target: None,
        payment: [0; 6],
        counter_payment: Counters::default(),
    };
    if game.legal_actions().contains(&action) {
        act(game, action);
    } else {
        act(
            game,
            Action::AnnounceActivate {
                card,
                ability,
                counter_payment: Counters::default(),
            },
        );
        pay(game);
    }
    settle(game);
}

fn indestructible(game: &Game, id: CardId) -> bool {
    game.object_view(id)
        .keywords
        .contains(Keywords::INDESTRUCTIBLE)
}

#[test]
fn repeated_forms_keep_the_name_and_static_until_the_exact_source_leaves() {
    let card = Card::from_name("Grizzly Bears").unwrap();
    let blank = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(card, &EVOLUTION), (blank, &BLANK)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let hero = spawn(&mut game, player, card, Zone::Battlefield);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let rival = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    evolve(&mut game, hero, 1);
    assert!(!game.object_view(hero).is_legendary());
    assert_eq!(stats(&game, hero), (2, 3));
    evolve(&mut game, hero, 0);
    evolve(&mut game, hero, 1);
    let named = game.object_view(hero);
    assert_eq!(named.name, NAME);
    assert!(named.is_legendary() && named.has_subtype("Detective"));
    assert!(named.keywords.contains(Keywords::VIGILANCE));
    assert_eq!(stats(&game, hero), (5, 5));
    assert!(indestructible(&game, ally));
    assert!(!indestructible(&game, hero));
    assert!(!indestructible(&game, rival));
    let copiable = game.copiable_recipe(named.reference);
    assert_eq!(copiable.definition, card);
    assert_eq!(copiable.overrides, None);

    evolve(&mut game, hero, 0);
    assert_eq!(game.object_view(hero).name, NAME);
    assert!(game.object_view(hero).is_legendary());
    assert_eq!(stats(&game, hero), (4, 4));
    assert!(indestructible(&game, ally));

    let mut blanked = game.clone();
    let blank = spawn(&mut blanked, player, blank, Zone::Hand);
    cast(&mut blanked, blank, Some(Target::Card(hero)));
    settle(&mut blanked);
    assert!(!indestructible(&blanked, ally));
    assert_eq!(blanked.object_view(hero).name, NAME);
    assert_eq!(stats(&blanked, hero), (4, 4));

    game.record_control(named.reference, opponent, None);
    assert!(!indestructible(&game, ally));
    assert!(indestructible(&game, rival));
    game.move_object(hero, Zone::Exile);
    game.move_object(hero, Zone::Battlefield);
    assert!(!indestructible(&game, rival));
    assert_eq!(game.object_view(hero).name, "Grizzly Bears");
    assert!(!game.object_view(hero).is_legendary());
    assert_eq!(stats(&game, hero), (2, 3));
}
