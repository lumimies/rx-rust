
pub trait Observer {
    type Item;
	fn on_next(self, value : Self::Item) -> Self;
	fn on_completed(self);
	// Not sure how the error stuff looks in Rust
	// fn on_error(Self/*,???*/);
}

pub trait Subscribable<O : Observer<Item = Self::Item>> : Observable {
    type Subscription : Drop;
	fn subscribe(self, o : O) -> Self::Subscription;
}

pub trait Observable {
    type Item;
}

struct SubscriptionAdapter<Inner : Drop> { _inner_subscription : Inner }
impl<Inner : Drop> Drop for SubscriptionAdapter<Inner> {
    fn drop(&mut self) {}
}
impl<Inner: Drop> SubscriptionAdapter<Inner> {
    pub fn new(inner : Inner) -> Self {
        SubscriptionAdapter { _inner_subscription: inner }
    }
}
pub mod filter {
    use std::marker::PhantomData;
    use super::{Observer, Observable, Subscribable, SubscriptionAdapter};
    pub struct Filter<T, Inner, F : Fn(&T) -> bool> { f : F, inner : Inner, _t : PhantomData<T> }
    impl<T, O, F: Fn(&T) ->bool> Observable for Filter<T,O,F> { type Item = T; }
    struct FilterObserver<T,O : Observer<Item = T>, F : Fn(&T) -> bool> { f : F, o : O, _t : PhantomData<T> }

    impl<T, O : Observer<Item = T>, F : Fn(&T) -> bool> Observer for FilterObserver<T, O, F> {
        type Item = T;
        fn on_next(self, value : T) -> Self {
            if (self.f)(&value) {
                FilterObserver { o: self.o.on_next(value), ..self }
            } else { self }
        }
        fn on_completed(self) {
            self.o.on_completed();
        }
    }

    impl<T, Q : Observer<Item = T>, O : Observable<Item = T> + Subscribable<FilterObserver<T, Q, F>>, F : Fn(&T)->bool> Subscribable<Q> for Filter<T,O,F> {
        type Subscription = SubscriptionAdapter<O::Subscription>;
        fn subscribe(self, o : Q) -> Self::Subscription {
            let observer = FilterObserver { f: self.f, o: o, _t: PhantomData };
            SubscriptionAdapter::<O::Subscription>::new(self.inner.subscribe(observer))
        }
    }
    pub fn new<O : Observable, F : Fn(&O::Item) -> bool>(seq : O, f : F) -> Filter<O::Item,O,F> {
        
        Filter { inner: seq, f: f, _t: PhantomData}
    }
    // add code here
}

pub fn filter<O, F>(seq : O, f : F) -> filter::Filter<O::Item, O, F>
    where O : Observable, F : Fn(&O::Item) -> bool {
    filter::new(seq,f)
}

pub mod map {
    use std::marker::PhantomData;
    use super::{Observer, Observable, Subscribable, SubscriptionAdapter};
    pub struct Map<T, Inner : Observable, F: Fn(Inner::Item) -> T> { f : F, inner : Inner, _t:PhantomData<T> }
    impl<T, Inner : Observable, F: Fn(Inner::Item) -> T> Observable for Map<T,Inner,F> {
        type Item = T;
    }
    struct MapObserver<T, S, O : Observer<Item = S>, F : Fn(T) -> S> { f : F, o : O, _t : PhantomData<T>, _s : PhantomData<S> }

    impl<T, S, O: Observer<Item = S>, F : Fn(T) -> S> Observer for MapObserver<T, S, O, F> {
        type Item = T;
        fn on_next(self, value : T) -> Self {
            let value = (self.f)(value);
            MapObserver { f: self.f, o: self.o.on_next(value), _t: self._t, _s: self._s}
        }
        fn on_completed(self) {
            self.o.on_completed();
        }
    }

    impl<T, Q : Observer<Item = T>, O : Observable<Item = T> + Subscribable<MapObserver<Q::Item, T, Q, F>>, F : Fn(O::Item) -> T> Subscribable<Q> for Map<T, O, F> {
        type Subscription = SubscriptionAdapter<O::Subscription>;
        fn subscribe(self, o : Q) -> Self::Subscription {
            let observer = MapObserver { f: self.f, o: o, _t: PhantomData, _s: PhantomData };
            SubscriptionAdapter::<O::Subscription>::new(self.inner.subscribe(observer))
        }
    }
}

#[cfg(test)]
pub mod test_source {
    use std::sync::{Arc,Weak};
    use std::iter::Iterator;
    use super::{Observer, Observable, Subscribable};
    pub struct TestSequence<I : Iterator> { it : I }
    impl<I : Iterator> Observable for TestSequence<I> { type Item = I::Item; }
    struct Sub { _p : Arc<()> }
    impl Drop for Sub { fn drop(&mut self) {} }
    pub fn from_iter<I: Iterator>(it : I) -> TestSequence<I> { TestSequence { it: it } }
    impl<I : Iterator, Q : Observer<Item = I::Item>> Subscribable<Q> for TestSequence<I> {
        type Subscription = Sub;
        fn subscribe(self, o : Q) -> Self::Subscription {
            let mut o = o;
            for x in self.it {
                o = o.on_next(x);
            }
            o.on_completed();
            Sub { _p: Arc::new(()) }
        }
    }

}


/*impl<T, Q: Observer<T>, O : Subscribable<T,Q>> O {
	fn filter(self, predicate : Fn(&T) -> bool) -> Filter::Filter<T,Self> {
		Filter { f: predicate, inner: self }
	}
}*/