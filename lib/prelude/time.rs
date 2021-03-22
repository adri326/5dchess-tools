use std::time::{Duration, Instant};

pub trait TimedFilterTrait: Iterator {
    /**
        Returns a filtered iterator, which only yields the elements of the current iterator if:
        - `condition(item) == true`
        - The time since the first call to `next()` does not exceeds `duration`

        See `TimedFilter` for more information.
    **/
    fn filter_timed<F>(self, condition: F, duration: Duration) -> TimedFilter<Self, F>
    where
        for<'b> F: Fn(&'b Self::Item) -> bool,
        Self: Sized,
    {
        TimedFilter::new(self, condition, duration)
    }

    /**
        Returns an iterator, which stops when the time since the first call to `next()` exceeds `duration`.

        See `Timed` for more information.
    **/
    fn timed(self, duration: Duration) -> Timed<Self>
    where
        Self: Sized,
    {
        Timed::new(self, duration)
    }

    /**
        Returns a filtered iterator, which only yields the elements of the current iterator if:
        - `condition(item) == true`
        - The sum of the time taken by `condition` does not exceed `duration`

        See `SigmaFilter` for more information.
    **/
    fn filter_sigma<F>(self, condition: F, duration: Duration) -> SigmaFilter<Self, F>
    where
        for<'b> F: Fn(&'b Self::Item) -> bool,
        Self: Sized,
    {
        SigmaFilter::new(self, condition, duration)
    }

    /**
        Returns an iterator, which stops when the sum of the time taken by the underlying iterator exceeds `duration`.

        See `Sigma` for more information.
    **/
    fn sigma(self, duration: Duration) -> Sigma<Self>
    where
        Self: Sized,
    {
        Sigma::new(self, duration)
    }
}

impl<T: ?Sized> TimedFilterTrait for T where T: Iterator {}

/**
    A variant of `Iterator::filter`, which has a maximum duration.
    It measures the elapsed time from the first call to `next` and stops when the maximum duration is reached.

    You should create instances of it by calling the `filter_timed` function on any iterator, given that `TimedFilterTrait` is
    loaded in your context (it is by default included in `prelude::*`).

    Note that this iterator does *not* measure the summation of the time taken by the filter function, but instead naively measures
    the elapsed time since the first call to `next`.
    If you wish to have the former, use `SigmaFilter` instead.
**/
#[derive(Clone)]
pub struct TimedFilter<J, F>
where
    J: Iterator,
    for<'b> F: Fn(&'b J::Item) -> bool,
{
    pub iterator: J,
    pub condition: F,
    pub start: Option<Instant>,
    pub duration: Duration,
}

impl<J, F> TimedFilter<J, F>
where
    J: Iterator,
    F: for<'b> Fn(&'b J::Item) -> bool,
{
    /** Creates a new TimedFilter iterator, with the given duration. **/
    pub fn new(iterator: J, condition: F, duration: Duration) -> Self {
        Self {
            iterator,
            condition,
            start: None,
            duration,
        }
    }

    /** Creates a new TimedFilter iterator, with a given duration and start. **/
    pub fn with_start(
        iterator: J,
        condition: F,
        start: Option<Instant>,
        duration: Duration,
    ) -> Self {
        Self {
            iterator,
            condition,
            start,
            duration,
        }
    }

    /** Returns the time elapsed from the start of the timer, or None if the timer hasn't started yet. **/
    pub fn elapsed(&self) -> Option<Duration> {
        match self.start {
            Some(instant) => Some(instant.elapsed()),
            None => None,
        }
    }

    /** Returns the remaining time. **/
    pub fn remaining(&self) -> Option<Duration> {
        match self.start {
            Some(instant) => {
                let elapsed = instant.elapsed();
                if elapsed > self.duration {
                    Some(Duration::new(0, 0))
                } else {
                    Some(self.duration - elapsed)
                }
            }
            None => None,
        }
    }

    /** Returns true if the iterator timed out. The iterator will always return None if this value is true. **/
    pub fn timed_out(&self) -> bool {
        match self.start {
            Some(instant) => instant.elapsed() > self.duration,
            None => false,
        }
    }
}

