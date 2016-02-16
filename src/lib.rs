pub mod observer;
#[cfg(test)]
mod test_observer {
	use super::observer;
	use std::fmt::Debug;
	use std::marker::PhantomData;
	use std::sync::atomic::{AtomicBool, Ordering};
	use std::sync::Arc;

	pub struct TestObserver<T> {
		count: i32,
		target: i32,
		completed: Arc<AtomicBool>,
	    _t: PhantomData<T>
	}

	pub struct CompletionGuard {
		completed: Arc<AtomicBool>,
		expected_completion: bool
	}
	impl Drop for CompletionGuard {
		fn drop(&mut self) {
			let actual_completion = self.completed.load(Ordering::Relaxed);
			if self.expected_completion {
				assert!(actual_completion);
			} else {
				assert!(!actual_completion);
			}
		}
	}

	impl<T> TestObserver<T> {
		pub fn ensure_completion(&self) -> CompletionGuard {
		    CompletionGuard { completed: self.completed.clone(), expected_completion: true }
		}
		pub fn ensure_no_completion(&self) -> CompletionGuard {
			CompletionGuard { completed: self.completed.clone(), expected_completion: false }
		}
	}

	pub fn new<T>(target: i32) -> TestObserver<T> {
		TestObserver { count: 0, target: target, completed: Arc::new(AtomicBool::new(false)), _t: PhantomData }
	}
	impl<T: Debug> observer::Observer for TestObserver<T> {
	 	type Item = T;
	 	fn on_next(mut self, val: T) -> Option<Self> {
	 		self.count += 1;
	 		println!("{:?}", val);
	 		Some(self)
	 	}
	 	fn on_completed(self) {
	 		assert!(self.count == self.target);
	 		self.completed.store(true, Ordering::Relaxed);
	 	}
	}
}
#[test]
fn it_works() {
	use observer::{Subscribable, Observable};
	let s = observer::test_source::from_iter(1..5).filter(|&x| x > 2);
	 
	let observer = test_observer::new(2);
	let _completion = observer.ensure_completion();
	let _subscription = s.subscribe(observer);
}

#[test]
fn take_works() {
	use observer::{Subscribable};

	let s = observer::take::new(observer::test_source::from_iter(1..5), 3);
	let observer = test_observer::new(3);
	let _completion = observer.ensure_completion();
	let _subscription = s.subscribe(observer);

}

#[test]
fn never_works() {
	use observer::{Subscribable};

	let s = observer::never::<i32>();
	let observer = test_observer::new(3);
	let _completion = observer.ensure_no_completion();
	let _subscription = s.subscribe(observer);
}