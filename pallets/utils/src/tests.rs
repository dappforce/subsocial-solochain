use crate::{mock::*, remove_from_vec, log_2};

#[test]
fn log_2_should_work() {
    ExtBuilder::build().execute_with(|| {
        // None should be returned if zero (0) is provided
        assert!(log_2(0).is_none());

        // Log2 of 1 should be zero (0)
        assert_eq!(log_2(1), Some(0));

        // Log2 of 2 should be 1
        assert_eq!(log_2(2), Some(1));

        // Log2 of 128 should be 7
        assert_eq!(log_2(128), Some(7));

        // Log2 of 512 should be 9
        assert_eq!(log_2(512), Some(9));

        // Log2 of u32::MAX (4294967295) should be 31
        assert_eq!(log_2(u32::MAX), Some(31));
    });
}

#[test]
fn remove_from_vec_should_work_with_zero_elements() {
    ExtBuilder::build().execute_with(|| {
        let element: u16 = 2;
        let vector: &mut Vec<u16> = &mut vec![];

        remove_from_vec(vector, element);
        assert!(vector.is_empty());
    });
}

#[test]
fn remove_from_vec_should_work_with_last_element() {
    ExtBuilder::build().execute_with(|| {
        let element: u16 = 2;
        let vector: &mut Vec<u16> = &mut vec![6, 2];

        vector.remove(0);
        assert_eq!(vector, &mut vec![2]);

        remove_from_vec(vector, element);
        assert!(vector.is_empty());
    });
}

#[test]
fn remove_from_vec_should_work_with_two_elements() {
    ExtBuilder::build().execute_with(|| {
        let element: u16 = 2;
        let vector: &mut Vec<u16> = &mut vec![6, 2, 7];

        vector.remove(0);
        assert_eq!(vector, &mut vec![2, 7]);

        remove_from_vec(vector, element);
        assert_eq!(vector, &mut vec![7]);
    });
}

#[test]
fn convert_users_vec_to_btree_set_should_work() {
    ExtBuilder::build().execute_with(|| {
        // Empty vector should produce empty set
        assert_eq!(
            _convert_users_vec_to_btree_set(vec![]).ok().unwrap(),
            UsersSet::new()
        );

        assert_eq!(
            _convert_users_vec_to_btree_set(vec![USER1]).ok().unwrap(),
            vec![USER1].into_iter().collect()
        );

        // Duplicates should produce 1 unique element
        assert_eq!(
            _convert_users_vec_to_btree_set(vec![USER1, USER1, USER3]).ok().unwrap(),
            vec![USER1, USER3].into_iter().collect()
        );

        // Randomly filled vec should produce sorted set
        assert_eq!(
            _convert_users_vec_to_btree_set(vec![USER3, USER1, USER3, USER2, USER1]).ok().unwrap(),
            vec![USER1, USER2, USER3].into_iter().collect()
        );
    });
}
