pub mod observer;

#[cfg(test)]
use std::fmt::Debug;
#[cfg(test)]
struct TestObserver<T> {
	count : i32,
	target : i32,
    _t : std::marker::PhantomData<T>
}
 #[cfg(test)]
 impl<T : Debug> observer::Observer for TestObserver<T> {
 	type Item = T;
 	fn on_next(mut self, val : T) -> Option<Self> {
 		self.count += 1;
 		println!("{:?}", val);
 		Some(self)
 	}
 	fn on_completed(self) {
 		assert!(self.count == self.target);
 	}
}

#[test]
fn it_works() {
	use observer::Subscribable;
	let s = observer::test_source::from_iter((1..5).into_iter());
	let s = observer::filter(s, |&x| { x > 2 });
	 
	 let observer = TestObserver { count: 0, target: 2, _t: std::marker::PhantomData };
	 let _subscription = s.subscribe(observer);
}
