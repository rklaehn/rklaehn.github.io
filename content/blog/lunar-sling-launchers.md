+++
title = "Lunar Sling Launchers"
date = 2026-02-21
description = "Using rotating slings to fling payloads off the Moon without rockets."
[taxonomies]
tags = ["space", "propulsion"]
+++

With the renewed interest in lunar settlement and using the Moon as a source of raw materials, we are finally getting to the point where we need a way to launch bulk mass off the Moon. The most popular concept to do this is a mass driver, basically a magnetic catapult, as described in [The High Frontier][high-frontier] by Gerard K. O'Neill.

But there is a less well known concept that is much more realistic in the near term. A sling launcher. This is a device so simple that it could have been built by medieval siege engineers. But due to material science progress and lunar vacuum, it is possible to build one that reaches lunar orbital or even lunar escape velocity.

A sling launcher looks very much like a tower crane, with a rigid boom extending in both directions, each end carrying a tether with a hook.

Just like with a tower crane, the booms can rotate around the vertical axis with modest speed.

Unlike with tower cranes, the cable is made out of high strength fabric and can be reeled out much more than the total height of the crane. A realistic near term system would have a tether length in the kilometer range.

![Sling launcher on the lunar surface](/images/tether-catapult.svg)

To use the system, you would attach an identical mass to each end, then spin up the system and slowly reel out the tether. The purpose of the system is to launch mass to an Earth/Moon [lagrange point][lagrange], specifically EML1, so from now on we will do all calculations with a target release velocity in the range of the lunar escape velocity, 2,376 m/s.

![Sling launcher spinning up with payloads attached](/images/tether-catapult-2.svg)

## Tip acceleration

Once fully reeled out, the tether will be almost horizontal and the masses will experience substantial acceleration. The acceleration is given by

$$a = \frac{v^2}{r}$$

where *v* is the tip velocity and *r* is the tether length. The following table gives acceleration at the tip, rotation rate and tether slope. Tether slope is the ratio of lunar gravity to outward acceleration, i.e. the vertical drop per unit of horizontal distance.

| Tether length | Tip acceleration | Tip acceleration (g) | Rotation rate (rpm) | Tether slope |
|---------------|------------------|----------------------|---------------------|--------------|
| 2 km          | 2,823 m/s²       | 288                  | 11.3                | 0.058%       |
| 10 km         | 565 m/s²         | 57.5                 | 2.27                | 0.29%        |
| 100 km        | 56.5 m/s²        | 5.8                  | 0.23                | 2.9%         |

Unlike with compact spin launcher systems such as [SpinLaunch][spinlaunch], the tip acceleration is easily survivable even for complex satellites at modest tether lengths.

For a 100 km tether, the tip acceleration would be survivable for humans. But in the near term the main purpose of a propellantless launch system is to launch bulk materials or satellites, so a radius between 2 km and 10 km seems optimal.

## Feasibility with modern materials

Compared to a tower crane, the tether is under a lot of load. For a notional 1000 kg payload, the force at the tip is 2,823 kN at 2 km, 565 kN at 10 km, and 56.5 kN at 100 km.

For a long tether, the tether cross section is small, while for a shorter tether we need a thick tether cross section. At the extreme end of a tiny system like [SpinLaunch][spinlaunch], we need a massive rigid arm instead of a tether.

It turns out that the mass ratio for a tapered rotating tether only depends on the ultimate tensile strength (N/m²) and the density (kg/m³), the tip velocity and a safety factor, but not on r:

$$\frac{M_{tether}}{M_{payload}} = \sqrt{\pi} \cdot V_r \cdot e^{V_r^2} \cdot \text{erf}(V_r)$$

where $V$ is the tip velocity and $V_c = \sqrt{2\sigma/\rho}$ is the characteristic velocity of the tether material, with $\sigma$ being the tensile strength and $\rho$ the density.

The following table gives the mass ratio for several materials at lunar escape velocity with a safety factor of 1.5:

| Material       | Tensile strength | Density   | $V_c$     | $V_r$ | Mass ratio |
|----------------|------------------|-----------|-----------|--------|------------|
| [Dyneema SK99][sk99] | 4.1 GPa    | 970 kg/m³   | 2,374 m/s | 1.00   | 4.1        |
| [Toray T1100G][t1100g] | 7.0 GPa  | 1,790 kg/m³ | 2,283 m/s | 1.04   | 4.7        |
| [Toray T1200][t1200] | 8.0 GPa    | ~1,800 kg/m³ | 2,434 m/s | 0.98   | 3.7        |
| Piano wire     | 2.8 GPa          | 7,850 kg/m³ | 690 m/s   | 3.45   | 873,000    |