impl<J, F> Iterator for TimedFilter<J, F>
where
    J: Iterator,
    F: for<'b> Fn(&'b J::Item) -> bool,
{
    type Item = J::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start.is_none() {
            self.start = Some(Instant::now());
        }

        loop {
            if self.start.unwrap().elapsed() > self.duration {
                return None;
            }
            match self.iterator.next() {
                Some(item) => {
                    if (self.condition)(&item) {
                        return Some(item);
                    }
                }
                None => return None,
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.remaining() {
            Some(d) => {
                if d == Duration::new(0, 0) {
                    return (0, Some(0));
                }
            }
            _ => {}
        }

        (0, self.iterator.size_hint().1)
    }
}


/**
    An iterator, which has a maximum duration.
    It measures the elapsed time from the first call to `next` and stops when the maximum duration is reached.

    You should create instances of it by calling the `timed` function on any iterator, given that `TimedFilterTrait` is
    loaded in your context (it is by default included in `prelude::*`).

    Note that this iterator does *not* measure the summation of the time taken by the filter function, but instead naively measures
    the elapsed time since the first call to `next`.
    If you wish to have the former, use `Sigma` instead.
**/
#[derive(Clone)]
pub struct Timed<J>
where
    J: Iterator,
{
    pub iterator: J,
    pub start: Option<Instant>,
    pub duration: Duration,
}

impl<J> Timed<J>
where
    J: Iterator,
{
    /** Creates a new Timed iterator, with the given duration. **/
    pub fn new(iterator: J, duration: Duration) -> Self {
        Self {
            iterator,
            start: None,
            duration,
        }
    }

    /** Creates a new Timed iterator, with a given duration and start. **/
    pub fn with_start(
        iterator: J,
        start: Option<Instant>,
        duration: Duration,
    ) -> Self {
        Self {
            iterator,
            start,
            duration,
        }
    }

    /** Returns the time elapsed from the start of the timer, or None if the timer hasn't started yet. **/
    pub fn elapsed(&self) -> Option<Duration> {
        match self.start {
            Some(instant) => Some(instant.elapsed()),
            None => None,
        }
    }

    /** Returns the remaining time. **/
    pub fn remaining(&self) -> Option<Duration> {
        match self.start {
            Some(instant) => {
                let elapsed = instant.elapsed();
                if elapsed > self.duration {
                    Some(Duration::new(0, 0))
                } else {
                    Some(self.duration - elapsed)
                }
            }
            None => None,
        }
    }

    /** Returns true if the iterator timed out. The iterator will always return None if this value is true. **/
    pub fn timed_out(&self) -> bool {
        match self.start {
            Some(instant) => instant.elapsed() > self.duration,
            None => false,
        }
    }
}

impl<J> Iterator for Timed<J>
where
    J: Iterator,
{
    type Item = J::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start.is_none() {
            self.start = Some(Instant::now());
        }

        if self.start.unwrap().elapsed() > self.duration {
            return None;
        }

        self.iterator.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.remaining() {
            Some(d) => {
                if d == Duration::new(0, 0) {
                    return (0, Some(0));
                }
            }
            _ => {}
        }

        (0, self.iterator.size_hint().1)
    }
}

/**
    A variant of `Iterator::filter`, which limits the time that the filter function may take.
    It measures the sum of the elapsed time taken by the filter function and stops once it exceeds the given, maximum duration.

    You should create instances of it by calling the `filter_sigma` function on any iterator, given that `TimedFilterTrait` is
    loaded in your context (it is by default included in `prelude::*`).

    Note that this iterator does *not* measure the elapsed time since the first call to `next`, but instead the summation of the time taken by the filter function.

    If you wish to have the former, use `TimedFilter` instead.
**/
#[derive(Clone)]
pub struct SigmaFilter<J, F>
where
    J: Iterator,
    for<'b> F: Fn(&'b J::Item) -> bool,
{
    pub iterator: J,
    pub condition: F,
    pub sigma: Duration,
    pub duration: Duration,
}

