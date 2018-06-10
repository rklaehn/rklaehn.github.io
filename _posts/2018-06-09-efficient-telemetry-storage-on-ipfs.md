---
layout: post
title: Safely running a public IPFS gateway using nginx
---

-----
# TL;DR;

You have large quantities of telemetry or other simply structured JSON data. You want to store it on IPFS in a very efficient way. Then this is for you.

# History

## Telemetry at GSOC

I have been working with large quantities of telemetry at my previous job at [Heavens-Above GmbH](https://www.heavens-above.com/main.aspx), working for the [German Space Operations Center](https://www.dlr.de/dlr/en/desktopdefault.aspx/tabid-10368/562_read-479/). I was writing a system to store and make accessible telemetry for several satellite missions.

The most demanding mission in terms of data rate was the columbus module of the international space station.

<a title="By NASA (https://archive.org/details/s127e009781) [Public domain], via Wikimedia Commons" href="https://commons.wikimedia.org/wiki/File:Columbus_module_-_cropped.jpg"><img width="1024" alt="Columbus module - cropped" src="https://upload.wikimedia.org/wikipedia/commons/thumb/e/ec/Columbus_module_-_cropped.jpg/1024px-Columbus_module_-_cropped.jpg"></a>

A big advantage of the solution we came up with is that it is able to store vast quantities of telemetry data on relatively modest hardware. For the columbus module, we stored on the order of 4 * 10<sup>12</sup> telemetry samples on a single decent machine with a RAID-5 system.

This system is not a relational database that allows generic queries, but it does allow *very* efficient retrieval by telemetry parameter name and by time period.

We achieved this level of efficiency by storing the telemetry in an efficient binary format based on [google protocol buffers](https://developers.google.com/protocol-buffers/), and then using generic [deflate](https://www.ietf.org/rfc/rfc1951.txt) compression in a way that still allows somewhat random access.

A limitation of the solution developed for GSOC was that it was schema-based, so there was a tight coupling between the storage engine and the application level data formats. But since telemetry schemata for systems like [SCOS](https://en.wikipedia.org/wiki/SCOS_2000) don't change frequently, this is not a big limitation. 

## Telemetry at Actyx

At my current employer [Actyx AG](actyx.io), we are also working with vast quantities of machine data, but now we are dealing with telemetry for manufacturing processes. 

We are using IPFS as the core infrastructure for all our solutions.

So we need a solution to store machine telemetry in an efficient way on IPFS. But there is an incredible variety of manufacturing machines and processes with vastly different telemetry formats, so a tight coupling between storage engine and application data formats won't do in this case.

As an additional complication, we want to be able to write telemetry not just on beefy servers, but on relatively underpowered edge devices.

As strange as it sounds, the hardware I have to design for at Actyx in 2018 (ARM based edge devices) is actually *slower* than the hardware I had at GSOC in 2010 (Multi core XEON blade servers).

### 2010

<a title="By Dmitry Nosachev [CC BY-SA 4.0 (https://creativecommons.org/licenses/by-sa/4.0)], from Wikimedia Commons" href="https://commons.wikimedia.org/wiki/File:Supermicro_SBI-7228R-T2X_blade_server.jpg"><img width="1024" alt="Supermicro SBI-7228R-T2X blade server" src="https://upload.wikimedia.org/wikipedia/commons/thumb/d/d0/Supermicro_SBI-7228R-T2X_blade_server.jpg/1024px-Supermicro_SBI-7228R-T2X_blade_server.jpg"></a>

### 2018

<a title="By SmartAsianRobot [CC BY-SA 4.0 (https://creativecommons.org/licenses/by-sa/4.0)], from Wikimedia Commons" href="https://commons.wikimedia.org/wiki/File:Pi3B%2B.jpg"><img width="1024" alt="Pi3B+" src="https://upload.wikimedia.org/wikipedia/commons/thumb/7/76/Pi3B%2B.jpg/1024px-Pi3B%2B.jpg"></a>

## What is telemetry data

First of all, let's take a look at how telemetry data is typically structured.

In the simplest case, a telemetry sample consists of a time and a value. In more complex cases, you have several different values and/or some status fields that indicate the status of the sample and individual subvalues. Some of these additional fields might be optional.

But in all cases the structure of a *single* telemetry measurement is relatively simple.

What makes handling telemetry demanding is the vast quantities of data you have to store and process. A single sensor measuring at a relatively modest rate of one per second will produce 86&nbsp;400 samples per day, 31&nbsp;536&nbsp;000 samples per year. And on a spacecraft you might have 10000 or more different sensors, some of which might be sampled at much higher rates than 1 Hz.

You also can't throw away or summarize any of this data. This is not a [click stream](https://en.wikipedia.org/wiki/Clickstream) where losing a few samples is not a big deal. In case of an anomaly, you want to be able to investigate every single sample at the time at and before the anomaly.

Sometimes even the *milliseconds* of the *timestamps* of samples at the time before an anomaly can tell you something interesting.

If you would have the ability to process the data in the cloud, an obvious solution would be to throw hardware at the problem. But if you want to store the data on edge devices where it is gathered, this is not an option.

## Compression is understanding

Another reason why I dislike just throwing hardware at the problem is that I think compression is understanding. You can only compress data well if you understand it.

Having the data available in a highly compressed way makes the job easier for every downstream system (analysis, backup, transmission, ...).

If your data is highly compressed, it can effortlessly be copied to where you perform computations on it, even if your edge device is connected via an intermittent GSM connection. And of course you can replicate it in many different places, making the chances of losing data astronomically small.

## What is good compression

There is a theoretical limit on how well you can compress sensor data. Sensor data typically contains some sensor noise for both the value and the timestamp. You obviously can not compress better than that theoretical limit, but if you get within a factor of 2 of it you are probably doing pretty well.

# The encoding approach

## Columnar storage

A first insight when working with large quantities of telemetry is that there is significant redundancy between subsequent values. E.g. a temperature or pressure reading of a sensor typically does not change very much from one second to the next, except if you have a [really bad day](https://www.youtube.com/watch?v=Zl12dXYcUTo).

This insight is not revolutionary. In fact, there are several systems that exploit this to offer highly compressed data storage for analysis workloads:

- [CitusData](https://www.citusdata.com/)

  Vanilla postgres with columnar storage engine, open source, hosted service

- [AWS redshift](https://docs.aws.amazon.com/redshift/latest/mgmt/welcome.html)

  Customized postgres with columnar storage, only hosted

- [SQL Server](https://blogs.msdn.microsoft.com/sqlserverstorageengine/2017/02/09/json-data-in-clustered-column-store-indexes/)

  "Clustered ColumnStore Indices"

But none of them works on a raspberry pi, and none of them stores in a location-transparent way on IPFS.

## Simple transpose

So assuming you have a large number of telemetry samples in JSON format. The simplest thing you can do to improve the serialized representation is to transpose an array of records to a record of arrays:

Array of records (uniform type)

```javascript
[
  { latitude: 1, longitude: 2 },
  { latitude: 0, longitude: 3 },
  { latitude: 1, longitude: 2 },
  { latitude: 0, longitude: 3 },
  { latitude: 1, longitude: 5 },
  { latitude: 2, longitude: 2 }
]
```

Record of arrays

```javascript
{
  latitude: [1, 0, 1, 0, 1, 2],
  longitude: [2, 3, 2, 3, 5, 2]
}
```

There are several projects that do this. See for example [JSON compression - transpose and binary](http://mainroach.blogspot.com/2013/08/json-compression-transpose-binary.html).

But this most basic approach only works if we have exactly the same values in each sample. As we have seen this is not the case for some satellite telemetry samples such as [SCOS](https://en.wikipedia.org/wiki/SCOS_2000) samples.

Ideally we should be able to deal with arbitrary json samples.

## Heterogenous types

So how do we deal with optional fields or, more generically, heterogenous records, like you might see in an [event sourcing](https://martinfowler.com/eaaDev/EventSourcing.html) application?

Here we have an array of records of different types. The `type` field is always present, but `w` and `h` are only present for `rectangle` records whereas `r` is only present for `circle` records.

Array of heterogenous records
```javascript
[
  { type: 'circle', r: 10 }
  { type: 'rectangle', w: 3, h: 4 },
  { type: 'rectangle', w: 10, h: 10 }
]
```

One way to deal with this would be to have some sort of guard value for when a value is not present. But that would be very inefficient if a value is absent very often (essentially a [sparse matrix](https://en.wikipedia.org/wiki/Sparse_matrix))

Another way is to store two arrays per field, one that has the *value* of the field, and another that stores the *index* at which each field is present.

```javascript
{
  type: [[0,1,2], ['circle','rectangle','rectangle']],
  r: [[0],[10]],
  w: [[1,2],[3,10]],
  h: [[1,2],[4,10]]
}
```

In the very common case of uniform records, this looks like it is making things *much* worse. We now store two arrays instead of one. So we will need to have a way to get rid of this duplication later.

## Nested records

What we have now is the ability to transform an array of flat records with arbitrary fields. But telemetry samples frequently contain nested records. For example, a SCOS sample contains different subvalues, each of which has a value and a status.

And in any case, our goal is to be able to handle *arbitrary* json data. If a nested record is the *natural* way to represent a telemetry sample, we should not have to flatten it to store it.

So here is a tiny example of a number of nested records.

```javascript
[
  { type: 'start', pos: { lat: 10, lon: 10 }},
  { type: 'pause', pos: 'na'},
  { type: 'stop', pos: { lat: 20, lon: 0 }}
]
```

To store this, at every level of our index structure we can either have values (index array and value array, as described above), or named children to be able to store nested records.

```javascript
{
  children: {
    type: {
      values: [[0,1,2], ['start', 'pause', 'stop']]
    },
    pos: {
      values: [[1], ['na']],
      children: {
        lat: {
          values: [[0, 2], [10,20]]
        },
        lon: {
          values: [[0, 2], [10,0]]
        }
      }
    }
  }
}
```

So now we have managed to blow up our tiny sample of nested records to a much larger and more complex structure. Wasn't the idea to *improve* compression?

## Binary encoding with CBOR

One thing that is immediately obvious is that JSON is not a very efficient storage format for many kinds of data, especially numeric data. E.g. the number 100 can be encoded as a single byte, which is a factor of three improvement. For strings the improvement is not as drastic, but still significant.

An encoding format that is binary and schemaless is [CBOR](https://tools.ietf.org/html/rfc7049). It stands for *Concise Binary Object Representation*. The fact that the author of the format is called *Carsten Bormann* is a mere coincidence.

IPFS is [using CBOR](https://github.com/ipld/specs/blob/master/IPLD.md#serialized-data-formats) as serialized format, so whenever you add JSON to ipfs using the ipfs dag API, it is actually stored as more efficient CBOR.

So CBOR seems like a good choice for binary encoding.

As described above, the index and value arrays for an individual field can be expected to be of the same type most of the time, and also to have significant redundancy. So we don't encode the entire structure, but the individual index and value arrays.

CBOR encoders and decoders exist for many languages, so doing this is just a matter of picking and using an existing library.

## Compression

Now that we have the individual index and value arrays in a nice and compact CBOR binary representation, we can compress them with a generic compression algorithm such as [deflate](https://www.ietf.org/rfc/rfc1951.txt).

Our index arrays always contain just typically small integers, so both CBOR encoding and the subsequent compression can be expected to work well. The same is true for the value arrays.

### Enum compression

For the type field for a discriminated union, we can expect a very high level of compression. An entire, possibly long, string is used, but what is actually needed is just a small integer indicating which of the small number of possible string values to use.

#### Example data

Example event type field data (1000 random elements)

```javascript
['start', 'resume', 'pause', ..., 'stop']
```

#### Results

| Compression | bytes | bytes/sample |
| -------- | -------- | -------- |
| JSON     | 8502     | 8.502    |
| CBOR     | 6507     | 6.507    |
| Deflate  | 323      | 0.323    |

From the results we can see that CBOR helps a bit, since it stores a single byte length prefix instead of the `=".."` for JSON, so we gain two bytes for each string. But the real gain comes from deflate compression, which reduces the storage requirements for each value to less than 4 bits.

### Sensor data compression

Numerical sensor data is another case where we can expect large gains due to binary encoding and subsequent compression.

#### Example data

As example data we use temperature measurements in Kelvin, that fluctuate around room temperature (293 K), with some sensor noise.

```javascript
[
  293.046,
  293.054,    
  293.158,
  293.08,
  ...
  293.024
]
```

#### Results

| Compression | bytes | bytes/sample |
| -------- | -------- | -------- |
| JSON     | 7772     | 7.772    |
| CBOR     | 8955     | 8.955    |
| Deflate  | 2254     | 2.254    |

In this case, the CBOR encoding actually makes things worse. Each sample value with a fractional component is encoded as an IEEE double with prefix, so 9 bytes, whereas the decimal textual representation is actually smaller.

But the compression step reduces the size per sample to just over 2 bytes. For a larger sample array we could expect even better compression, limited of course by the amount of sensor noise, which is more than 8 bits in this case.

### Index and timestamp sequence compression

The last type of data we have to deal with in typical telemetry data is sequences of timestamps. Sequences of timestamps are *almost always* monotoneously increasing, and *very frequently* increasing by a roughly constant amount from one sample to the next.

In addition to timestamp data, we have the index sequences which we use to deal with sparse fields. These are *always* integers, *always* strictly monotoneously increasing, and *very frequently* increasing by a constant amount of 1.

A third example for monotoneously increasing numerical values are counters.

Given how frequent roughly linearly increasing sequences are, it it useful to perform an additional transformation on such data before binary encoding and compressing them.

### Delta encoding for linear sequences

The transformation is extremely simple. Instead of storing `N` numerical values, we store an initial value and `N-1` deltas d<sub>i</sub> = x<sub>i+1</sub> - x<sub>i</sub>. A *completely regular* linear sequence will be transformed into a constant sequence, which of course compresses extremely well.

```javascript
[0,1,2,3,....,999]
```

becomes

```javascript
{ r: 0, d: [0,...,0] }
```

More interesting is the case of a sequence of timestamps. Timestamps typically have some jitter, since the timer does not fire at exactly regular intervals. But even in this case, the delta compression is a huge win, since regular compression algorithms such as deflate have difficulties "seeing the pattern" without delta compression.

#### Example data

As example data, I have chosen an array of unix timestamp values generated by running a repeating timer on my development machine. The increase is roughly regular, but not entirely, due to timer imprecison or load on the machine.

```javascript
[1523565937887,1523565938890,1523565939895,1523565940896,1523565941896]
```

Becomes

```javascript
{
  reference: 1523565937887,
  deltas: [1003,1005,1001,1000]
}
```

Obviously, both CBOR encoding and compression will be improved for the sequence of roughly regular, small deltas.

#### Results

| Compression | bytes | bytes/sample |
| -------- | -------- | -------- |
| JSON     | 14001    | 14.001   |
| CBOR     |  9003    |  9.003   |
| Deflate  |  3502    |  3.502   |
| Δ-Deflate  | 672    | 0.672    |

As you can see from the results, CBOR encoding improves storage efficiency a bit. Every value is stored as a long integer. Normal deflate reduces the size to 3.5 bytes per sample, but given how regular the data is this is not very good.

The delta encoding before deflate allows the deflate algorithm to see the regularity, improving the compression to less than a byte per sample.

### When to choose delta encoding

It is probably possible to come up with a good heuristic on when to use delta compression. But what I do is to just try both normal and delta compression whenever compressing an array containing just numerical values, and choose whatever approach yields better compression.

Index sequences always benefit from delta compression, so for those I force using delta compression.

Sequences containing non-numerical values or NaN can not be delta-compressed at all.

### Noop compression

In some cases (completely random data or very small arrays), compression has no benefit at all. In these cases I use noop compression.

## Storage on IPFS and deduplication

As the last step, each compressed index or value array is stored *separately* in IPFS as an [IPLD](https://github.com/ipld/ipld) DAG object.

This has several benefits. In the long term, it is possible to filter fields by name before reconstructing an object from IPFS, which will make certain analytical queries very efficient.

But the most immediate benefit is that IPFS content adressed storage will of course *deduplicate* arrays that occur multiple times. This sometimes happens for value arrays, but *almost always* happens for index arrays.

Taking for example this transformed data:

```javascript
{
  children: {
    type: {
      values: [[0,1,2], ['start', 'pause', 'stop']]
    },
    pos: {
      values: [[1], ['na']],
      children: {
        lat: {
          values: [[0, 2], [10,20]]
        },
        lon: {
          values: [[0, 2], [10,0]]
        }
      }
    }
  }
}
```

The index array `[0,2]` occurs two times and will be deduplicated when storing in IPFS. The benefit of this is not as high as it might seem, since index arrays already compress very well due to delta compression. But there are certain cases where the deduplication can be very beneficial, and it comes for free anyway with a content-addressed storage system such as IPFS.

## Overall results

To evaluate the overall efficiency, I generated some data that closely mimics typical telemetry, but has a known amount of noise which gives an optimum compression ratio for comparison. The samples contain a certain amount of unchanging data such as tags to identify the sensor. This is something you frequently see in the wild.

The results are summarized in this table. All values are in bytes.

|                | size     | size/sample     |
| -------------- | -------- | --------------- |
| JSON           | 16265164 |	 162.65164    |
| CBOR           | 12268523 |	 122.68523    |
| JSON/deflate	 | 870921   | 	 8.70921      |
| CBOR/deflate   | 923066   |         9.23066 |
| Columns/CBOR/deflate | 162721 |	  1.62721 |
| " + dedup      |	 161769 |	      1.61769 |
| Theoretical optimum | 112500 |	    1.125 |

As expected, the JSON representation is incredibly inefficient due to the schemaless nature of JSON. The size is dominated by the field names.

Encoding in CBOR (which happens anyway when you store on IPFS) improves things quite a bit, but we are still a factor of 100 from the theoretical optimum.

The next two rows are a naive approach of just compressing the JSON using the deflate algorithm or encoding using CBOR and then deflating.

We get within a factor of 8 of the theoretical optimum, which is decent.

The last two columns are the encoding and compression scheme described above, with and without deduplication due to IPFS. Deduplication in this case does not have a large benefit, since the index sequences that benefit most from deduplication are very regular.

We do get within a factor of **1.5** to the theoretical optimum despite using the relatively simple deflate algorithm.

### Size per sample (all cases)

![Size per sample (all cases)](https://i.imgur.com/egHO6hu.png)

### Size per sample (just compressed)

![Size per sample (just compressed)](https://i.imgur.com/uIK6vHM.png)

# Implementation

The described encoding and compression scheme can of course be implemented in any language. The current implementation is in typescript and runs on nodejs.

The core package is a very small library that performs the transformation of an array of arbitrary json objects
into the columnar representation and back, and some utilities to encode and compress columns.

The CLI package provides two simple CLI tools to compress and store / retrieve and decompress data. It requires access to an IPFS API, which you of course have if you are running an IPFS node locally.

## Compression

The compression CLI tool allows compressing data from a file containing an arbitrary JSON array, or from standard input. It will transform the data, store it on IPFS, and return the hash of the root of the stored data structure.

```shell
$ npm run compress -- -v test.json

Input size       16265628
Compressed size  216089
Ratio            75.27
IPFS dag objects 9
zdpuAw2qxLonhUCukfSsRbhWnKfEJZCKPw2k5UAqHXF39kkuF
```

## Decompression

Decompression takes a single argument: the hash of a compressed IPFS dag object. It retrieves the data, decompresses it, applies the columnar to row transform, and produces the original JSON.

## License

The code is licensed under the [Apache 2](https://www.apache.org/licenses/LICENSE-2.0.html) license.

