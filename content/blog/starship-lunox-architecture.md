+++
title = "Starship Lunar Architecture with Tether-Launched LunOX"
date = 2026-05-10
description = "A lunar architecture using Starship and LunOX launched from a sling tether."
[taxonomies]
tags = ["space", "propulsion"]
+++

# Propellantless lunar launcher applications

In [previous][prev-1] [posts][prev-2] I have described a lunar sling catapult that can launch mass to [EML1][eml1-wiki] without propellant expenditure and with very low cost.

[Other people][elon-tweet] are in favour of electromagnetic mass drivers. See this recent blog post from [Casey Handmer][handmer-blog].

Whatever solution wins, in the medium term we can have a cheap propellantless way to launch mass from the lunar surface to EML1.

So let's take a look at what we could do with this capability. I am not interested in the far future, but something that could realistically happen in a decade.

[Producing oxygen from lunar soil][lunar-isru] isn't hard. It is just very energy intensive. But whether via nuclear reactors or photovoltaics, the Moon is going to be a very energy rich environment. So let's assume we got an industrial scale [LOX][lox-wiki] production plant on the Moon.

# A LunOX pipeline to EML1

So we got cheap LOX on the Moon, and a "pipeline" that can send LOX cheaply to EML1. That means cheap LOX in EML1. How could we take advantage of this to make moving mass *to* the Moon cheaper?

Both Starship and [other promising cislunar transportation systems][blue-moon] rely on refueling instead of having dedicated stages for each leg. For both Starship (LOX/[CH4][ch4-wiki]) and Stoke/Blue Origin (LOX/[LH2][h2-wiki]), the vast majority of fuel mass is oxygen.

So while CH4 is hard to produce on the Moon, let's see how far we get by just using lunar oxygen and the tether LOX pipeline to EML1.

The current *public* plans for lunar Starship are surprisingly vague, but what is known is that there will be a lunar Starship variant without the capability to reenter the Earth atmosphere. It is also known that there will be one or possibly two propellant depots involved, one in [LEO][leo-wiki] and one closer to the Moon.

Starting with these building blocks and taking advantage of cheap EML1 LOX in every possible way, we end up with the following architecture:

We have two propellant depots. One in a low Earth orbit with a suitable inclination to be reachable from both Cape Canaveral and Starbase. And one in EML1, which is the destination of our LOX pipeline.

The LEO depot gets supplied by Starship tankers that carry only CH4. The EML1 depot gets supplied by the tether sling catapult.

Starships carrying payloads launch to the LEO depot, refuel with as much LOX and CH4 needed to reach the EML1 depot, plus some excess CH4. They then immediately depart to EML1.

At the EML1 depot, they unload their payload as well as most of the excess CH4. They keep just enough CH4 for a burn that lowers the perigee enough to touch the Earth atmosphere. Their LOX tanks get filled almost completely.

The stay at the EML1 depot will be longer than at the LEO depot, since the payload has to be unloaded.

After departure, the Starship performs the perigee lowering burn and then lowers the apogee by aerobraking until it matches the LEO propellant depot.

Reaching the LEO depot, it unloads almost all LOX and takes on a tiny bit of CH4 for the reentry burn. It then performs the reentry burn and returns to the launch site.

At the EML1 depot, the payload from three incoming Starships gets combined into a single Moon-bound Starship. The Moon-bound Starship gets the excess CH4 from the three inbound Starships and enough LOX for the lunar landing.

It then departs the EML1 depot and lands on the Moon with enough excess CH4 to make it back to the EML1 depot empty.

After unloading the payload and refilling with LOX corresponding to the excess CH4, it launches back to the EML1 depot.

<figure>
<p align="center">
  <a href="/images/lunox-architecture.svg" target="_blank" title="Click to open in isolation (full hover tooltips)">
    <img src="/images/lunox-architecture.svg" alt="Mission architecture diagram: Earth, LEO, EML1, Moon with vehicles cycling between them" style="max-width: 100%;"/>
  </a>
</p>
<figcaption style="text-align: center; font-size: 1.05em; font-style: italic; padding: 0.6em 1em; margin-top: 0.5em; border-left: 3px solid #888;">Mission architecture: cargo Starships shuttle Earth ↔ EML1, lunar Starship shuttles EML1 ↔ Moon, tether delivers LunOX to EML1.</figcaption>
</figure>


The end result is very promising. The exact numbers will vary depending on exact Starship capabilities. But the end result is a *large* reduction of the number of tanker flights needed to get a very substantial payload to the Moon.

# Launch windows

From Earth to the LEO depot you have one or two almost instantaneous launch windows per day, more if some orbit phasing is acceptable.

From LEO to EML1 and back you have two pretty wide launch windows when the LEO depot orbit crosses the lunar orbital plane.

From EML1 to the Moon and back you can start at any time with some minor delta-v variations.

<figure>
<p align="center">
  <img src="/images/sling-launcher-demo.gif" alt="Trajectory simulation showing Moon-to-EML1 ballistic launches" style="max-width: 100%;"/>
</p>
<figcaption style="text-align: center; font-size: 1.05em; font-style: italic; padding: 0.6em 1em; margin-top: 0.5em; border-left: 3px solid #888;">Moon → EML1 trajectories from a sling-launched payload. Simulation written in <a href="https://www.rust-lang.org/">Rust</a> using <a href="https://nyxspace.com">nyx-space</a> and <a href="https://bevyengine.org">bevy</a>; pre-built binaries on the <a href="https://github.com/rklaehn/solar-system-sim/releases/tag/v0.1.0">GitHub releases page</a>.</figcaption>
</figure>

So you would have the cargo Starships launch to the depot roughly at the right time for continuing the flight to EML1, and fill the less optimal times with tanker flights.

