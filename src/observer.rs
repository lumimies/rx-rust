pub trait Observer : Sized{
    type Item;
	fn on_next(self, value : Self::Item) -> Option<Self>;
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

pub mod filter {
    use super::{Observer, Observable, Subscribable};
    pub struct Filter<Inner : Observable, F : Fn(&Inner::Item) -> bool> { f : F, inner : Inner }
    impl<O : Observable, F: Fn(&O::Item) ->bool> Observable for Filter<O,F> { type Item = O::Item; }
    pub struct FilterObserver<O : Observer, F : Fn(&O::Item) -> bool> { f : F, o : O }

    impl<O : Observer, F : Fn(&O::Item) -> bool> Observer for FilterObserver<O, F> {
        type Item = O::Item;
        fn on_next(self, value : O::Item) -> Option<Self> {
            if (self.f)(&value) {
                let f = self.f;
                self.o.on_next(value).map(|o| { FilterObserver { o: o, f: f }})
            } else { Some(self) }
        }
        fn on_completed(self) {
            self.o.on_completed();
        }
    }

    impl<Q : Observer<Item = O::Item>, O : Observable + Subscribable<FilterObserver<Q, F>>, F : Fn(&O::Item)->bool> Subscribable<Q> for Filter<O,F> {
        type Subscription = O::Subscription;
        fn subscribe(self, o : Q) -> Self::Subscription {
            let observer = FilterObserver { f: self.f, o: o };
            self.inner.subscribe(observer)
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
    use super::{Observer, Observable, Subscribable};
    pub struct Map<T, Inner : Observable, F: Fn(Inner::Item) -> T> { f : F, inner : Inner, _t:PhantomData<T> }
    impl<T, Inner : Observable, F: Fn(Inner::Item) -> T> Observable for Map<T,Inner,F> {
        type Item = T;
    }
    pub struct MapObserver<T, O : Observer, F : Fn(T) -> O::Item> { f : F, o : O, _t : PhantomData<T> }

    impl<T, O: Observer, F : Fn(T) -> O::Item> Observer for MapObserver<T, O, F> {
        type Item = T;
        fn on_next(self, value : T) -> Option<Self> {
            let value = (self.f)(value);
            let f = self.f;
            self.o.on_next(value).map(|o| { MapObserver { o: o, f: f, _t: PhantomData } })
        }
        fn on_completed(self) {
            self.o.on_completed();
        }
    }

    impl<T, Q : Observer, O : Observable<Item = T> + Subscribable<MapObserver<T, Q, F>>, F : Fn(O::Item) -> Q::Item> Subscribable<Q> for Map<Q::Item, O, F> {
        type Subscription = O::Subscription;
        fn subscribe(self, o : Q) -> Self::Subscription {
            let observer = MapObserver { f: self.f, o: o, _t: PhantomData };
            self.inner.subscribe(observer)
        }
    }
}
pub struct DoNothingSub;
impl Drop for DoNothingSub {
    fn drop(&mut self) {}
}
pub mod never {
    use std::marker::PhantomData;
    use super::{Observable, Observer, Subscribable,DoNothingSub};
    pub struct Never<T> { _t: PhantomData<T> }
    impl<T> Observable for Never<T> {
        type Item = T;
    }
    impl<T, Q : Observer<Item = T>> Subscribable<Q> for Never<T> {
        type Subscription = DoNothingSub;
        fn subscribe(self, _o : Q) -> Self::Subscription {
            DoNothingSub
        }
    }
    pub fn new<T>() -> Never<T> { Never { _t: PhantomData } }
}
pub fn never<T>() -> never::Never<T> { 
    never::new::<T>()
}

pub mod empty {
    use std::marker::PhantomData;
    use super::{Observable, Observer, Subscribable,DoNothingSub};
    pub struct Empty<T> { _t: PhantomData<T> }
    impl<T> Observable for Empty<T> {
        type Item = T;
    }
    impl<T, Q : Observer<Item = T>> Subscribable<Q> for Empty<T> {
        type Subscription = DoNothingSub;
        fn subscribe(self, o : Q) -> Self::Subscription {
            o.on_completed();
            DoNothingSub
        }
    }
    pub fn new<T>() -> Empty<T> { Empty { _t: PhantomData } }
}
pub mod take {
    use super::{Observable, Observer, Subscribable};
    pub struct Take<Inner: Observable> { inner : Inner, count : i64 }
    pub struct TakeObserver<Q : Observer> { inner: Q, count : i64 }
    impl<Q : Observer> Observer for TakeObserver<Q> {
        type Item = Q::Item;
        fn on_next(mut self, val : Q::Item) -> Option<Self> {
            if self.count > 0 {
                let o =self.inner.on_next(val);
                if o.is_none() {
                    return None;
                }
                self.inner = o.unwrap();
                self.count -= 1;
                if self.count == 0 {
                    self.inner.on_completed();
                    return None;
                }
            }
            Some(self)
        }

        fn on_completed(self) {
            self.inner.on_completed();
        }
    }
    impl<Inner: Observable> Observable for Take<Inner> {
        type Item = Inner::Item;
    }

    impl<Q: Observer, Inner: Observable<Item = Q::Item> + Subscribable<TakeObserver<Q>>> Subscribable<Q> for Take<Inner> {
        type Subscription = Inner::Subscription;
        fn subscribe(self, observer: Q) -> Self::Subscription {
            self.inner.subscribe(TakeObserver { inner: observer, count: self.count })
        }
    }

}

pub mod skip {
    use super::{Observable, Observer, Subscribable};
    pub struct Skip<Inner : Observable> { inner: Inner, count: u64 }

    impl<Inner: Observable> Observable for Skip<Inner> {
        type Item = Inner::Item;
    }

    impl<Q: Observer, Inner: Observable<Item = Q::Item> + Subscribable<SkipObserver<Q>>> Subscribable<Q> for Skip<Inner> {
        type Subscription = Inner::Subscription;
        fn subscribe(self, observer: Q) -> Self::Subscription {
            self.inner.subscribe(SkipObserver { inner: observer, count: self.count })
        }
    }


    pub struct SkipObserver<Q : Observer> { inner: Q, count: u64 }
    impl<Q: Observer> Observer for SkipObserver<Q> {
        type Item = Q::Item;
        fn on_next(mut self, val: Q::Item) -> Option<Self> {
            if self.count == 0 {
                if let Some(next) = self.inner.on_next(val) {
                self.inner = next;
                } else {
                    return None;
                }
            } else {
                self.count -= 1;
            }
            Some(self)
        }

        fn on_completed(self) {
            self.inner.on_completed();
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
                if let Some(o2) = o.on_next(x) {
                    o = o2;
                } else {
                    return Sub;
                }
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