While even the best steel wire is completely hopeless, there are materials that are common and commercially available in thousands of ton quantities that can survive the required forces with a generous safety factor.

There are no carbon nanotubes needed. You can buy a rope with the required strength at your [local walmart][walmart-dyneema].

## Concept of operations in detail

To launch a payload for an initial system, you would attach a payload and a counterweight to both arms of the launcher while at rest.

You would then increase the rotation speed up to a very modest value, in the same ballpark as the release rotation rate, and then slowly reel out the tether until the target radius is reached.

Once the tether is reeled out to the target radius and the target tip speed is reached, the payload is released at a precise time and flies away tangentially.

## Release dynamics

At this point we have multiple options. 

### Simultaneous counterweight release

To minimize the loads on the hub, you would release both the payload and the counterweight at exactly the same time.

{{ svg(path="/images/release-simultaneous.svg") }}

There would be only short transient forces on the hub, so this release mode would even work for a tower consisting of a rocket stage standing on the lunar surface without anchoring.

### Simultaneous release of two payloads

For certain locations on the Moon, it is possible to release two payloads simultaneously, so that both make it to EML1.

{{ svg(path="/images/release-two-payloads.svg") }}

### Counterweight as second payload

Once your hub is solidly anchored or tied down, you would have the option to let the system be briefly unbalanced and release the second weight as a payload half a rotation later.

{{ svg(path="/images/release-delayed.svg") }}

The forces involved are large, but nothing that a large industrial bearing can't handle. The main downside of this is that it would tip over the tower unless it is tied down.

### Keep counterweight

The fourth option would be to keep the counterweight attached and accept a transient imbalance until balance can be restored by extending the now unloaded payload side and reeling in the counterweight side.

{{ svg(path="/images/release-keep-counterweight.svg") }}

This is the most flexible option, since you can launch single payloads without loss of efficiency. But it is also the option that requires the most solid anchoring of the hub.

## Anchoring the hub

The most efficient release options require the hub to survive side forces. E.g. for a 1000 kg payload and a 10 km tether, an asymmetric release would create a side force of 565 kN for 13.2 s until the second payload can be released.

An unanchored lightweight tower would tip over. A straightforward option would be to tie down the tower with screw anchors in lunar regolith to unlock the more efficient release options.

In addition, if the tower base is an expended lander stage, you might want to fill the empty propellant tanks with regolith to add mass.

![Anchored hub with screw anchors](/images/anchored-hub.svg)

Alternatively, you could simply bury the lower half of the tower in regolith using a bulldozer. This provides both added mass and passive lateral resistance without any specialized hardware.

![Buried hub](/images/buried-hub.svg)

## The sling launcher as an energy storage device

While the masses are being reeled out, the rotating tether and tip masses serve as an energy storage system. The sling launcher can be powered directly from a modestly sized photovoltaic system and gain energy over many hours.

For a system with two 1,000 kg tip masses at lunar escape velocity, the kinetic energy of the tip masses is:

$$E_{tips} = 2 \times \frac{1}{2} m v^2 = 1000 \times 2376^2 \approx 1.6\text{ MWh}$$

Despite the velocity decreasing linearly toward the hub, the tapered tether carries roughly the same amount of energy as the tip masses due to its higher total mass. The total system energy is therefore roughly 2x the tip mass energy, or about 3.1 MWh. Adding a modest factor for motor and bearing losses, the energy consumption per launch is about 3.9 MWh.

A 1 MW solar array could spin up the system in about 14,000 seconds, or roughly 3.9 hours. This is a very modest photovoltaic installation in the blazing lunar sun.

Reeling in the tether could be done faster, so a total launch rate of 4 per Earth day could be achieved.

## Azimuth flexibility

Unlike a stationary linear accelerator which has a fixed orientation, a sling launcher can launch into any launch azimuth.

The azimuth precision depends on precise timing of the release. However, with a large tether radius the timing requirements are modest. A 10ms release timing error translates to a 0.14° azimuth angle error, which can be corrected with 5.7 m/s.

