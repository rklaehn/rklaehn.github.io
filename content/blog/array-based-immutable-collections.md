+++
title = "Array based immutable collections"
date = 2015-12-18
description = "Wrap arrays as fast, immutable collections in Scala."
[taxonomies]
tags = ["scala", "algorithms"]
+++

[abc](https://github.com/rklaehn/abc) - Wrap arrays as fast, immutable collections

-----

# Motivation

Primitive arrays are fast and compact in memory. But they are mutable, have a lot of quirks, and do not even have a working equals and hashcode. So they are rarely used directly as data structures in scala.

This library wraps flat, primitive arrays as **immutable** sequences, sets and maps. Now at first, the idea of having an immutable data structure backed by a single flat array without any sort of tree structure might seem **ridiculous**. Updating a single element is going to require copying the entire array and is thus going to be an O(n) operation.

But take a look at the typical usage patterns for immutable collections. Often, you transform the entire collection repeatedly with a sequence of map, flatmap and filter operations. So optimizing for single element updates at the expense of things like compact in-memory representation might not be worth it. So the approach in this library is not to make single element updates as fast as possible, *but to give you the tools to avoid having to do them in the first place*.

Now obviously *building* a collection by starting with an empty immutable collection and then adding all elements one-by-one would be an O(n<sup>2</sup>) operation and thus totally unacceptable. See benchmark results below. But *intentionally* doing this is also very rare in my experience. Usually you have some sort of sequence or iterable which you want to convert into a collection. That can always be done in at most O(n log n) using [the sonic reducer](@/blog/sonicreducer.md).

# Benchmarks

Benchmarks are done using [JMH](http://openjdk.java.net/projects/code-tools/jmh/) via the excellent [sbt-jmh](https://github.com/ktoso/sbt-jmh) plugin.

## ArraySet

The benchmarks compare creation, membership test, and bulk operations for `ArraySet[T]` with [`scala.collection.immutable.SortedSet[T]`](http://www.scala-lang.org/api/current/index.html#scala.collection.SortedSet) and [`scala.collection.immutable.HashSet[T]`](http://www.scala-lang.org/api/current/index.html#scala.collection.immutable.HashSet). An `ArraySet[A]` is just a wrapper around an ordered array. Lookup for contains etc. is done using a binary search and is therefore O(log n). Elements are sorted, so an `ArraySet[A]` is most closely comparable with a `SortedSet[A]`. But it will still perform better than a binary search tree, since the data is in a single continuous section of memory. And of course it will work up to very large collections where a binary tree will run out of memory because of its memory overhead.

### Building

The best approach to build ArraySets is to use the constructor that takes a sequence of elements. This is referred to as "create bulk" in the benchmarks. The naive approach of building an ArraySet is to add elements one by one. This is referred to as "create elements" in the benchmarks. In the scala collections, bulk creation is internally done by adding elements sequentially, so there is no difference between the two approaches.

![Building ArraySets](/images/setcreate.png)

As you can see from this benchmark, bulk creation of ArraySet[Int] is consistently *much* faster than building HashSet[Int] or SortedSet[Int]. The naive approach of adding elements sequentially unsurprisingly gets much slower for high n, since it is an O(n<sup>2</sup>) operation. *So don't do that.*

### Set/element operations

The essential set/element operation for a set is membership test. The two cases are for if the element is contained in the set, and if it is not contained in the set (outside).

![Set/Element operations](/images/setelement.png)

For a successful membership test, the performance is better than that of the SortedSet and sometimes even better than that of the HashSet. The performance difference between all three collections is not as high. The performance for a failed membership test is somewhere in the middle between SortedSet (worst) and HashSet (best). Note that both the x scale and the y scale are logarithmic, so ArraySet is doing **quite** a bit better than SortedSet due to the compact in-memory representation.

### Set/set operations

For all major set/set operations that are supported by scala collections, ArraySet is faster than SortedSet, often by **two orders of magnitude**. It is also significantly faster than HashSet for large n. Note the log scale on both the x- and the y-axis.

There are multiple lines because each benchmark is done multiple times for varying *overlaps*. See [the benchmark source](https://github.com/rklaehn/abc/blob/4eef7940c80da84b4c212b1e1dc2aff624c34930/jmhBenchmarks/src/main/scala/com/rklaehn/abc/SetSetBench.scala).

Set/set operations are implemented using my [Minimum-Comparison Merging algorithm](@/blog/binarymerge.md).

![Set/Set operations](/images/setset.png)

# Memory usage

Memory usage is measured using [jamm](https://github.com/jbellis/jamm). See [build.sbt](https://github.com/rklaehn/abc/blob/c9cb4f8ca8af6daa504869c5bfbe7d693032fa71/build.sbt#L127) for how it is used.

## ArraySeq

Memory usage of various Seq[Int] in bytes

|n|ArraySeq|Vector|List|
|--:|--:|--:|--:|
| 1| 48| 216| 56|
| 10| 80| 360| 416|
| 100| 440| 2376| 4016|
| 1000| 4040| 20808| 40016|
| 10000| 40040| 206712| 400016|
| 100000| 400040| 2064888| 4000016|

For List[Int], memory usage is almost exactly 10 times as large as for an Array[Int] or ArraySeq[Int]. For Vector[Int], memory usage is only about 5 times as high as Array[Int].

## ArraySet

Memory usage of various Set[Int] in bytes

|    n | ArraySet | HashSet | SortedSet |
|-----:|---------:|--------:|----------:|
|     1|        48|       40|        104|
|    10|        80|      480|        536|
|   100|       440|     5840|       4856|
|  1000|      4040|    57952|      48056|
| 10000|     40040|   543696|     480056|
|100000|    400040|  5862192|    4800056|

For Int sets, the memory usage of SortedSet is *about 12 times* more than that of ArraySet. The memory usage of HashSet is *about 14 times* higher than for ArraySet.

## ArrayMap

Memory usage of various Map[Int, Int] in bytes

|n|ArrayMap|HashMap|SortedMap|
|--:|--:|--:|--:|
| 1| 80| 64| 152|
| 10| 144| 768| 800|
| 100| 864| 8160| 7280|
| 1000| 8064| 95696| 86048|
| 10000| 80064| 943648| 878048|
| 100000| 800064| 9863680| 8798048|

For Int sets, the memory usage of SortedSet is *about 11 times* more than that of ArraySet. The memory usage of HashSet is *about 12 times* higher than for ArraySet.
