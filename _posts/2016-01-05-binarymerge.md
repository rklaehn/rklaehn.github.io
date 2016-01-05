---
layout: post
title: Minimum-Comparison Merging in spire
---

[BinaryMerge](https://github.com/rklaehn/spire/blob/eb70e8e89f669c1cdb731cacf5398c4f9e0dd3f7/core/shared/src/main/scala/spire/math/Merging.scala#L17) - Efficient binary merge in spire

-----

# Problem

At the end of 2014, I was thinking about what would be the most efficient way to merge two sorted sequences.

The answer is obviously trivial if you consider *copying* elements to be roughly as expensive as *comparing* elements. In that case, simply compare the *first remaining element* of each sequence and take the smaller one, until you run out of elements, then just copy the rest. Here is [the code in spire](https://github.com/rklaehn/spire/blob/eb70e8e89f669c1cdb731cacf5398c4f9e0dd3f7/core/shared/src/main/scala/spire/math/Merging.scala#L148).

But in many cases this the assumption that comparing is as expensive as copying is not true. Let's say you have a sequence of `BigInt`, `Rational`, very long `String` or complex tuples. In that case *comparing* two elements will be *several orders of magnitude* more expensive than *copying* an element.

So let's consider the case where only the number of comparisons matters, and the copying is considered to be essentially free  (Copying a pointer is just about the cheapest operation you can do. You can literally copy millions of pointers in less than a millisecond on a modern machine).

In that case, the seemingly trivial problem of merging two sorted lists turns into a problem that has *10 pages of [TAOCP](https://en.wikipedia.org/wiki/The_Art_of_Computer_Programming)* devoted to it (Volume 3, Pages 197-207, **Minimum-Comparison Merging**)

So I did what you usually do in this situation: [ask on stackexchange](http://programmers.stackexchange.com/questions/267406/algorithm-to-merge-two-sorted-arrays-with-minimum-number-of-comparisons). Given that this should be a pretty common problem, I was expecting an answer like "you obviously have to use the Foo-Bar algorithm described by 1969 by XYZ". But to my surprise, the algorithm that was posted as the answer, despite being called [A simple algorithm for merging two disjoint linearly-ordered sets (F. K. Hwang , S. Lin)](http://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.419.8292), is not very simple. It is asymptotically optimal, but too way too complex to degrade well in the case that the comparison is relatively cheap. 

Also, it is pretty complex to implement. For example, it is using ***floating point operations*** to calculate indices.

So I tried to come up with a simpler solution.

# Cases

There are several cases that have to be considered when merging two sorted sequences. Coming up with a good solution for any of these cases is simple. The challenge is to come up with a solution that works well for **all** of the cases and that gracefully degrades in the worst case.

a) Merging long sequence with single element sequence

```
a = [1,2,3,4,6,7,8,9,10]
b = [5]
```

The best solution in this case is to do a binary search for the insertion point of the single element of `b` in `a`, then just copy the part of a that is below `b(0)`, `b(0)`, and the part of a that is above `b(0)`. Obviously it would be possible to just special-case this solution. But that would be unelegant and in any case would not help in case b)

b) Merging a long sequence and a short sequence

```
a = [1,2,4,5,6,7,9,10]
b = [3,8]
```

In this case you might be tempted to just insert all elements of the smaller list into the larger list, doing binary searches for each insert. But that would be less than optimal. From the insertion position of the first element, we know which elements are definitely smaller than the second element and thus do not have to be compared, so we can restrict the range of the second binary search based on the result of the first.

c) Merging two large sequences which are non-overlapping

```
a = [1,2,3,4,5]
b = [6,7,8,9,10]
```

This is a case where you can expect huge performance gains, because you just have to copy one list after the other. You could detect this case by comparing the first element of one sequence with the last element of the other sequence and vice versa. But the cost of that comparison will be overhead in other cases.

d) Merging two completely interleaved sequences

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

Before we start thinking about complex things, let's consider the base case(s). Merging a section of a sequence with an *empty* section of another sequence means just copying over all elements of interest from that sequence to the target sequence. So if `a0` is `a1`, just copy everything from `b0` to `b1` to the result, and vice versa.

## The first comparison

It is clear that we have to gain the maximum information from each comparison in order to limit the number of comparisons to the minimum. So it seems intuitively obvious that we have to compare the *middle* element of `a` with the *middle* element of `b`. No matter what the result of the comparison is, we have 50% of all elements in `a` that we never again have to compare with 50% of the elements in `b`. We have gained information for a quarter of all possible comparisons with just a single comparison. If you had a table of size m \* n with each cell being a possible comparison, executing the comparison at the *center* of the table allows you to eliminate an entire quadrant of the table.

   | 5 | 6 | 7 | 8 | 9 |
---|---|---|---|---|---|
 1 |   |   | > | > | > |
 3 |   |   | > | > | > |
 5 |   |   | > | > | > |
 7 |   |   |   |   |   |
 9 |   |   |   |   |   |

