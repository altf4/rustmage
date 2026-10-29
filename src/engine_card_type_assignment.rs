//! Exact assignment of distinct card-type slots to objects.
//!
//! This is a matching primitive only. Callers must separately enforce target
//! identity, zone, ownership and the actual card's targeting rules.

/// Whether each mask can be assigned a distinct set bit. A multi-type object
/// occupies one chosen slot, not every slot it supports. The reassignment step
/// avoids making the answer depend on the order of earlier choices.
pub(super) fn distinct_type_assignment(masks: &[u16]) -> bool {
    if masks.len() > u16::BITS as usize {
        return false;
    }
    fn augment(
        index: usize,
        masks: &[u16],
        owners: &mut [Option<usize>; 16],
        seen: &mut u16,
    ) -> bool {
        let mut available = masks[index] & !*seen;
        while available != 0 {
            let bit = available.trailing_zeros() as usize;
            let flag = 1u16 << bit;
            available &= !flag;
            *seen |= flag;
            if owners[bit].is_none_or(|old| augment(old, masks, owners, seen)) {
                owners[bit] = Some(index);
                return true;
            }
        }
        false
    }
    let mut owners = [None; 16];
    for index in 0..masks.len() {
        if !augment(index, masks, &mut owners, &mut 0) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::distinct_type_assignment;
    #[test]
    fn assigns_multi_type_objects_without_greedy_order_dependence() {
        let artifact = 1 << 4;
        let creature = 1;
        let enchantment = 1 << 5;
        assert!(distinct_type_assignment(&[]));
        assert!(distinct_type_assignment(&[artifact | creature, creature]));
        assert!(distinct_type_assignment(&[creature, artifact | creature]));
        assert!(distinct_type_assignment(&[
            artifact | creature,
            creature | enchantment,
            artifact
        ]));
        assert!(!distinct_type_assignment(&[
            artifact | creature,
            artifact | creature,
            creature
        ]));
        assert!(!distinct_type_assignment(&[artifact, 0]));
        assert!(!distinct_type_assignment(&[u16::MAX; 17]));
    }
}
