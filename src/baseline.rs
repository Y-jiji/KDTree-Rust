use std::{fmt::Debug, collections::BinaryHeap};
use crate::util::*;
use std::sync::Arc;

#[derive(Clone, Debug)]
#[cfg(test)]
pub(crate)
struct StupidKNN<const K: usize, T: Debug + Clone> {
    inner: Vec<([f32; K], T)>,
}

#[cfg(test)]
impl<const K: usize, T: Debug + Clone + 'static + Send + Sync> StupidKNN<K, T> {
    pub fn build(data: Vec<([f32; K], T)>) -> Self {
        StupidKNN { inner: data }
    }
    /// 暴力遍历, 用最大堆放置距离最小的k个元素
    pub fn search(&self, k: usize, x: [f32; K], p: f32) -> Vec<(f32, T)> {
        let d = &self.inner;
        let mut h = BinaryHeap::new();
        for (y, label) in d {
            h.push(OrdT(pdist(x, y.clone(), p), label.clone()));
            if h.len() > k {h.pop();}
        }
        h.into_iter().map(|OrdT(x, label)| (x, label)).collect()
    }
    pub fn batch_search(&self, k: usize, x: Vec<[f32; K]>, p: f32) -> Vec<Vec<(f32, T)>>
    where T: Send {
        const DIVIDE: usize = 8;
        let mut result = Vec::new();
        let arc_self = Arc::new(self.clone());
        let batch_sz = x.len();
        let arc_x = Arc::new(x);
        let mut handles = Vec::new();
        for i in 0..(DIVIDE-1) {
            let arc_self = arc_self.clone();
            let arc_x = arc_x.clone();
            let start = i * (batch_sz / DIVIDE);
            let end = (i+1) * (batch_sz / DIVIDE);
            let h = std::thread::spawn(move || {
                let mut result = Vec::new();
                for j in start..end
                { result.push(arc_self.search(k, arc_x[j], p)); }
                result
            });
            handles.push(h);
        }
        let start = (DIVIDE-1) * (batch_sz / DIVIDE);
        let end = batch_sz;
        let h = std::thread::spawn(move || {
            let mut result = Vec::new();
            for j in start..end 
            { result.push(arc_self.search(k, arc_x[j], p)); }
            result
        });
        handles.push(h);
        for h in handles { result.extend(h.join().unwrap()); }
        result
    }
}