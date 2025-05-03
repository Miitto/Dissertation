#[derive(Clone, Copy, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn all() -> [Axis; 3] {
        [Axis::X, Axis::Y, Axis::Z]
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Dir {
    Left,
    Right,
    Up,
    Down,
    Forward,
    Backward,
}

impl Dir {
    pub fn all() -> [Dir; 6] {
        [
            Dir::Left,
            Dir::Right,
            Dir::Up,
            Dir::Down,
            Dir::Forward,
            Dir::Backward,
        ]
    }
}

impl From<Dir> for Axis {
    fn from(dir: Dir) -> Self {
        match dir {
            Dir::Forward | Dir::Backward => Axis::Z,
            Dir::Left | Dir::Right => Axis::X,
            Dir::Up | Dir::Down => Axis::Y,
        }
    }
}

impl From<Dir> for usize {
    fn from(dir: Dir) -> Self {
        match dir {
            Dir::Left => 0,
            Dir::Right => 1,
            Dir::Up => 2,
            Dir::Down => 3,
            Dir::Forward => 4,
            Dir::Backward => 5,
        }
    }
}

impl From<Axis> for usize {
    fn from(axis: Axis) -> Self {
        match axis {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }
}

impl From<usize> for Dir {
    fn from(i: usize) -> Self {
        match i {
            0 => Dir::Left,
            1 => Dir::Right,
            2 => Dir::Up,
            3 => Dir::Down,
            4 => Dir::Forward,
            5 => Dir::Backward,
            _ => panic!("Invalid Dir: {}", i),
        }
    }
}
