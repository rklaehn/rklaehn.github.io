---
layout: post
title: The sonic reducer
---

[sonicreducer](https://github.com/rklaehn/sonicreducer) - hierarchical reduction of sequences and iterables

-----

# Problem

This is a tiny library for reducing arrays or traversables in an hierarchical way. This can turn O(n^2) operations into O(n log(n)) operations in some cases.

Imagine you had to concatenate a large number of `String`s without using a `StringBuilder`. The most straightforward way to do this would be `strings.reduce(_ + _)`. But that is O(n<sup>2</sup>). Using hierarchical reduction, this can be done in O(n log(n)) without too much effort `Reducer.reduce(strings)(_ + _).get`.

*Of course this assumes that the operation used for reduction is associative*, or in other words the type and the binary operation should form a [semigroup](https://en.wikipedia.org/wiki/Semigroup)

When reducing traversables of indeterminate size, the traversable is converted to a binary tree structure and then immediately reduced without retaining the intermediate tree structure. This is done using a [buffer of 32 elements](https://github.com/rklaehn/sonicreducer/blob/e98fa6facfa55c8d323caa7e448dc651422124cc/src/main/scala/com/rklaehn/sonicreducer/Reducer.scala#L75), which is enough for collections of up to 2<sup>32</sup> elements.

```
(1 to 8).reduceLeft(_ + _)
(((((((1+2)+3)+4)+5)+6)+7)+8)

(1 to 8).reduceRight(_ + _)
(1+(2+(3+(4+(5+(6+(7+8)))))))

Reducer.reduce(1 to 8)(_ + _)
((1+2)+(3+4))+((5+6)+(7+8))
```

ASCII art (note that the tree is never actually fully constructed):
```
              36
             / \
            /   \
           /     \
          /       \
         /         \
        /           \
       /             \
      10              26
     / \             / \
    /   \           /   \
   /     \         /     \
  3       7       11      15
 / \     / \     / \     / \
1   2   3   4   5   6   7   8 
```

Now obviously, in the case of concatenating strings you would just use a `StringBuilder`. But there are many situations where it is easy to define a *combine* operation that is efficient when both sides have roughly the same *weight*.

I use this extensively in my [immutable collections library](http://rklaehn.github.io/2015/12/18/array-based-immutable-collections/), where I construct sets by having an union operation that is efficient for sets of roughly the same size.

# Examples

An example would be summing up the rational numbers 1/1 + 1/2 + 1/3 + 1/4 + ... + 1/n. The intermediate results will have very large numerators and denominators, so summing up using reduceLeft will be inefficient. Hierarchical reduction will only produce large numerators and denominators at the top of the tree.

Another example would be summming a number of doubles. Floating point addition is of course not associative, but people like to pretend that it is. Summing floating point numbers sequentially from left to right will let the floating point error accumulate more than hierarchical summing.

# Benchmarks

See the [Benchmarks](src/test/com/rklaehn/sonicreducer/SonicReducerBench.scala) for examples on how to use the Reducer. To run the benchmarks, use `sbt sonicReducerJVM/test:run`.

Here is an example for summing the rational numbers 1/1 + 1/2 + 1/3 + 1/4 + ...

```
val th = Thyme.warmed(warmth = Thyme.HowWarm.BenchOff, verbose = println)
val rationals = (1 to 1000).map(i ⇒ Rational(1, i))
th.pbenchOffWarm("sum 1000 Rationals 1/x")(th.Warm(rationals.reduce(_ + _)))(th.Warm(Reducer.reduce(rationals)(_ + _).get))
```

As you can see from the results, there is a significant performance benefit in hierarchical reduction. Note that this would also work for a very large stream of rationals.

Results:
```
Benchmark comparison (in 5.757 s): sum 1000 Rationals 1/x
Significantly different (p ~= 0)
  Time ratio:    0.04779   95% CI 0.04699 - 0.04858   (n=20)
    First     34.63 ms   95% CI 34.43 ms - 34.84 ms
    Second    1.655 ms   95% CI 1.629 ms - 1.681 ms
```

# Conclusion

I think even though this is just a few lines of code, it is useful enough to warrant making a library out of it.