There is a slight variation in delta-v required for the LEO to EML1 trip due to differential nodal regression of the lunar orbital plane and the LEO depot orbital plane.

But the required plane change delta-v is small when combined with the very large apogee raise maneuver.

# Comparison with a fully Earth launched lunar architecture

I did a quick model for the no [LunOX][lunox-wiki] case. You end up needing a total number of 27 ships for the 450t to the lunar surface, so a factor of 3 more.

<figure>
<p align="center">
  <a href="/images/no-lunox-architecture.svg" target="_blank" title="Click to open in isolation (full hover tooltips)">
    <img src="/images/no-lunox-architecture.svg" alt="No-LunOX mission architecture: same Earth, LEO, EML1, Moon layout but with all propellant lifted from Earth and an empty cargo Starship return" style="max-width: 100%;"/>
  </a>
</p>
<figcaption style="text-align: center; font-size: 1.05em; font-style: italic; padding: 0.6em 1em; margin-top: 0.5em; border-left: 3px solid #888;">No-LunOX architecture: same two depots, but all propellant lifted from Earth — 24 prop tankers + 3 cargo Starships per cycle for the same 450 t lunar payload, with cargo Starships deadheading back empty.</figcaption>
</figure>

# Comparison with a hydrogen architecture

The only hydrogen powered vehicle under development that is capable of aerobraking and reentry is [Stoke Space Nova][stoke-nova], and there is very little data for it.

I made some educated guesses and ran the same architecture with H2 propellant. The result is a modest reduction of the number of tanker flights from 6 to 4, so we have 7 instead of 9 total launches.

A detailed analysis of a H2 based architecture is beyond the scope of this post and in any case would require more concrete launch vehicle data.

But I think it is safe to say that hydrogen gives you a slight improvement in overall efficiency at the expense of much more difficult handling (boiloff, volume etc.). *It is not a game changer!*

# Mars

The architecture can be easily adapted to other targets such as Mars. You would keep the LEO and EML1 depot, grab LunOX from EML1 to the LEO depot, then use that LunOX combined with CH4 from Earth for TMI and Mars landing.

<figure>
<p align="center">
  <a href="/images/mars-architecture.svg" target="_blank" title="Click to open in isolation">
    <img src="/images/mars-architecture.svg" alt="Mars architecture sketch: same Earth/LEO/EML1 hub with Moon in the top-left and Mars in the top-right" style="max-width: 100%;"/>
  </a>
</p>
<figcaption style="text-align: center; font-size: 1.05em; font-style: italic; padding: 0.6em 1em; margin-top: 0.5em; border-left: 3px solid #888;">Mars architecture: LunOX flows Moon → EML1 → LEO via dedicated LOX transporters, then 3 Mars Starships depart directly from LEO with LunOX-supplied LOX and Earth-launched CH4 for TMI and Mars landing.</figcaption>
</figure>

The efficiency of the LunOX shuttle from LEO to EML1 and back depends heavily on the tanker Starship empty mass.

To optimize this, you could have a special Starship variant that has the payload section converted to additional LOX tanks.

<figure>
<p align="center">
  <img src="/images/lox-shuttle.svg" alt="Stylized Starship LOX shuttle with LOX/CH4/LOX sandwich tank layout" style="max-width: 200px;"/>
</p>
<figcaption style="text-align: center; font-size: 1.05em; font-style: italic; padding: 0.6em 1em; margin-top: 0.5em; border-left: 3px solid #888;">LOX shuttle variant with LOX/CH4/LOX sandwich tank layout — the former payload bay becomes a second LOX tank, keeping the centre of gravity balanced during aerobraking.</figcaption>
</figure>

Such a ship could increase the efficiency of the LunOX shuttle from 60% to up to 71%, reducing the total number of flights from 9 to 8.

# Other alternatives

Another alternative for improving cislunar transportation is a [rotating tether (rotovator)][rotovator-wiki] in some Earth orbit as a momentum exchange mechanism. I spent quite some time thinking about this and might blog about this in the future. But with both low Earth orbit and sun synchronous orbit being increasingly crowded, it might just be that the time for this idea has come and gone.

# Summary

By using a propellantless transportation pipeline for lunar oxygen from the lunar surface to EML1, the payload efficiency of Earth launches can be improved by a factor of three for Moon- or Mars-bound spacecraft.

# Bleg

If anybody from Blue Origin or Stoke reads this and thinks a hydrogen architecture is superior to methane for this use case: I don't know. There just isn't enough data about the vehicles, so my H2 analysis is pretty vague. So please publish more data!

[prev-1]: /blog/lunar-sling-launchers/
[prev-2]: /blog/lunar-sling-launcher-tether-dynamics/
[elon-tweet]: https://x.com/elonmusk/status/2036299707446861904
[handmer-blog]: https://caseyhandmer.wordpress.com/2026/05/08/how-to-build-a-lunar-mass-driver/
[blue-moon]: https://www.blueorigin.com/blue-moon
[lunar-isru]: https://x.com/blueorigin/status/2041583026140762480
[stoke-nova]: https://www.stokespace.com/
[lox-wiki]: https://en.wikipedia.org/wiki/Liquid_oxygen
[ch4-wiki]: https://en.wikipedia.org/wiki/Liquid_methane
[h2-wiki]: https://en.wikipedia.org/wiki/Liquid_hydrogen
[lunox-wiki]: https://en.wikipedia.org/wiki/Lunar_resources
[eml1-wiki]: https://en.wikipedia.org/wiki/Lagrange_point
[leo-wiki]: https://en.wikipedia.org/wiki/Low_Earth_orbit
[rotovator-wiki]: https://en.wikipedia.org/wiki/Skyhook_(structure)