You could get to sub-millisecond release precision using [frangible bolts][frangible-bolts], but a mechanical quick release system with <10ms accuracy is very feasible as well. The problem is certainly much less difficult than for a short radius system like SpinLaunch.

## Tether dynamics

Releasing the weights instantly would cause the force on the tether tip to instantly decrease. This would lead to the tether rapidly recoiling, which would lead to a wave traveling to the hub.

You could drastically reduce the wear on the tether tip by engineering a soft release mechanism. You would also need active damping at the hub to cancel out waves induced by the release. But both of these subjects are beyond the scope of this blog post.

## Trajectories

I used the excellent [nyx][nyx] and [anise][anise] rust crates to write a trajectory simulation for a sling launcher system. For an area in the equatorial region of the lunar far side, it is possible to launch towards EML1 without any major course corrections, both prograde and retrograde. Minor course corrections could be done with a cold gas RCS, or you could have a tug at EML1 pick up completely passive payloads.

For an initial system there is even a location on the equator of the lunar far side where you can launch to EML1 with the same velocity both retrograde and prograde.

![Sling launcher demonstration](/images/sling-launcher-demo.gif)

*Simulation using [nyx-space](https://nyxspace.com), visualization using [bevy](https://bevyengine.org), coded with Claude*

## Mass budget of an initial system

The best way to design an initial system would be to use a large lander such as Lunar Starship as the tower base.

Lunar Starship does not have a tank in the tip, so the entire payload section could be dedicated to a deployable sling launcher: two booms, two reels, a hub, motors and of course the tether itself, as well as a small ~100kW PV system for initial operations.

The tether weight for a system for two 1000 kg payloads would be 8200 kg of Dyneema SK99.

A detailed mass budget is beyond the scope of this blog post, but it seems very plausible that you could fit the entire system into the ~100t mass budget of a Lunar Starship with a lot of margin.

Unused mass budget could be used for deployable ground mounted PV panels, screw anchors, tethers, and empty payload containers/bags.

## Payloads

Payloads could be something as complex as a LOX tank with a small cold gas RCS system, or as simple as regolith in [bulk bags][bulk-bag] without any propulsion system that would rely on precise release timing and a tug at EML1 to pick them up.

I will leave military applications as an exercise for the reader.

![The Moon is a Harsh Mistress](/images/tmiahm-cover.jpg)

## Mass ROI

With a 1000 kW external power supply, the system could do 4 launches per Earth day, or 56 launches per lunar day. Once tied down, each launch is 2000 kg in 2 payloads. So the system would pay back its landed payload mass in less than a single lunar day, and could pay back >100x its payload mass over a 10 year lifetime.

## References

- Gerard K. O'Neill, *[The High Frontier: Human Colonies in Space][high-frontier]*, William Morrow and Company, 1976.
- Geoffrey A. Landis, "[Analysis of a Lunar Sling Launcher][landis-sling]," *Journal of the British Interplanetary Society (JBIS)*, Vol. 58, pp. 294-297, 2005.
- Geoffrey A. Landis, "Reactionless Orbital Propulsion Using Tether Deployment," *Acta Astronautica*, Vol. 26, No. 5, pp. 307-312, 1992.

[high-frontier]: https://en.wikipedia.org/wiki/The_High_Frontier:_Human_Colonies_in_Space
[lagrange]: https://en.wikipedia.org/wiki/Lagrange_point
[landis-sling]: https://www.researchgate.net/publication/267800402_Analysis_of_a_Lunar_Sling_Launcher
[sk99]: https://www.dsm.com/dyneema/en_GB/our-products/fiber/sk99.html
[t1100g]: https://www.toraycma.com/products/carbon-fiber/
[t1200]: https://www.toraycma.com/toray-develops-torayca-t1200-the-ultra-high-strength-carbon-fiber/
[walmart-dyneema]: https://www.walmart.com/c/kp/dyneema-line
[nyx]: https://nyxspace.com/
[anise]: https://docs.rs/anise
[spinlaunch]: https://en.wikipedia.org/wiki/SpinLaunch
[bulk-bag]: https://www.walmart.com/ip/FIBC-Bulk-Bag-Super-Sack-2200lbs-35-L-x-35-W-x-43-H-Duffle-Top-Flat-Bottom-Creative-American-Home/1098010449
[tmiahm]: https://en.wikipedia.org/wiki/The_Moon_Is_a_Harsh_Mistress
[frangible-bolts]: https://en.wikipedia.org/wiki/Frangible_nut