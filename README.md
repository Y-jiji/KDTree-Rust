# KDTree-Rust

> This project provides a tutorial for data scientists who wants to work in Rust. 

In this tutorial, we will implement a simple K-nearest neighbour (KNN) algorithm in Rust. 

Finally this implmentation will achieve a performance comparable to [scikit-learn](https://scikit-learn.org/stable/modules/neighbors.html) . 

The tutorial divides into three phases. Each phase presents some basics of Rust language, and makes the KNN algorithm faster. 

***
***

# phase-1: Naive KNN

In phase-1, we will implement a naive k-nearest-neighbour algorithm. 

## step-1

To achieve this goal, we will first learn how to start a project with Rust. 

First, we install rust toolchain following the guide on rust [official website](https://www.rust-lang.org/tools/install). 

After installation, run the following instruction to make sure it works normally. 

```bash
cargo --version
```

The output should look like `cargo x.x.x ({hash} {YYYY}-{MM}-{DD})`, e.g. `cargo 1.62.1 (a748cf5a3 2022-06-08)` on my laptop. 

Change the working directory to phase-1, where we will start our work. However, before we start coding, we should first change this directory into a cargo project. To accomplish this, we type the command: 

```bash
cargo init --lib
```

(TODO@Y-jiji)

## step-2

(TODO@Y-jiji)

***
***

# phase-2: KD-Tree

## step-1: Basic Ideas of KD-Tree

In phase-2, we will implement a KD-Tree with two search strategies ([bfs](https://en.wikipedia.org/wiki/Breadth-first_search) and [dfs](https://en.wikipedia.org/wiki/Depth-first_search)). 

However, before coding, we should bear in mind how kd-tree works. 

We have a consensus that scaning all points for the nearest neighbour in our dataset is too expensive -- for practical purpose, a dataset may consist of billions of points. 

In fact, there is a simple heuristic to filter out a bunch of impossible points from candidates at O(1) time expense. 

Lets pick an simple example to see how it works. 

Notation: $d(x, y)$ means distance between $x$ and $y$

(TODO@Y-jiji: Add an image here)

We observe that for all point $a$ in the box, and the center of the box $c$, $d(c, p) < d(a, p) + d(c, a)$ . 

So, if there is a point $b$ with $\forall a: d(b, p) < d(c, p) - d(c, a) < d(a, p)$, the nearest neighbour will never fall into the box. With some logically equivalent transformation, we get: 

$$
d(b, p) < \inf_{a\in Box} d(c, p) - d(c, a) < d(c, p) - \sup_{a \in Box} d(c, a)
$$

We can easily compute $\sup_{a\in Box} d(c, a)$, which is just the distance between the center point and an arbitary corner of the box. To our delight, for a fixed box, this value is a constant. In summary, if there is a point $b$ with $d(b, p) < d(c, p) - (distance\;between\;c\;and\;a\;corner)$, we can truncate the whole box from our search space. 

In order to put this idea into practice, the KD-Tree first divides data into boxes (or super-rectangles in fancy terms). 

At each level of KD-Tree, it sorts the dataset ${\cal D} = \{x_1,x_2,\cdots, x_n\}$ by a randomly picked dimension $d$, and divides the dataset by lower median $x_i$ under this ordering. The dataset falls into three parts $A = \{x_j: x_j^d < x_i^d\}$, $B = \{x_i\}$ and $C = \{x_j: x_j^d > x_i^d\}$ . Keep the information of $x_i$ on the current level, and build the left subtree with dataset $A$ and the right subtree with dataset $B$ recursively. This recursion ends when the input dataset is empty. For each subtree, we record the smallest possible box that contains all points in the data set. 

For searching, a subtree is truncated if the correspondent box is truncated by the heuristic described before, any top-down tree traversing algorithm can be combined with this method. 

## step-2: Tree is a Recursive Type

A tree have other trees as substructures. For such types that contain substructures of the same type, we call them [recursive types](https://en.wikipedia.org/wiki/Recursive_data_type). 

This is OK in most [imperative programming languages](https://en.wikipedia.org/wiki/Imperative_programming), but it makes the situation tricky in Rust, which is partially a [functional programming language](https://en.wikipedia.org/wiki/Functional_programming) and takes intensive care of memory layout of structures. 

Things like the following makes the compiler unhappy: 

```rust
struct A {
    a: A,
    b: A,
    c: i32,
}
```

However, the compiler is happy with: 

```rust
struct A {
    a: Box<A>,
    b: Box<A>,
    c: i32
}
```

## step-3: KD-Tree Builder Trait

(TODO@Y-jiji)

## step-4: KD-Tree Searching

(TODO@Y-jiji)

***
***

# phase-3: Fearless Cocurrency with Rust

In phase-3, we will implement a multi-threaded version of our KD-Tree searching. 

The rust standard library enables expressing multi-threading in an elegant way. 

## step-1

(TODO@Y-jiji)

