mod observer;
#[test]
fn it_works() {
	use std::fmt::Debug;
	use observer::Subscribable;
	let s = observer::test_source::from_iter((1..5).into_iter());
	let s = observer::filter(s, |&x| { x > 2 });

	struct TestObserver<T> {
		count : i32,
		target : i32,
	    _t : std::marker::PhantomData<T>
	 }
	 impl<T : Debug> observer::Observer for TestObserver<T> {
	 	type Item = T;
	 	fn on_next(mut self, val : T) -> Self {
	 		self.count += 1;
	 		println!("{:?}", val);
	 		self
	 	}
	 	fn on_completed(self) {
	 		assert!(self.count == self.target);
	 	}
	 }
	 let observer : TestObserver<i32> = TestObserver { count: 0, target: 2, _t: std::marker::PhantomData };
	 let subscription = s.subscribe(observer);
}
