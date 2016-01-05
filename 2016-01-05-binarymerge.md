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

In that case, the seemingly trivial problem of merging two sorted lists turns into a problem that has *10 pages* of [TAOCP](https://en.wikipedia.org/wiki/The_Art_of_Computer_Programming) devoted to it. So I did what you usually do in this situation: [ask on stackexchange](http://programmers.stackexchange.com/questions/267406/algorithm-to-merge-two-sorted-arrays-with-minimum-number-of-comparisons). Given that this should be a pretty common problem, I was expecting an answer like "you obviously have to use the Foo-Bar algorithm described by 1969 by XYZ". But to my surprise, the algorithm that was posted as the answer, despite being called [A simple algorithm for merging two disjoint linearly-ordered sets](http://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.419.8292), is not very simple. It is asymptotically optimal, but too complex to degrade well in the case that the comparison is cheap.

So I tried to come up with a simpler solution.

# Cases

There are several cases that have to be considered when merging two sorted sequences. Coming up with a good solution for any of these cases is simple. The challenge is to come up with a solution that works well for all of the cases and gracefully degrades in the worst case.

1) Merging long list with single element list
```
a = [1,2,3,4,6,7,8,9,10]
b = [5]
```

The best solution in this case is to do a binary search for the position of the single element of *b* in *a*, then just copy the part of a that is below b(0), b(0), and the part of a that is above b(0). Obviously it would be possible to just special-case this solution. But that would be unelegant and in any case would not help in case 2

2) Merging a long list and a small list
```
a = [1,2,4,5,6,7,9,10]
b = [3,8]
```

In this case you might be tempted to just insert all elements of the smaller list into the larger list. But that would be less than optimal. From the insertion position of the first element, we know which elements are definitely smaller than the second element and thus do not have to be compared.

2) Merging two large lists which are disjoint
```
a = [1,2,3,4,5]
b = [6,7,8,9,10]
```

You could detect this case by comparing the first element of one list with the last element of the other list and vice versa. But the cost of that comparison will be overhead in other cases.

3) Merging two completely interleaved lists
```
a = [1,3,5,7,9]
b = [2,4,6,8,10]
```

This is the worst case, where it won't be possible to get better results than the linear merge. Any good algorithm should gracefully handle this case without doing much more than n+k-1 comparisons.
