use super::{Board, Layer, Physical, Time};

/** Tuple struct containing a set of coordinates and some utility functions. **/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coords(pub Layer, pub Time, pub Physical, pub Physical); // ⟨l, t, x, y⟩

impl Coords {
    /** Creates a new `Coords` instance. **/
    #[inline]
    pub fn new(l: Layer, t: Time, x: Physical, y: Physical) -> Self {
        Self(l, t, x, y)
    }

    /** Returns the physical half of a `Coords` instance (3rd and 4th element). **/
    #[inline]
    pub fn physical(self) -> (Physical, Physical) {
        (self.2, self.3)
    }

    /** Returns the non-physical half of a `Coords` instance (1st and 2nd element). **/
    #[inline]
    pub fn non_physical(self) -> (Layer, Time) {
        (self.0, self.1)
    }

    /** Returns the layer component of a `Coords` instance. **/
    #[inline]
    pub fn l(self) -> Layer {
        self.0
    }

    /** Returns the time component of a `Coords` instance. **/
    #[inline]
    pub fn t(self) -> Time {
        self.1
    }

    /** Returns the horizontal component of a `Coords` instance. **/
    #[inline]
    pub fn x(self) -> Physical {
        self.2
    }

    /** Returns the vertical component of a `Coords` instance. **/
    #[inline]
    pub fn y(self) -> Physical {
        self.3
    }
}

impl From<((Layer, Time), (Physical, Physical))> for Coords {
    #[inline]
    fn from(((l, t), (x, y)): ((Layer, Time), (Physical, Physical))) -> Self {
        Self(l, t, x, y)
    }
}

impl From<(Layer, Time, Physical, Physical)> for Coords {
    #[inline]
    fn from((l, t, x, y): (Layer, Time, Physical, Physical)) -> Self {
        Self(l, t, x, y)
    }
}

impl From<(isize, isize, isize, isize)> for Coords {
    #[inline]
    fn from((l, t, x, y): (isize, isize, isize, isize)) -> Self {
        Self(l as Layer, t as Time, x as Physical, y as Physical)
    }
}

impl From<(Board, (Physical, Physical))> for Coords {
    #[inline]
    fn from((board, (x, y)): (Board, (Physical, Physical))) -> Self {
        Self(board.l(), board.t(), x, y)
    }
}

impl std::ops::Add<Coords> for Coords {
    type Output = Coords;

    #[inline]
    fn add(self, w: Coords) -> Coords {
        Self(self.0 + w.0, self.1 + w.1, self.2 + w.2, self.3 + w.3)
    }
}

impl std::ops::Sub<Coords> for Coords {
    type Output = Coords;

    #[inline]
    fn sub(self, w: Coords) -> Coords {
        Self(self.0 - w.0, self.1 - w.1, self.2 - w.2, self.3 - w.3)
    }
}

impl std::ops::Mul<isize> for Coords {
    type Output = Coords;

    #[inline]
    fn mul(self, w: isize) -> Coords {
        Self(
            self.0 * w as Layer,
            self.1 * w as Time,
            self.2 * w as Physical,
            self.3 * w as Physical,
        )
    }
}
