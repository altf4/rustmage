//! Starcage's acquired ability pairs link only their own exile events.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const COPY_ARTIFACT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 16,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: true,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn activate(game: &mut Game, source: ObjectRef) {
    let actor = game.actor().unwrap();
    game.state.players[actor.index()].mana = [2, 0, 0, 0, 0, 6];
    act(
        game,
        Action::AnnounceActivate {
            card: source.card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(game);
    assert_eq!(
        game.state.stack.last().unwrap().source().unwrap().object,
        source
    );
}

fn robots(game: &Game, player: Player) -> usize {
    game.observe(player)
        .objects
        .iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.token.is_some()
                && object.has_subtype("Robot")
        })
        .count()
}

#[test]
fn queued_old_pair_keeps_its_cards_and_a_new_copy_pair_has_no_link() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &COPY_ARTIFACT)]);
    for copied_at_entry in [false, true] {
        let player = Player::P0;
        let mut game = scenario(player);
        let victim = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let (cage, template) = if copied_at_entry {
            let template = put(&mut game, player, "Pinnacle Starcage", Zone::Battlefield);
            let entrant = put(&mut game, player, "Grizzly Bears", Zone::Hand);
            (entrant, Some(game.object_ref(template)))
        } else {
            (
                put(&mut game, player, "Pinnacle Starcage", Zone::Hand),
                None,
            )
        };
        cast(&mut game, cage, None);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        if let Some(template) = template {
            act(
                &mut game,
                Action::ChooseEntryCopy {
                    object: Some(template),
                },
            );
        }
        let source = game.object_ref(cage);
        let original = game.copy_ability_origin(source);
        assert_eq!(original.is_some(), copied_at_entry);
        let trigger = game.state.stack.last().unwrap();
        assert_eq!(trigger.kind, StackKind::TriggeredAbility { ability: 0 });
        assert_eq!(trigger.source().unwrap().object, source);
        assert_eq!(trigger.source_ability_origin, original);
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Exile);
        assert_eq!(
            game.state.exile_links,
            [ExileLink {
                source,
                exiled: game.object_ref(victim),
                return_zone: Zone::Battlefield,
                ability_origin: original,
            }]
        );
        let before = game.snapshot();
        let same_copiable_values = game.copiable_recipe(source);

        let mut old = game.clone();
        activate(&mut old, source);
        assert_eq!(
            old.state.stack.last().unwrap().source_ability_origin,
            original
        );
        old.apply_temporary_copy(source, same_copiable_values, CopyDeadline::EndOfTurn);
        assert_ne!(old.copy_ability_origin(source), original);
        settle(&mut old);
        assert_eq!(old.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(robots(&old, player), 1);

        let mut new = game.clone();
        new.apply_temporary_copy(source, same_copiable_values, CopyDeadline::EndOfTurn);
        let acquired = new.copy_ability_origin(source);
        assert!(acquired.is_some());
        assert_ne!(acquired, original);
        activate(&mut new, source);
        assert_eq!(
            new.state.stack.last().unwrap().source_ability_origin,
            acquired
        );
        settle(&mut new);
        assert_eq!(new.object_view(victim).zone, Zone::Battlefield);
        assert_eq!(robots(&new, player), 0);
        assert!(new.state.exile_links.is_empty());
        assert_eq!(game.snapshot(), before);
    }
}
