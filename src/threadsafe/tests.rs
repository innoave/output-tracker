use super::*;
use assertor::*;
use proptest::collection::vec;
use proptest::prelude::*;
use std::thread;
use std::time::Duration;

#[test]
fn an_output_tracker_can_be_created_from_a_default_subject() {
    let subject = OutputSubject::<String>::default();
    let tracker = subject
        .create_tracker()
        .unwrap_or_else(|err| panic!("could not create output tracker: {err}"));

    assert_that!(tracker.output()).ok().is_empty();
}

#[test]
fn a_new_output_tracker_has_no_items_recorded() {
    let subject = OutputSubject::<String>::new();
    let tracker = subject
        .create_tracker()
        .unwrap_or_else(|err| panic!("could not create output tracker: {err}"));

    assert_that!(tracker.output()).ok().is_empty();
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn an_output_tracker_records_any_number_of_items_in_order(
        items in (0..=10_000_usize).prop_flat_map(|size| vec(any::<i64>(), size)),
    ) {
        let subject = OutputSubject::<i64>::new();
        let tracker = subject
            .create_tracker()
            .unwrap_or_else(|err| panic!("could not create output tracker: {err}"));

        for item in &items {
            subject.emit(*item)
                .unwrap_or_else(|err| panic!("could not emit item {item} on output subject: {err}"));
        }

        let output = tracker.output()
            .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"));
        prop_assert_eq!(output, items);
    }

    #[test]
    fn the_output_of_a_tracker_can_be_read_several_times(
        items in (0..=10_000_usize).prop_flat_map(|size| vec(any::<i64>(), size)),
    ) {
        let subject = OutputSubject::<i64>::new();
        let tracker = subject
            .create_tracker()
            .unwrap_or_else(|err| panic!("could not create output tracker: {err}"));

        for item in &items {
            subject.emit(*item)
                .unwrap_or_else(|err| panic!("could not emit item {item} on output subject: {err}"));
        }

        let output = tracker.output()
            .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"));
        prop_assert_eq!(&output, &items);
        let output = tracker.output()
            .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"));
        prop_assert_eq!(&output, &items);
        let output = tracker.output()
            .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"));
        prop_assert_eq!(&output, &items);
    }

    #[test]
    fn several_output_tracker_for_the_same_subject_track_same_items_independently(
        num_trackers in 2..=300_usize,
        items in (0..=500_usize).prop_flat_map(|size| vec(any::<i64>(), size)),
    ) {
        let subject = OutputSubject::<i64>::new();
        let trackers = (0..num_trackers).map(|i| {
            subject
            .create_tracker()
            .unwrap_or_else(|err| panic!("could not create output tracker {i}: {err}"))
        }).collect::<Vec<_>>();

        for item in &items {
            subject.emit(*item)
                .unwrap_or_else(|err| panic!("could not emit item {item} on output subject: {err}"));
        }

        for tracker in trackers {
            let output = tracker.output()
                .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"));
            prop_assert_eq!(&output, &items);
        }
    }

    #[test]
    fn after_an_output_tracker_is_stopped_it_no_longer_tracks_items(
        items_before_stop in (0..=300_usize).prop_flat_map(|size| vec(any::<i64>(), size)),
        items_after_stop in (0..=50_usize).prop_flat_map(|size| vec(any::<i64>(), size)),
    ) {
        let subject = OutputSubject::<i64>::new();
        let tracker = subject
            .create_tracker()
            .unwrap_or_else(|err| panic!("could not create output tracker: {err}"));

        for item in &items_before_stop {
            subject.emit(*item)
                .unwrap_or_else(|err| panic!("could not emit item {item} on output subject: {err}"));
        }

        tracker.stop().unwrap_or_else(|err| panic!("failed to stop output tracker: {err}"));

        for item in &items_after_stop {
            subject.emit(*item)
                .unwrap_or_else(|err| panic!("could not emit item {item} on output subject: {err}"));
        }

        let output = tracker.output()
            .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"));
        prop_assert_eq!(output, items_before_stop);
    }

    #[test]
    fn after_an_output_tracker_is_stopped_it_still_outputs_items_recorded_before(
        items_before_stop in (0..=300_usize).prop_flat_map(|size| vec(any::<i64>(), size)),
    ) {
        let subject = OutputSubject::<i64>::new();
        let tracker = subject
            .create_tracker()
            .unwrap_or_else(|err| panic!("could not create output tracker: {err}"));

        for item in &items_before_stop {
            subject.emit(*item)
                .unwrap_or_else(|err| panic!("could not emit item {item} on output subject: {err}"));
        }

        tracker.stop().unwrap_or_else(|err| panic!("failed to stop output tracker: {err}"));

        let output = tracker.output()
            .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"));
        prop_assert_eq!(output, items_before_stop);
    }

    #[test]
    fn stopping_an_output_tracker_several_times_does_not_give_error_or_panic(
        items_before_stop in (0..=300_usize).prop_flat_map(|size| vec(any::<i64>(), size)),
    ) {
        let subject = OutputSubject::<i64>::new();
        let tracker = subject
            .create_tracker()
            .unwrap_or_else(|err| panic!("could not create output tracker: {err}"));

        for item in &items_before_stop {
            subject.emit(*item)
                .unwrap_or_else(|err| panic!("could not emit item {item} on output subject: {err}"));
        }

        assert_that!(tracker.stop()).is_ok();
        assert_that!(tracker.stop()).is_ok();
        assert_that!(tracker.stop()).is_ok();
    }

    #[test]
    fn after_clearing_an_output_tracker_its_output_is_empty(
        items in (0..=500_usize).prop_flat_map(|size| vec(any::<i64>(), size)),
    ) {
        let subject = OutputSubject::<i64>::new();
        let tracker = subject
            .create_tracker()
            .unwrap_or_else(|err| panic!("could not create output tracker: {err}"));

        for item in &items {
            subject.emit(*item)
                .unwrap_or_else(|err| panic!("could not emit item {item} on output subject: {err}"));
        }

        tracker.clear().unwrap_or_else(|err| panic!("failed to stop output tracker: {err}"));

        let output = tracker.output()
            .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"));
        prop_assert!(output.is_empty(), "output of tracker should be empty, but is: {output:?}");
    }

    #[test]
    fn after_clearing_an_output_tracker_it_outputs_only_items_emitted_after_clearing(
        items_before_clear in (0..=50_usize).prop_flat_map(|size| vec(any::<i64>(), size)),
        items_after_clear in (0..=300_usize).prop_flat_map(|size| vec(any::<i64>(), size)),
    ) {
        let subject = OutputSubject::<i64>::new();
        let tracker = subject
            .create_tracker()
            .unwrap_or_else(|err| panic!("could not create output tracker: {err}"));

        for item in &items_before_clear {
            subject.emit(*item)
                .unwrap_or_else(|err| panic!("could not emit item {item} on output subject: {err}"));
        }

        tracker.clear().unwrap_or_else(|err| panic!("failed to stop output tracker: {err}"));

        for item in &items_after_clear {
            subject.emit(*item)
                .unwrap_or_else(|err| panic!("could not emit item {item} on output subject: {err}"));
        }

        let output = tracker.output()
            .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"));
        prop_assert_eq!(output, items_after_clear);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn an_output_tracker_tracks_items_emmitted_from_different_threads(
        (n1, n2, _n3, items) in (0..=200_usize, 0..=200_usize, 0..=200_usize).prop_flat_map(|(s1, s2, s3)| (Just(s1), Just(s2), Just(s3), vec(any::<i64>(), s1 + s2 + s3))),
    ) {
        let subject = OutputSubject::<i64>::new();
        let tracker = subject
            .create_tracker()
            .unwrap_or_else(|err| panic!("could not create output tracker 1: {err}"));

        let subject1 = subject.clone();
        let items_thread1 = items[0..n1].to_owned();
        let thread1 = thread::spawn(move || {
            for item in &items_thread1 {
                subject1.emit(*item)
                    .unwrap_or_else(|err| panic!("could not emit item {item} on output subject: {err}"));
            }
        });

        let subject2 = subject.clone();
        let items_thread2 = items[n1..n1 + n2].to_owned();
        let thread2 = thread::spawn(move || {
            for item in &items_thread2 {
                subject2.emit(*item)
                    .unwrap_or_else(|err| panic!("could not emit item {item} on output subject: {err}"));
            }
        });

        let subject3 = subject;
        let items_thread3 = items[n1 + n2..].to_owned();
        let thread3 = thread::spawn(move || {
            for item in &items_thread3 {
                subject3.emit(*item)
                    .unwrap_or_else(|err| panic!("could not emit item {item} on output subject: {err}"));
            }
        });

        thread1.join().unwrap_or_else(|err| panic!("thread 1 panicked: {err:?}"));
        thread2.join().unwrap_or_else(|err| panic!("thread 2 panicked: {err:?}"));
        thread3.join().unwrap_or_else(|err| panic!("thread 3 panicked: {err:?}"));

        let mut output = tracker.output()
            .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"));
        output.sort_unstable();

        let mut items = items;
        items.sort_unstable();

        prop_assert_eq!(&output, &items);
    }

    #[test]
    fn output_tracker_can_be_moved_to_different_threads(
        items in (0..=500_usize).prop_flat_map(|size| vec(any::<i64>(), size)),
    ) {
        let subject = OutputSubject::<i64>::new();
        let tracker1 = subject
            .create_tracker()
            .unwrap_or_else(|err| panic!("could not create output tracker 1: {err}"));
        let tracker2 = subject
            .create_tracker()
            .unwrap_or_else(|err| panic!("could not create output tracker 2: {err}"));
        let tracker3 = subject
            .create_tracker()
            .unwrap_or_else(|err| panic!("could not create output tracker 3: {err}"));

        let thread1 = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            tracker1.output()
                .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"))
        });

        let thread2 = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            tracker2.output()
                .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"))
        });

        let thread3 = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            tracker3.output()
                .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"))
        });

        for item in &items {
            subject.emit(*item)
                .unwrap_or_else(|err| panic!("could not emit item {item} on output subject: {err}"));
        }

        let output1 = thread1.join().unwrap_or_else(|err| panic!("thread 1 panicked: {err:?}"));
        let output2 = thread2.join().unwrap_or_else(|err| panic!("thread 2 panicked: {err:?}"));
        let output3 = thread3.join().unwrap_or_else(|err| panic!("thread 3 panicked: {err:?}"));

        prop_assert_eq!(&output1, &items);
        prop_assert_eq!(&output2, &items);
        prop_assert_eq!(&output3, &items);
    }

    #[test]
    fn output_tracker_can_be_created_from_different_threads(
        items in (0..=500_usize).prop_flat_map(|size| vec(any::<i64>(), size)),
    ) {
        let subject = OutputSubject::<i64>::new();

        let subject1 = subject.clone();
        let thread1 = thread::spawn(move || {
            let tracker1 = subject1
                .create_tracker()
                .unwrap_or_else(|err| panic!("could not create output tracker 1: {err}"));
            thread::sleep(Duration::from_millis(15));
            tracker1.output()
                .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"))
        });

        let subject2 = subject.clone();
        let thread2 = thread::spawn(move || {
            let tracker2 = subject2
                .create_tracker()
                .unwrap_or_else(|err| panic!("could not create output tracker 2: {err}"));
            thread::sleep(Duration::from_millis(15));
            tracker2.output()
                .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"))
        });

        let subject3 = subject.clone();
        let thread3 = thread::spawn(move || {
            let tracker3 = subject3
                .create_tracker()
                .unwrap_or_else(|err| panic!("could not create output tracker 3: {err}"));
            thread::sleep(Duration::from_millis(15));
            tracker3.output()
                .unwrap_or_else(|err| panic!("failed to read tracker output: {err}"))
        });

        thread::sleep(Duration::from_millis(5));

        for item in &items {
            subject.emit(*item)
                .unwrap_or_else(|err| panic!("could not emit item {item} on output subject: {err}"));
        }

        let output1 = thread1.join().unwrap_or_else(|err| panic!("thread 1 panicked: {err:?}"));
        let output2 = thread2.join().unwrap_or_else(|err| panic!("thread 2 panicked: {err:?}"));
        let output3 = thread3.join().unwrap_or_else(|err| panic!("thread 3 panicked: {err:?}"));

        prop_assert_eq!(&output1, &items);
        prop_assert_eq!(&output2, &items);
        prop_assert_eq!(&output3, &items);
    }
}
