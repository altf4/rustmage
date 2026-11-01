use super::*;
use super::*;
use crate::cards::PlayerCounterKind;
pub use rustmage_core::engine::player_counter_events::*;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owned_bytes_excludes_inline_queue_and_counts_events() {
        let mut events = PlayerCounterEvents::default();
        assert_eq!(events.owned_bytes(), 0);
        events.push(Player::P0, PlayerCounterKind::Poison, 1, 0);
        assert_eq!(
            events.owned_bytes(),
            std::mem::size_of::<PlayerCounterEvent>()
        );
    }

    #[test]
    fn redirected_and_split_events_keep_separate_replacement_history() {
        for player in [Player::P0, Player::P1] {
            let mut queue = PlayerCounterEvents::default();
            let initial = queue.push(player, PlayerCounterKind::Poison, 2, 41);
            let other = queue.push(player.other(), PlayerCounterKind::Energy, 1, 42);
            let source = ObjectRef {
                card: 3,
                generation: 1,
            };
            assert!(queue.mark_external(initial, (source, 7)));
            let applied = EffectIdentity {
                origin_stack_id: 41,
                index: 0,
            };
            let destination = CounterDestination {
                player: player.other(),
                kind: PlayerCounterKind::Energy,
                amount: 3,
            };
            assert!(queue.apply_self(initial, applied, &[destination, destination]));
            assert_eq!(queue.events().len(), 3);
            assert_ne!(queue.events()[0].id, queue.events()[1].id);
            assert_ne!(queue.events()[1].id, other);
            for event in &queue.events()[..2] {
                assert_eq!(event.effect_origin, 41);
                assert_eq!(event.applied_self, vec![applied]);
                assert_eq!(event.applied_external, vec![(source, 7)]);
            }
            assert!(queue.events()[2].applied_external.is_empty());
            let saved = queue.clone();
            let first = queue.events()[0].id;
            assert!(!queue.apply_self(first, applied, &[destination]));
            assert!(!queue.apply_self(
                first,
                EffectIdentity {
                    origin_stack_id: 42,
                    index: 1
                },
                &[destination]
            ));
            assert!(queue.mark_external(first, (source, 8)));
            assert!(!queue.mark_external(first, (source, 8)));
            assert_eq!(queue.events()[1].applied_external, vec![(source, 7)]);
            queue = saved.clone();
            assert_eq!(queue, saved);
            assert!(queue.apply_self(
                first,
                EffectIdentity {
                    origin_stack_id: 41,
                    index: 1
                },
                &[]
            ));
            assert_eq!(queue.events().len(), 2);
        }
    }
}
