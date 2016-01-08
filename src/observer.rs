
pub trait Observer<T> {
	fn on_next(self, value : T) -> Self;
	fn on_completed(self);
	// Not sure how the error stuff looks in Rust
	// fn on_error(Self/*,???*/);
}

pub trait Observable<T, O : Observer<T>> {
    type Subscription : Drop;
	fn subscribe(&self, o : O) -> Self::Subscription;
}

struct SubscriptionAdapter<Inner : Drop> { inner_subscription : Inner }
impl<Inner : Drop> Drop for SubscriptionAdapter<Inner> {
    fn drop(&mut self) {}
}
impl<Inner: Drop> SubscriptionAdapter<Inner> {
    pub fn new(inner : Inner) -> Self {
        SubscriptionAdapter { inner_subscription: inner }
    }
}
pub mod filter {
    use std::marker::PhantomData;
    use super::{Observer, Observable, SubscriptionAdapter};
    pub struct Filter<T, Inner, F : Fn(&T) -> bool> { f : F, inner : Inner, _t : PhantomData<T> }

    struct FilterObserver<T,O : Observer<T>, F : Fn(&T) -> bool> { f : F, o : O, _t : PhantomData<T> }

    impl<T, O : Observer<T>, F : Fn(&T) -> bool> Observer<T> for FilterObserver<T, O, F> {
            fn on_next(self, value : T) -> Self {
                if (self.f)(&value) {
                    FilterObserver { f: self.f, o: self.o.on_next(value), _t: self._t }
                } else { self }
            }
            fn on_completed(self) {
                self.o.on_completed();
            }
        }

    impl<T, Q : Observer<T>, O : Observable<T,FilterObserver<T, Q, F>>, F : Fn(&T)->bool + Clone> Observable<T, Q> for Filter<T,O,F> {
        type Subscription = SubscriptionAdapter<O::Subscription>;
        fn subscribe(&self, o : Q) -> Self::Subscription {
            let observer = FilterObserver { f: self.f.clone(), o: o, _t: PhantomData };
            SubscriptionAdapter::<O::Subscription>::new(self.inner.subscribe(observer))
        }
    }
    // add code here
}

pub mod map {
    use std::marker::PhantomData;
    use super::{Observer, Observable, SubscriptionAdapter};
    pub struct Map<T, S, Inner, F: Fn(T) -> S> { f : F, inner : Inner, _t:PhantomData<T>, _s:PhantomData<S> }
    struct MapObserver<T, S, O : Observer<S>, F : Fn(T) -> S> { f : F, o : O, _t : PhantomData<T>, _s : PhantomData<S> }

    impl<T, S, O: Observer<S>, F : Fn(T) -> S> Observer<T> for MapObserver<T, S, O, F> {
        fn on_next(self, value : T) -> Self {
            let value = (self.f)(value);
            MapObserver { f: self.f, o: self.o.on_next(value), _t: self._t, _s: self._s}
        }
        fn on_completed(self) {
            self.o.on_completed();
        }
    }

    impl<T, S, Q : Observer<S>, O : Observable<T, MapObserver<T, S, Q, F>>, F : Fn(T) -> S + Clone> Observable<S, Q> for Map<T, S, O, F> {
        type Subscription = SubscriptionAdapter<O::Subscription>;
        fn subscribe(&self, o : Q) -> Self::Subscription {
            let observer = MapObserver { f: self.f.clone(), o : o, _t : PhantomData, _s : PhantomData };
            SubscriptionAdapter::<O::Subscription>::new(self.inner.subscribe(observer))
        }
    }
}
