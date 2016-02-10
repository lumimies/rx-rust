pub trait Observer : Sized{
    type Item;
	fn on_next(self, value: Self::Item) -> Option<Self>;
	fn on_completed(self);
	// Not sure how the error stuff looks in Rust
	// fn on_error(Self/*,???*/);
}

pub trait Subscribable<Q: Observer<Item = Self::Item>> : Observable {
    type Subscription : Drop;
	fn subscribe(self, o: Q) -> Self::Subscription;
}

pub trait Observable: Sized {
    type Item;

    fn filter<F: FnMut(&Self::Item) -> bool>(self, f: F) -> filter::Filter<Self, F> {
        filter::new(self, f)
    }
}

pub mod filter {
    use super::{Observer, Observable, Subscribable};
    pub struct Filter<Inner: Observable, F: FnMut(&Inner::Item) -> bool> { f: F, inner: Inner }
    impl<O: Observable, F: FnMut(&O::Item) ->bool> Observable for Filter<O,F> { type Item = O::Item; }
    pub struct FilterObserver<Q: Observer, F: FnMut(&Q::Item) -> bool> { f: F, inner: Q }

    impl<Q: Observer, F: FnMut(&Q::Item) -> bool> Observer for FilterObserver<Q, F> {
        type Item = Q::Item;
        fn on_next(mut self, value: Q::Item) -> Option<Self> {
            if (self.f)(&value) {
                let f = self.f;
                self.inner.on_next(value).map(|o| { FilterObserver { inner: o, f: f }})
            } else { Some(self) }
        }
        fn on_completed(self) {
            self.inner.on_completed();
        }
    }

    impl<Q: Observer<Item = O::Item>, O: Subscribable<FilterObserver<Q, F>>, F: FnMut(&O::Item)->bool> Subscribable<Q> for Filter<O, F> {
        type Subscription = O::Subscription;
        fn subscribe(self, o: Q) -> Self::Subscription {
            let observer = FilterObserver { f: self.f, inner: o };
            self.inner.subscribe(observer)
        }
    }
    pub fn new<O: Observable, F: FnMut(&O::Item) -> bool>(seq: O, f: F) -> Filter<O, F> {
        
        Filter { inner: seq, f: f }
    }
    // add code here
}

pub fn filter<O, F>(seq: O, f: F) -> filter::Filter<O, F>
    where O: Observable, F: FnMut(&O::Item) -> bool {
    filter::new(seq,f)
}

pub mod map {
    use std::marker::PhantomData;
    use super::{Observer, Observable, Subscribable};
    pub struct Map<T, Inner: Observable, F: FnMut(Inner::Item) -> T> { f: F, inner: Inner, _t: PhantomData<T> }
    impl<T, Inner: Observable, F: FnMut(Inner::Item) -> T> Observable for Map<T, Inner, F> {
        type Item = T;
    }
    pub struct MapObserver<T, Q: Observer, F : FnMut(T) -> Q::Item> { f: F, inner: Q, _t: PhantomData<T> }

    impl<T, Q: Observer, F: FnMut(T) -> Q::Item> Observer for MapObserver<T, Q, F> {
        type Item = T;
        fn on_next(self, value: T) -> Option<Self> {
            let mut f = self.f;
            let value = f(value);
            self.inner.on_next(value).map(|o| { MapObserver { inner: o, f: f, _t: PhantomData } })
        }
        fn on_completed(self) {
            self.inner.on_completed();
        }
    }

