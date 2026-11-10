//! A resolved subtype grant includes noncreatures and follows exact recipients.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const GRANT: Keywords = Keywords::FLYING
    .union(Keywords::FIRST_STRIKE)
    .union(Keywords::TRAMPLE)
    .union(Keywords::LIFELINK)
    .union(Keywords::HASTE);
const SOURCE_NAME: &str = "Spider-Byte, Web Warden";
const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantOtherPermanentGroupKeywords {
            filters: &[BattlefieldFilter {
                kinds: 0,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: Some("Spider"),
                max_mana: u32::MAX,
            }],
            keywords: GRANT,
        }],
    }],
    ..CardRules::EMPTY
};

#[test]
fn other_spiders_include_effective_noncreature_subtypes_and_freeze_on_resolution() {
    let _rules = fixture_rules::activate(&[(Card::from_name(SOURCE_NAME).unwrap(), &SOURCE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, SOURCE_NAME, Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    let opposing_spider = put(
        &mut game,
        player.other(),
        "Treetop Snarespinner",
        Zone::Battlefield,
    );
    let spider = put(&mut game, player, "Treetop Snarespinner", Zone::Battlefield);
    let kindred = put(&mut game, player, "Boggart Mischief", Zone::Battlefield);
    let kindred_ref = game.object_ref(kindred);
    let mut recipe = CopyRecipe::new(game.object_view(kindred).card);
    recipe.replacement_subtype = Some(CreatureType::from_name("Spider").unwrap());
    game.set_copy_recipe(kindred_ref, recipe);
    assert!(game.has_subtype(kindred, "Spider"));
    assert!(!game.is_creature(kindred));
    assert!(game.has_subtype(source, "Spider"));

    let copy_spell = put(&mut game, player, "Self-Reflection", Zone::Hand);
    cast(&mut game, copy_spell, Some(Target::Card(spider)));
    settle(&mut game);
    let token = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield && object.is_token() && object.has_subtype("Spider")
        })
        .unwrap()
        .reference;
    let lands = [
        put(&mut game, player, "Plains", Zone::Battlefield),
        put(&mut game, player, "Plains", Zone::Battlefield),
    ];
    for _ in 0..12 {
        if game.state.step == Step::BeginCombat && !game.state.stack.is_empty() {
            break;
        }
        act(&mut game, Action::Pass);
    }
    assert_eq!(game.state.step, Step::BeginCombat);
    assert_eq!(game.state.stack.len(), 1);
    assert!(game.state.stack[0].targets.is_empty());
    settle(&mut game);
    for id in [spider, token.card, kindred] {
        assert!(game.keywords(id).contains(GRANT));
    }
    for id in [source, opposing_spider] {
        assert!(!game.keywords(id).intersects(GRANT));
    }

    game.clear_copy_recipe(kindred_ref);
    assert!(!game.has_subtype(kindred, "Spider"));
    assert!(game.keywords(kindred).contains(GRANT));
    let old_spider = game.object_ref(spider);
    for card in lands {
        act(&mut game, Action::TapMana { card });
    }
    let blink = put(&mut game, player, "Personify", Zone::Hand);
    cast(&mut game, blink, Some(Target::Card(spider)));
    settle(&mut game);
    assert_ne!(game.object_ref(spider), old_spider);
    assert!(game.has_subtype(spider, "Spider"));
    assert!(!game.keywords(spider).intersects(GRANT));
    assert!(game.keywords(token.card).contains(GRANT));
}
