---
layout: post
title: [abc](https://github.com/rklaehn/abc) - Array based immutable collections
---

Wrap arrays as fast, immutable collections

-----

Primitive arrays are fast and compact in memory. But they are mutable, have a lot of quirks, and do not even have a working equals and hashCode. So they are rarely used as data structures in scala.

This library wraps primitive arrays as **immutable** sequences, sets and maps. Now at first, the idea of having an immutable data structure backed by a single flat array without any sort of tree structure might seem *ridiculous*. Updating a single element is going to require copying the entire array and is thus going to be an O(n) operation.

But take a look at the typical usage patterns for immutable collections. Often, you transform the entire collection repeatedly with a sequence of map, flatmap and filter operations. So optimizing for single element updates at the expense of things like compact in-memory representation might not be worth it.

# Benchmarks

Benchmarks are done using [JMH](http://openjdk.java.net/projects/code-tools/jmh/) via the excellent [sbt-jmh](https://github.com/ktoso/sbt-jmh) plugin.

# ArraySet

An ArraySet[A] is just a wrapper around an ordered array. Lookup for contains etc. is done using a binary search and is therefore O(log n). But it will still perform better than a binary search tree, since the data is in a single continuous section of memory. And of course it will work up to very large collections where a binary tree will run out of memory because of its memory overhead.

## Set/element operations

The essential set/element operation for a set is membership test. This benchmark compares an ArraySet[T] with a scala.collection.immutable.HashSet[T] and scala.collection.immutable.SortedSet[T]. The two cases are for if the element is contained in the set, and if it is not contained in the set (outside).

![Set/Element operations]({{ site.url }}/assets/setelement.png)

As you can see, the performance for a failed membership test is somewhere in the middle between SortedSet (worst) and HashSet (best). Note that both the x scale and the y scale are logarithmic, so ArraySet is doing **quite** a bit better than SortedSet due to the compact in-memory representation.

For a successful membership test, the performance is a bit better than that of the SortedSet, and the performance difference between all three collections is not as high.

## Set/set operations

This is where the array-based representation really shines. For all major set/set operations that are supported by scala collections, ArraySet is significantly faster than both HashSet and SortedSet, often by *two orders of magnitude*. Note the log scale on both the x- and the y-axis.

![Set/Set operations]({{ site.url }}/assets/setset.png)
