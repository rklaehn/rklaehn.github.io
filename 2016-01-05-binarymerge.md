---
layout: post
title: Efficient binary merge in spire
---

[sonicreducer](https://github.com/rklaehn/sonicreducer) - hierarchical reduction of sequences and iterables

-----

# Problem

At the end of 2014, I was thinking about what would be the most efficient way to merge two sorted sequences.

The answer is obviously trivial if you consider *copying* elements to be roughly as expensive as *comparing* elements. In that case, simply compare the *first remaining element* of each sequence and take the smaller one, until you run out of elements, then just copy the rest. Here is [the code in spire](https://github.com/rklaehn/spire/blob/eb70e8e89f669c1cdb731cacf5398c4f9e0dd3f7/core/shared/src/main/scala/spire/math/Merging.scala#L148).

But in many cases this the assumption that comparing is as expensive as copying is not true. Let's say you have a sequence of big integers, rational, long strings or complex tuples. In that case *comparing* two elements will be *several orders of magnitude* more expensive than *copying* an element.

So let's consider the case where only the number of comparisons matters, and the copying is considered to be essentially free  (Copying a pointer is just about the cheapest operation you can do. You can literally copy millions of pointers in less than a millisecond on a modern machine).

In that case, the seemingly trivial problem of merging two sorted lists turns into a problem that has *10 pages* of [TAOCP](https://en.wikipedia.org/wiki/The_Art_of_Computer_Programming) devoted to it. So I did what you usually do in this situation: [ask on stackexchange](http://programmers.stackexchange.com/questions/267406/algorithm-to-merge-two-sorted-arrays-with-minimum-number-of-comparisons). Given that this should be a pretty common problem, I was expecting an answer like "you have to use the foo-bar algorithm described by 1969 by xyz". 