impl<J, F> SigmaFilter<J, F>
where
    J: Iterator,
    F: for<'b> Fn(&'b J::Item) -> bool,
{

    /** Creates a new SigmaFilter iterator, with the given duration. **/
    pub fn new(iterator: J, condition: F, duration: Duration) -> Self {
        Self {
            iterator,
            condition,
            sigma: Duration::new(0, 0),
            duration,
        }
    }


    /** Creates a new SigmaFilter iterator, with a given duration and start. **/
    pub fn with_sigma(iterator: J, condition: F, sigma: Duration, duration: Duration) -> Self {
        Self {
            iterator,
            condition,
            sigma,
            duration,
        }
    }

    /** Returns the time elapsed by the filter function and the iterator. **/
    pub fn elapsed(&self) -> Duration {
        self.sigma
    }


    /** Returns the remaining time. **/
    pub fn remaining(&self) -> Duration {
        if self.sigma > self.duration {
            Duration::new(0, 0)
        } else {
            self.duration - self.sigma
        }
    }


    /** Returns true if the iterator timed out. The iterator will always return None if this value is true. **/
    pub fn timed_out(&self) -> bool {
        self.sigma > self.duration
    }
}

impl<J, F> Iterator for SigmaFilter<J, F>
where
    J: Iterator,
    F: for<'b> Fn(&'b J::Item) -> bool,
{
    type Item = J::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let start = Instant::now();

        let res = loop {
            if self.sigma + start.elapsed() > self.duration {
                break None;
            }
            match self.iterator.next() {
                Some(item) => {
                    if (self.condition)(&item) {
                        break Some(item);
                    }
                }
                None => break None,
            }
        };

        self.sigma += start.elapsed();
        res
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.sigma > self.duration {
            (0, Some(0))
        } else {
            (0, self.iterator.size_hint().1)
        }
    }
}

/**
    An iterator, which limits the time that the iterator may take.
    It measures the sum of the elapsed time taken by the filter function and stops once it exceeds the given, maximum duration.

    You should create instances of it by calling the `sigma` function on any iterator, given that `TimedFilterTrait` is
    loaded in your context (it is by default included in `prelude::*`).

    Note that this iterator does *not* measure the elapsed time since the first call to `next`, but instead the summation of the time taken by the filter function.

    If you wish to have the former, use `Timed` instead.
**/
#[derive(Clone)]
pub struct Sigma<J>
where
    J: Iterator,
{
    pub iterator: J,
    pub sigma: Duration,
    pub duration: Duration,
}

impl<J> Sigma<J>
where
    J: Iterator,
{
    /** Creates a new SigmaFilter iterator, with the given duration. **/
    pub fn new(iterator: J, duration: Duration) -> Self {
        Self {
            iterator,
            sigma: Duration::new(0, 0),
            duration,
        }
    }

    /** Creates a new SigmaFilter iterator, with a given duration and start. **/
    pub fn with_sigma(iterator: J, sigma: Duration, duration: Duration) -> Self {
        Self {
            iterator,
            sigma,
            duration,
        }
    }

    /** Returns the time taken by the iterator. **/
    pub fn elapsed(&self) -> Duration {
        self.sigma
    }

    /** Returns the remaining time. **/
    pub fn remaining(&self) -> Duration {
        if self.sigma > self.duration {
            Duration::new(0, 0)
        } else {
            self.duration - self.sigma
        }
    }

    /** Returns true if the iterator timed out. The iterator will always return None if this value is true. **/
    pub fn timed_out(&self) -> bool {
        self.sigma > self.duration
    }
}

impl<J> Iterator for Sigma<J>
where
    J: Iterator,
{
    type Item = J::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let start = Instant::now();

        if self.sigma > self.duration {
            return None
        }

        let res = self.iterator.next();

        self.sigma += start.elapsed();
        res
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.sigma > self.duration {
            (0, Some(0))
        } else {
            (0, self.iterator.size_hint().1)
        }
    }
}