```
am = (a0 + a1) / 2
bm = (b0 + b1) / 2
````

`a(am) < b(bm)`, so *all* elements `a(i), a0 ≤ i ≤ am` are smaller than *all* elements `b(j), bm ≤ j < b1`.

## The recursion step

Now that know what we have to do for the first comparison, what do we do with it? What I came up with is the following: we look for the *insertion index* of the center element of `a` in `b`, using a binary search. The first comparison done by the binary search will be exactly as described above. Once we have the result, which we shall call `bm`, we can recurse.

We have to merge elements `a0 until am` from `a` with all elements `b0 until bm` from `b`. Then we have to copy the single element `a(am)` to the result, and finally merge elements `am + 1 until a1` from `a` with all elements `bm + 1 until b1` from `b`.

And that's it. Here is our code, for the case that `a` and `b` are disjoint ordered sets.

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

Note that while this method is using recursion, it is not referentially transparent. The result sequence is built in the methods fromA and fromB using a mutable builder for efficiency. Of course, you will typically wrap this algorithm in a referentially transparent way.

Also note that the [version in spire](https://github.com/rklaehn/spire/blob/eb70e8e89f669c1cdb731cacf5398c4f9e0dd3f7/core/shared/src/main/scala/spire/math/Merging.scala#L61) is slightly more complex, because it also works for the case where there are common elements in `a` and `b`, and because it is sometimes an advantage to have the insertion point.

# Behavior for the cases described above

a) Merging long list with single element list

It might seem that the algorithm is not symmetric. But at least for the case of merging a large list with a single element list, the algorithm boils down to a binary search in both cases.

b) Merging a long list and a small list

The algorithm will use the information from the comparison of both middle elements to avoid unnecessary comparisons

c) Merging two long non-overlapping lists

The algorithm will figure out in O(log n) in the first recursion step that the lists are disjoint, and then just copy them

d) Merging interleaved lists

This is tricky, but tests with counting comparisons have indicated that the maximum number of comparisons is never much more than `m + n - 1`.

# Benchmarks

## Case c: Merging two long non-overlapping lists

A simple benchmark was done to compare the linear merge with the binary merge in the case where two long, non-overlapping sequences are compared. This was a case that was very important for my original use case. The benchmark was done using both small rational numbers (as an example with a reasonably expensive comparison) and integers (as an example with a very cheap comparison).

```scala
val ar = (0 until 1000).map(Rational.apply).toArray
val br = (1000 until 1001).map(Rational.apply).toArray
th.pbenchOffWarm("binary merge vs. linear merge (Rational)")(th.Warm(LinearMerge.merge(ar,br)))(th.Warm(BinaryMerge.merge(ar,br)))

val ai = (0 until 1000).toArray
val bi = (1000 until 1001).toArray
th.pbenchOffWarm("binary merge vs. linear merge (Int)")(th.Warm(LinearMerge.merge(ai,bi)))(th.Warm(BinaryMerge.merge(ai,bi)))
```

Here are the results.

```
[info] Significantly different (p ~= 0)
[info]   Time ratio:    0.03200   95% CI 0.03140 - 0.03259   (n=20)
[info]     First     25.76 us   95% CI 25.32 us - 26.19 us
[info]     Second    824.1 ns   95% CI 817.4 ns - 830.8 ns
[info] Benchmark comparison (in 4.780 s): binary merge vs. linear merge (Int)
[info] Significantly different (p ~= 0)
[info]   Time ratio:    0.17180   95% CI 0.17020 - 0.17341   (n=20)
[info]     First     4.070 us   95% CI 4.039 us - 4.100 us
[info]     Second    699.2 ns   95% CI 695.3 ns - 703.0 ns
```

As expected, the performance difference for the rational case is pretty large, despite this being *small* rational numbers. For rational numbers with complex fractions, the difference would be even larger. 

But even for the integer case, there is a performance difference of a factor of 5. The reason is that copying a large number of elements using a few calls to System.arraycopy is *extremely* fast compared to copying them one by one, and of course that even integer comparisons are expensive compared to a simple copy.

## Case d: Merging interleaved lists

Now let's look at the results for the absolute worst case, where the two sequences are completely overlapping and exactly interleaved.

```
val ar = (0 until 2000 by 2).map(Rational.apply).toArray
val br = (1 until 2001 by 2).map(Rational.apply).toArray
th.pbenchOffWarm("binary merge vs. linear merge (Rational)")(th.Warm(LinearMerge.merge(ar,br)))(th.Warm(BinaryMerge.merge(ar,br)))

val ai = (0 until 2000 by 2).toArray
val bi = (1 until 2001 by 2).toArray
th.pbenchOffWarm("binary merge vs. linear merge (Int)")(th.Warm(LinearMerge.merge(ai,bi)))(th.Warm(BinaryMerge.merge(ai,bi)))
```

Here are the results.

```
[info] Significantly different (p ~= 0)
[info]   Time ratio:    1.52149   95% CI 1.49551 - 1.54748   (n=20)
[info]     First     52.99 us   95% CI 52.23 us - 53.74 us
[info]     Second    80.62 us   95% CI 79.86 us - 81.38 us
[info] Benchmark comparison (in 3.930 s): binary merge vs. linear merge (Int)
[info] Significantly different (p ~= 0)
[info]   Time ratio:    5.42174   95% CI 5.34107 - 5.50242   (n=20)
[info]     First     9.458 us   95% CI 9.379 us - 9.537 us
[info]     Second    51.28 us   95% CI 50.65 us - 51.91 us
```

In the case of rational numbers, the binary merge is a bit slower than the linear merge. But not by a significant factor. In the case of integers, where the cost of a comparison is almost too cheap to measure, the linear merge is superior by a factor of 5 or so. But this is to be expected, simply due to the fact that calls to System.arraycopy to copy a single element will be more expensive than just copying an integer. And again, this represents the exact opposite of what the binary merge is made for.

# Questions

- Does anybody know the name of this algorithm? I think that it is a big improvement over the Hwang-Lin algorithm. So if you know the name, I *really* would like to know it. If not, I call dibs. I find this tremendously useful and use it in several of my open source libraries such as [abc](https://github.com/rklaehn/abc) and [intervalset](https://github.com/rklaehn/intervalset), as well as in proprietary libraries for my current employer.

- I am pretty convinced that this algorithm is asymptotically optimal, just like Hwang-Lin. But I don't really know how to prove this. Anybody have any hints how to approach something like this?
