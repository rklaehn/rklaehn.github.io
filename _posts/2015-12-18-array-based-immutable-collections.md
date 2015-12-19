---
layout: post
title: Array based immutable collections
---

[abc](https://github.com/rklaehn/abc) - Wrap arrays as fast, immutable collections

-----

# Motivation

Primitive arrays are fast and compact in memory. But they are mutable, have a lot of quirks, and do not even have a working equals and hashCode. So they are rarely used as data structures in scala.

This library wraps primitive arrays as **immutable** sequences, sets and maps. Now at first, the idea of having an immutable data structure backed by a single flat array without any sort of tree structure might seem *ridiculous*. Updating a single element is going to require copying the entire array and is thus going to be an O(n) operation.

But take a look at the typical usage patterns for immutable collections. Often, you transform the entire collection repeatedly with a sequence of map, flatmap and filter operations. So optimizing for single element updates at the expense of things like compact in-memory representation might not be worth it.

Now obviously *building* a collection by starting with an empty immutable collection and then adding all elements one-by-one would be an O(n<sup>2</sup>) operation and thus totally unacceptable. But *intentionally* doing this is also very rare in my experience. Usually you have some sort of sequence or iterable which you want to convert into a collection. That can always be done in at most O(n log n) using [the sonic reducer](https://github.com/rklaehn/sonicreducer). But I might get to that later.

# Benchmarks

Benchmarks are done using [JMH](http://openjdk.java.net/projects/code-tools/jmh/) via the excellent [sbt-jmh](https://github.com/ktoso/sbt-jmh) plugin.

## ArraySet

An `ArraySet[A]` is just a wrapper around an ordered array. Lookup for contains etc. is done using a binary search and is therefore O(log n). Elements are sorted, so an `ArraySet[A]` is most closely comparable with a `SortedSet[A]` from the scala collections library. But it will still perform better than a binary search tree, since the data is in a single continuous section of memory. And of course it will work up to very large collections where a binary tree will run out of memory because of its memory overhead.

### Set/element operations

The essential set/element operation for a set is membership test. This benchmark compares an `ArraySet[T]` with a `scala.collection.immutable.HashSet[T]` and `scala.collection.immutable.SortedSet[T]`. The two cases are for if the element is contained in the set, and if it is not contained in the set (outside).

![Set/Element operations]({{ site.url }}/assets/setelement.png)

As you can see, the performance for a failed membership test is somewhere in the middle between SortedSet (worst) and HashSet (best). Note that both the x scale and the y scale are logarithmic, so ArraySet is doing **quite** a bit better than SortedSet due to the compact in-memory representation.

For a successful membership test, the performance is a bit better than that of the SortedSet, and the performance difference between all three collections is not as high.

### Set/set operations

This is where the array-based representation really shines. For all major set/set operations that are supported by scala collections, ArraySet is significantly faster than both HashSet and SortedSet, often by **two orders of magnitude**. Note the log scale on both the x- and the y-axis.

There are multiple lines because each benchmark is done multiple times for varying *overlaps*. See [the benchmark source](https://github.com/rklaehn/abc/blob/4eef7940c80da84b4c212b1e1dc2aff624c34930/jmhBenchmarks/src/main/scala/com/rklaehn/abc/SetSetBench.scala).

![Set/Set operations]({{ site.url }}/assets/setset.png)


# Memory usage

Memory usage is measured using [jamm](https://github.com/jbellis/jamm). See [build.sbt](https://github.com/rklaehn/abc/blob/c9cb4f8ca8af6daa504869c5bfbe7d693032fa71/build.sbt#L127) for how it is used.

## ArraySet

For Int sets, the memory usage of HashSet or SortedSet is *more than ten times* more than an Array[Int] or an ArraySet[Int].

|    n | ArraySet | HashSet | SortedSet |
|-----:|---------:|--------:|----------:|
|     1|        48|       40|        104|
|    10|        80|      480|        536|
|   100|       440|     5840|       4856|
|  1000|      4040|    57952|      48056|
| 10000|     40040|   543696|     480056|
|100000|    400040|  5862192|    4800056|

## ArrayMap

For Int to Int maps, the memory usage of HashMap or SortedMap is *more than ten times* more than an Array[Int] or an ArraySet[Int].

|n|ArrayMap|HashMap|SortedMap|
|--:|--:|--:|--:|
| 1| 80| 64| 152|
| 10| 144| 768| 800|
| 100| 864| 8160| 7280|
| 1000| 8064| 95696| 86048|
| 10000| 80064| 943648| 878048|
| 100000| 800064| 9863680| 8798048|
