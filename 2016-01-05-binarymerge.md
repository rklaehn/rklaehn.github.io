---
layout: post
title: Efficient binary merge in spire
---

[sonicreducer](https://github.com/rklaehn/sonicreducer) - hierarchical reduction of sequences and iterables

-----

# Problem

At the end of 2014, I was thinking about what would be the most efficient way to merge two sorted sequences.

The answer is obviously trivial if you consider *copying* elements to be roughly as expensive as *comparing* elements. In that case, simply compare the *first remaining element* of each sequence and take the smaller one, until you run out of elements, then just copy the rest. Here is [the code in spire](https://github.com/rklaehn/spire/blob/eb70e8e89f669c1cdb731cacf5398c4f9e0dd3f7/core/shared/src/main/scala/spire/math/Merging.scala#L148).

But in many cases this the assumption that comparing is as expensive as copying is not true. Let's say you have a sequence of `BigInt`, `Rational`, very long `String` or complex tuples. In that case *comparing* two elements will be *several orders of magnitude* more expensive than *copying* an element.

So let's consider the case where only the number of comparisons matters, and the copying is considered to be essentially free  (Copying a pointer is just about the cheapest operation you can do. You can literally copy millions of pointers in less than a millisecond on a modern machine).

In that case, the seemingly trivial problem of merging two sorted lists turns into a problem that has *10 pages* of [TAOCP](https://en.wikipedia.org/wiki/The_Art_of_Computer_Programming) devoted to it. 

So I did what you usually do in this situation: [ask on stackexchange](http://programmers.stackexchange.com/questions/267406/algorithm-to-merge-two-sorted-arrays-with-minimum-number-of-comparisons). Given that this should be a pretty common problem, I was expecting an answer like "you obviously have to use the Foo-Bar algorithm described by 1969 by XYZ". But to my surprise, the algorithm that was posted as the answer, despite being called [A simple algorithm for merging two disjoint linearly-ordered sets](http://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.419.8292), is not very simple. It is asymptotically optimal, but too complex to degrade well in the case that the comparison is cheap.

So I tried to come up with a simpler solution.

# Cases

There are several cases that have to be considered when merging two sorted sequences. Coming up with a good solution for any of these cases is simple. The challenge is to come up with a solution that works well for **all** of the cases and that gracefully degrades in the worst case.

1) Merging long list with single element list
```
a = [1,2,3,4,6,7,8,9,10]
b = [5]
```

The best solution in this case is to do a binary search for the position of the single element of *b* in *a*, then just copy the part of a that is below b(0), b(0), and the part of a that is above b(0). Obviously it would be possible to just special-case this solution. But that would be unelegant and in any case would not help in case 2)

2) Merging a long list and a small list
```
a = [1,2,4,5,6,7,9,10]
b = [3,8]
```

In this case you might be tempted to just insert all elements of the smaller list into the larger list, doing binary searches for each insert. But that would be less than optimal. From the insertion position of the first element, we know which elements are definitely smaller than the second element and thus do not have to be compared, so we can restrict the range of the second binary search based on the result of the first.

3) Merging two large lists which are disjoint
```
a = [1,2,3,4,5]
b = [6,7,8,9,10]
```

You could detect this case by comparing the first element of one list with the last element of the other list and vice versa. But the cost of that comparison will be overhead in other cases.

4) Merging two completely interleaved lists
```
a = [1,3,5,7,9]
b = [2,4,6,8,10]
```

This is the worst case, where it won't be possible to get better results than the linear merge. Any good algorithm should gracefully handle this case without doing much more than m + n - 1 comparisons. Depending on what you expect as the average case, doing twice as many comparisons might still be OK. But e.g. *o(m log n)* comparisons, like you would get by inserting all *n* elements from the smaller list into the larger list with *m* elements, would *not* be ok.

# Coming up with a good algorithm

Being a functional programmer, I think that the most elegant algorithms are recursive. So let's think about how a recursion step would look like.

## Naming

Let's use `a0` and `a1` for the first (inclusive) and last (exclusive) index of `a` that we're currently interested in. Likewise, `b0` and `b1` for the first (inclusive) and last (exclusive) index of `b` that we're currently interested in.

## The base cases

Before we start thinking about complex things, let's consider the base case(s). Merging a section of a sequence with an *empty* section of another sequence means just copying over all elements of interest from that sequence to the target sequence.

## The first comparison

It is clear that we have to gain the maximum information from each comparison in order to limit the number of comparisons to the minimum. So it seems intuitively obvious that we have to compare the *middle* element of `a` with the *middle* element of `b`. No matter what the result of the comparison is, we have 50% of all elements in `a` that we never again have to compare with 50% of the elements in `b`. We have gained information for a quarter of all possible comparisons with just a single comparison. If you had a table of size m \* n with each cell being a possible comparison, executing the comparison at the *center* of the table allows you to eliminate an entire quadrant of the table.

`am = (a0 + a1) / 2`
`bm = (b0 + b1) / 2`
`a(am) < b(bm)`, so all elements `a(i), i from a0 to am` are smaller than all elements `b(j), j from bm until b1`.

## The recursion step

Now that know what we have to do for the first comparison, what do we do with it? What I came up with is the following: we look for the insertion position of the center element of `a` in `b`, using a binary search. The first comparison done by the binary search will be exactly as described above. Once we have the result `bm`, we can recurse.

We have to merge elements `a0 until am` from `a` with all elements `b0 until bm` from `b`. Then we have to copy the single element `a(am)` to the result, and finally merge elements `am + 1 until a1` from `a` with all elements `bm + 1 until b1` from `b`.

And that's it. Here is our code, for the case that there are `a` and `b` do not contain duplicate elements:

```scala
  def merge0(a0: Int, a1: Int, b0: Int, b1: Int): Unit = {
    if (a0 == a1) {
      // base case 1
      fromB(b0, b1) 
    } else if (b0 == b1) {
      // base case 2
      fromA(a0, a1)
    } else {
      // find position of center element of a in b
      val am = (a0 + a1) / 2
      // binary search for element a(am) in b, from b0 until b1
      val res = binarySearchB(am, b0, b1)
      // we know that res is negative, since a and b do not have common elements
      val bm = -res - 1
      // merge everything below a(am) with everything below the found insertion point into the result
      merge0(a0, am, b0, bm)
      // add a(am) to the result
      fromA(am, am + 1)
      // everything above a(am) with everything above the found insertion point into the result
      merge0(am + 1, a1, bm, b1)
    }
  }
```