    impl<T, Q: Observer, O: Observable<Item = T> + Subscribable<MapObserver<T, Q, F>>, F: FnMut(O::Item) -> Q::Item> Subscribable<Q> for Map<Q::Item, O, F> {
        type Subscription = O::Subscription;
        fn subscribe(self, o: Q) -> Self::Subscription {
            let observer = MapObserver { f: self.f, inner: o, _t: PhantomData };
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
    use super::{Observable, Observer, Subscribable, DoNothingSub};
    pub struct Never<T> { _t: PhantomData<T> }
    impl<T> Observable for Never<T> {
        type Item = T;
    }
    impl<T, Q: Observer<Item = T>> Subscribable<Q> for Never<T> {
        type Subscription = DoNothingSub;
        fn subscribe(self, _o: Q) -> Self::Subscription {
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
    impl<T, Q: Observer<Item = T>> Subscribable<Q> for Empty<T> {
        type Subscription = DoNothingSub;
        fn subscribe(self, o: Q) -> Self::Subscription {
            o.on_completed();
            DoNothingSub
        }
    }
    pub fn new<T>() -> Empty<T> { Empty { _t: PhantomData } }
}
pub mod take {
    use super::{Observable, Observer, Subscribable};
    pub struct Take<Inner: Observable> { inner: Inner, count: i64 }
    pub struct TakeObserver<Q: Observer> { inner: Q, count: i64 }
    impl<Q: Observer> Observer for TakeObserver<Q> {
        type Item = Q::Item;
        fn on_next(mut self, val: Q::Item) -> Option<Self> {
            if self.count > 0 {
                let o = self.inner.on_next(val);
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

pub mod take_while {
    use super::{Observable, Observer, Subscribable};
    pub struct TakeWhile<Inner: Observable, F: FnMut(&Inner::Item) -> bool> { inner: Inner, f: F }
    pub struct TakeWhileObserver<Q: Observer, F: FnMut(&Q::Item) -> bool> { inner: Q, f: F }
    impl<Q: Observer, F: FnMut(&Q::Item) -> bool> Observer for TakeWhileObserver<Q, F> {
        type Item = Q::Item;
        fn on_next(mut self, val: Q::Item) -> Option<Self> {
            if (self.f)(&val) {
                if let Some(o) = self.inner.on_next(val) {
                    self.inner = o;
                    Some(self)
                } else {
                    None
                }
            } else {
                self.inner.on_completed();
                None
            }
        }

        fn on_completed(self) {
            self.inner.on_completed();
        }
    }
    impl<Inner: Observable, F: FnMut(&Inner::Item) -> bool> Observable for TakeWhile<Inner, F> {
        type Item = Inner::Item;
    }

    impl<Q: Observer, F: FnMut(&Inner::Item) -> bool, Inner: Observable<Item = Q::Item> + Subscribable<TakeWhileObserver<Q, F>>> Subscribable<Q> for TakeWhile<Inner, F> {
        type Subscription = Inner::Subscription;
        fn subscribe(self, observer: Q) -> Self::Subscription {
            self.inner.subscribe(TakeWhileObserver { inner: observer, f: self.f })
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


    pub struct SkipObserver<Q: Observer> { inner: Q, count: u64 }
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

pub mod skip_while {
    use super::{Observable, Observer, Subscribable};
    pub struct SkipWhile<Inner: Observable, F: FnMut(&Inner::Item) -> bool> { inner: Inner, f: F }
    pub struct SkipWhileObserver<Q : Observer, F: FnMut(&Q::Item) -> bool> { inner: Q, f: Option<F> }

    impl<Inner: Observable, F: FnMut(&Inner::Item) -> bool> Observable for SkipWhile<Inner, F> {
        type Item = Inner::Item;
    }

    impl<Q: Observer, Inner: Observable<Item = Q::Item> + Subscribable<SkipWhileObserver<Q, F>>, F: FnMut(&Inner::Item) -> bool> Subscribable<Q> for SkipWhile<Inner, F> {
        type Subscription = Inner::Subscription;
        fn subscribe(self, observer: Q) -> Self::Subscription {
            self.inner.subscribe(SkipWhileObserver { inner: observer, f: Some(self.f) })
        }
    }
    
    impl<Q: Observer, F: FnMut(&Q::Item) -> bool> Observer for SkipWhileObserver<Q, F> {
        type Item = Q::Item;
        fn on_next(mut self, val: Q::Item) -> Option<Self> {
            if let Some(mut f) = self.f {
                if f(&val) {
                    self.f = Some(f);
                    return Some(self);
                }
                self.f = None;
            } 
            if let Some(next) = self.inner.on_next(val) {
                self.inner = next;
                Some(self)
            } else {
                None
            }
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

    pub struct TestSequence<I: IntoIterator> { it: I }
    impl<I: IntoIterator> Observable for TestSequence<I> { type Item = I::Item; }

    pub struct Sub;
    impl Drop for Sub { fn drop(&mut self) {} }

    pub fn from_iter<I: IntoIterator>(it: I) -> TestSequence<I> { TestSequence { it: it } }

    impl<I: IntoIterator, Q: Observer<Item = I::Item>> Subscribable<Q> for TestSequence<I> {
        type Subscription = Sub;
        fn subscribe(self, o: Q) -> Self::Subscription {
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
