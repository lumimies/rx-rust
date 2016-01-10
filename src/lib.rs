mod observer;
#[test]
fn it_works() {
	let s = observer::test_source::from_iter((1..5).into_iter());
	let s = observer::filter::new(s, |&x| { x > 5});
}
