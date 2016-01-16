
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

pub struct SubscriptionAdapter<Inner : Drop> { _inner_subscription : Inner }
impl<Inner : Drop> Drop for SubscriptionAdapter<Inner> {
    fn drop(&mut self) {}
}
impl<Inner: Drop> SubscriptionAdapter<Inner> {
    pub fn new(inner : Inner) -> Self {
        SubscriptionAdapter { _inner_subscription: inner }
    }
}
pub mod filter {
    use super::{Observer, Observable, Subscribable, SubscriptionAdapter};
    pub struct Filter<Inner : Observable, F : Fn(&Inner::Item) -> bool> { f : F, inner : Inner }
    impl<O : Observable, F: Fn(&O::Item) ->bool> Observable for Filter<O,F> { type Item = O::Item; }
    pub struct FilterObserver<O : Observer, F : Fn(&O::Item) -> bool> { f : F, o : O }

    impl<O : Observer, F : Fn(&O::Item) -> bool> Observer for FilterObserver<O, F> {
        type Item = O::Item;
        fn on_next(self, value : O::Item) -> Self {
            if (self.f)(&value) {
                FilterObserver { o: self.o.on_next(value), ..self }
            } else { self }
        }
        fn on_completed(self) {
            self.o.on_completed();
        }
    }

    impl<Q : Observer<Item = O::Item>, O : Observable + Subscribable<FilterObserver<Q, F>>, F : Fn(&O::Item)->bool> Subscribable<Q> for Filter<O,F> {
        type Subscription = SubscriptionAdapter<O::Subscription>;
        fn subscribe(self, o : Q) -> Self::Subscription {
            let observer = FilterObserver { f: self.f, o: o };
            SubscriptionAdapter::<O::Subscription>::new(self.inner.subscribe(observer))
        }
    }
    pub fn new<O : Observable, F : Fn(&O::Item) -> bool>(seq : O, f : F) -> Filter<O,F> {
        
        Filter { inner: seq, f: f }
    }
    // add code here
}

pub fn filter<O, F>(seq : O, f : F) -> filter::Filter<O, F>
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
    pub struct MapObserver<T, O : Observer, F : Fn(T) -> O::Item> { f : F, o : O, _t : PhantomData<T> }

    impl<T, O: Observer, F : Fn(T) -> O::Item> Observer for MapObserver<T, O, F> {
        type Item = T;
        fn on_next(self, value : T) -> Self {
            let value = (self.f)(value);
            MapObserver { o: self.o.on_next(value), ..self }
        }
        fn on_completed(self) {
            self.o.on_completed();
        }
    }

    impl<T, Q : Observer, O : Observable<Item = T> + Subscribable<MapObserver<T, Q, F>>, F : Fn(O::Item) -> Q::Item> Subscribable<Q> for Map<Q::Item, O, F> {
        type Subscription = SubscriptionAdapter<O::Subscription>;
        fn subscribe(self, o : Q) -> Self::Subscription {
            let observer = MapObserver { f: self.f, o: o, _t: PhantomData };
            SubscriptionAdapter::<O::Subscription>::new(self.inner.subscribe(observer))
        }
    }
}

#[cfg(test)]
pub mod test_source {
    use std::iter::IntoIterator;
    use super::{Observer, Observable, Subscribable};

    pub struct TestSequence<I : IntoIterator> { it : I }
    impl<I : IntoIterator> Observable for TestSequence<I> { type Item = I::Item; }

    pub struct Sub;
    impl Drop for Sub { fn drop(&mut self) {} }

    pub fn from_iter<I: IntoIterator>(it : I) -> TestSequence<I> { TestSequence { it: it } }

    impl<I : IntoIterator, Q : Observer<Item = I::Item>> Subscribable<Q> for TestSequence<I> {
        type Subscription = Sub;
        fn subscribe(self, o : Q) -> Self::Subscription {
            let mut o = o;
            for x in self.it {
                o = o.on_next(x);
            }
            o.on_completed();
            Sub
        }
    }

}


/*impl<T, Q: Observer<T>, O : Subscribable<T,Q>> O {
	fn filter(self, predicate : Fn(&T) -> bool) -> Filter::Filter<T,Self> {
		Filter { f: predicate, inner: self }
	}
}*/