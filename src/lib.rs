use std::{fmt::Debug, collections::{BinaryHeap, VecDeque}};
use std::sync::Arc;

mod util;
use util::*;

use rand::*;

#[derive(Debug, Clone)]
pub struct KDTree<const K: usize, T> {
    bbox: BoundBox<K>,
    l: Option<Box<KDTree<K, T>>>,
    r: Option<Box<KDTree<K, T>>>,
    pdiv: [f32; K],
    meta: T,
}

unsafe impl<const K: usize, T> Send for KDTree<K, T> {}
unsafe impl<const K: usize, T> Sync for KDTree<K, T> {}

impl<const K: usize, T: Debug + Clone + Send> KDTree<K, T> 
where T: 'static {
    /// 构建一棵KD树
    pub fn build(mut data: Vec<([f32; K], T)>) -> Option<Box<Self>> {
        if data.is_empty() { return None; }
        let t = thread_rng().gen::<usize>() % K;
        data.sort_by(|x, y| { x.0[t].partial_cmp(&y.0[t]).unwrap() });
        let bbox = BoundBox::from_vec(&data.iter().map(|x| x.0).collect());
        let idiv = data.len() / 2;
        let (pdiv, meta) = data[idiv].clone();
        let (ldata, rdata) = data.split_at(idiv);
        let (_    , rdata) = rdata.split_at(1);
        let l = Self::build(ldata.to_vec());
        let r = Self::build(rdata.to_vec());
        Some(Box::new(Self { bbox, l, r, pdiv, meta }))
    }
    /// 搜索: 
    /// 维护一个大小为k的最大堆, 堆顶是当前看过的点中离输入第k远的点, 和该点与输入的距离
    /// 如果一个块中所有点都比这个距离大, 那么这个块中任意一个点都不可能是k近邻, 可以不搜索这个块中的点
    /// 而由三角不等式, 很容易得到某一下界的表达式
    fn search_bfs_impl(&self, 
        k: usize, 
        x: [f32; K], 
        p: f32
    ) -> BinaryHeap<OrdT<T>> {
        let mut q = VecDeque::new();
        let mut h = BinaryHeap::new();
        // self和后续搜索的子树类型不同, 后续搜索的是&Box<Self>类型, 否则这部分垃圾就可以去掉了
        h.push(OrdT(pdist(x, self.pdiv, p), self.meta.clone()));
        if h.len() > k { h.pop(); }
        let dist = h.peek().unwrap().0;
        if self.bbox.cdist(x, p) <= dist {
            if let Some(ch) = &self.l {q.push_front(ch)};
            if let Some(ch) = &self.r {q.push_front(ch)};
        }
        while let Some(node) = q.pop_front() {
            h.push(OrdT(pdist(x, node.pdiv, p), node.meta.clone()));
            if h.len() > k { h.pop(); }
            let dist = h.peek().unwrap().0;
            if node.bbox.cdist(x, p) <= dist {
                if let Some(ch) = &node.l {q.push_front(ch)};
                if let Some(ch) = &node.r {q.push_front(ch)};
            }
        }
        return h;
    }
    /// 搜索的包装器, 输入k:近邻个数, x:输入点, p:p范数的p, 输出一个元组, 其中是k近邻中数据点的(距离,标签)元组
    pub fn search(&self, k: usize, x: [f32; K], p: f32) -> Vec<(f32, T)> {
        self
        .search_bfs_impl(k, x, p)
        .into_iter().map(|OrdT(n, x)| (n, x)).collect()
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


#[cfg(test)]
mod baseline;

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use super::*;
    use std::time::*;
    use super::baseline::*;

    const TEST_DIM: usize = 5;
    const TEST_NUM: usize = 1000;
    const TEST_SAM: usize = 1000000;

    fn eq_as_set<T: Ord>(a: Vec<T>, b: Vec<T>) -> bool {
        let set_a = BTreeSet::from_iter(a.into_iter());
        let set_b = BTreeSet::from_iter(b.into_iter());
        set_a == set_b
    }

    #[test]
    fn check_sanity() {
        let genf32 = || {
            let mut rng = thread_rng();
            let a = rng.gen::<f32>().abs();
            let b = rng.gen::<f32>().abs();
            a / (a + b) + 1f32
        };
        let data = Vec::from_iter((0..TEST_SAM)
            .map(|i| {
                let mut point = [0f32; TEST_DIM];
                let label = i % 5;
                for j in 0..TEST_DIM { point[j] = 100.0 * ((label + 1) as f32) * genf32(); }
                (point, label)
            })
        );
        let kdtree = KDTree::build(data.clone()).unwrap();
        let stupid = StupidKNN::build(data.clone());
        let test_points = Vec::from_iter((0..TEST_NUM)
            .map(|_| {
                let mut point = [0f32; TEST_DIM];
                for j in 0..TEST_DIM { point[j] = 100.0 * genf32(); }
                point
            })
        );
        let kdtree_results = kdtree.batch_search(15, test_points.clone(), 2.0);
        let stupid_results = stupid.batch_search(15, test_points, 2.0);
        for i in 0..TEST_NUM {
            assert!(eq_as_set(
                stupid_results[i].iter().map(|(x, y)| { OrdT(*x, y) }).collect(), 
                kdtree_results[i].iter().map(|(x, y)| { OrdT(*x, y) }).collect()
            ));
        }
    }

    #[test]
    fn bench_throughput() {
        let genf32 = || {
            let mut rng = thread_rng();
            let a = rng.gen::<f32>().abs();
            let b = rng.gen::<f32>().abs();
            a / (a + b) + 1f32
        };
        let data = Vec::from_iter((0..TEST_SAM)
            .map(|i| {
                let mut point = [0f32; TEST_DIM];
                let label = i % 5;
                for j in 0..TEST_DIM { point[j] = 100.0 * genf32(); }
                (point, label)
            })
        );
        let kdtree = KDTree::build(data.clone()).unwrap();
        let stupid = StupidKNN::build(data.clone());
        println!("tree construction finished");
        let test_points = Vec::from_iter((0..TEST_NUM)
            .map(|_| {
                let mut point = [0f32; TEST_DIM];
                for j in 0..TEST_DIM { point[j] = 100.0 * genf32(); }
                point
            })
        );
        println!("test case construction finished");
        let start_kdtree = Instant::now();
        let kdtree_results = kdtree.batch_search(20, test_points.clone(), 1.5);
        let end_kdtree = Instant::now();
        println!("kdtree implementation time: {:?}", end_kdtree - start_kdtree);
        let start_stupid = Instant::now();
        let stupid_results = stupid.batch_search(20, test_points.clone(), 1.5);
        let end_stupid = Instant::now();
        println!("brute force implementation time: {:?}", end_stupid - start_stupid);
        for i in 0..TEST_NUM {
            assert!(eq_as_set(
                stupid_results[i].iter().map(|(x, y)| { OrdT(*x, y) }).collect(), 
                kdtree_results[i].iter().map(|(x, y)| { OrdT(*x, y) }).collect()
            ));
        }
        println!("san check ok!");
    }
}