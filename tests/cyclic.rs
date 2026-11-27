//! Cyclic terms. There is no occur check, so `x = [x]` makes a list that
//! contains itself. Commit 7bb61e6 computed these correctly but overflowed
//! the stack printing a cycle through a list's elements; the printer now
//! shows such an element as `↻`. A cycle through a list's tail is cut short
//! by `MAX_ITEMS`, as it always was.
use absys::machine::Opts;
use absys::report::{run, Show};
use absys::session::Session;

fn show(text: &str) -> String {
    let mut s = Session::new(Opts::default());
    run(&mut s, text, Show::default()).unwrap()
}

#[test]
fn a_list_that_is_its_own_element() {
    assert!(show("x = [x]").contains("x = [↻]"));
}

#[test]
fn a_cycle_after_other_elements() {
    assert!(show("x = [1, x]").contains("x = [1,↻]"));
    assert!(show("x = [[x], x]").contains("x = [[↻],↻]"));
}

#[test]
fn a_cycle_through_two_variables() {
    let out = show("x = [y]; y = [x]");
    assert!(out.contains("x = [[↻]]"), "{out}");
    assert!(out.contains("y = [[↻]]"), "{out}");
}

#[test]
fn a_cycle_into_the_middle_of_a_list() {
    assert!(show("x = [1|t]; t = [t]").contains("x = [1,↻]"));
}

#[test]
fn a_cycle_through_the_tail_is_cut_short_as_before() {
    let out = show("x = [1|x]");
    let want = format!("x = [{}…]", "1,".repeat(32));
    assert!(out.contains(&want), "{out}");
}

#[test]
fn shared_but_acyclic_terms_are_printed_in_full() {
    assert!(show("x = [y, y]; y = [1]").contains("x = [[1],[1]]"));
    assert!(show("z = [t, 1|t]; t = [2]").contains("z = [[2],1,2]"));
}

#[test]
fn both_kinds_of_cycle_together_end() {
    let out = show("x = [1, 2|t]; t = [x|t]");
    assert!(out.contains("x = [1,2,↻,↻"), "{out}");
}
