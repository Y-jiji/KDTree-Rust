use std::fmt::Debug;

pub(crate)
fn pdist<const K: usize>(x: [f32; K], y: [f32; K], p: f32) -> f32 {
    let mut dist = 0f32;
    for i in 0..K
    { dist += (y[i] - x[i]).abs().powf(p); }
    return dist.powf(1f32/p);
}

#[derive(Debug, Clone)]
pub(crate)
struct BoundBox<const K: usize> {
    /// distance between corner and center
    diff: [f32; K],
    /// the center point
    cent: [f32; K],
}

impl<const K: usize> BoundBox<K> {
    pub(crate)
    fn cdist(&self, x: [f32; K], p: f32) -> f32 {
        pdist(x, self.cent, p) - 
        pdist([0f32; K], self.diff, p)
    }
    pub(crate)
    fn from_vec(data: &Vec<[f32; K]>) -> Self {
        let mut max_corner = [f32::MIN; K];
        let mut min_corner = [f32::MAX; K];
        for x in data {
            for i in 0..K {
                max_corner[i] = f32::max(max_corner[i], x[i]);
                min_corner[i] = f32::min(min_corner[i], x[i]);
            }
        }
        let mut diff = [0f32; K];
        let mut cent = [0f32; K];
        for i in 0..K {
            diff[i] = (max_corner[i] - min_corner[i]) / 2f32;
            cent[i] = (max_corner[i] + min_corner[i]) / 2f32;
        }
        Self { diff, cent }
    }
}

#[derive(Debug, Clone)]
pub(crate)
struct OrdT<T: Debug>(pub f32, pub T);

impl<T: Debug> PartialEq for OrdT<T> {
    fn eq(&self, other: &Self) -> bool {self.0 == other.0}
    fn ne(&self, other: &Self) -> bool {self.0 != other.0}
}

impl<T: Debug> PartialOrd for OrdT<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.0.is_nan() { panic!("{self:?} is nan"); }
        if other.0.is_nan() { panic!("{other:?} is nan"); }
        if self.0 < other.0 { return Some(std::cmp::Ordering::Less) }
        else if self.0 == other.0 { return Some(std::cmp::Ordering::Equal) }
        else { return Some(std::cmp::Ordering::Greater) }
    }
}

impl <T: Debug> Eq for OrdT<T> {
    fn assert_receiver_is_total_eq(&self) {assert!(!self.0.is_nan());}
}

impl<T: Debug> Ord for OrdT<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {self.partial_cmp(other).unwrap()}
}