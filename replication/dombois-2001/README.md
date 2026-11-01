# Dombois (2001): audification of seismograms

> **Source:** Dombois, F. (2001). Using audification in planetary seismology. *Proceedings of the 2001 International Conference on Auditory Display (ICAD)*, Espoo, Finland. [PDF](https://www.icad.org/Proceedings/2001/Dombois2001.pdf) (`dombois2001using`). Read from the proceedings PDF; SDR-31.
>
> **Data:** GEOFON Data Centre (1993): GEOFON Seismic Network. Deutsches GeoForschungsZentrum GFZ. [doi:10.14470/TR560404](https://doi.org/10.14470/TR560404). Licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) (the DOI's DataCite record); clipped, DC-offset removed and converted for SONARITHM (below). Chosen by the owner in [OD-31](../../docs/planning/open-decisions.md).

Dombois played seismograms as sound by resampling them 2,200 times faster. Near earthquakes then sound as sharp clicks, far ones as deep rumbles that rise in pitch, and waves that crossed the Earth's core have a harder attack. This group plays one near and one far earthquake recorded at the same station, so the site response is the same:

| Spec | Earthquake | Distance | Window | Samples | Plays |
|---|---|---|---|---|---|
| [petrinja-2020.json](petrinja-2020.json) | Mw 6.4 Petrinja, Croatia, 2020-12-29 11:19:55 UTC (GEOFON event gfz2020zocb) | About 4.4° | 10 min from 11:19:25 UTC | 12,000 | About 0.27 s |
| [kermadec-2021.json](kermadec-2021.json) | Mw 8.0 Kermadec Islands, 2021-03-04 19:28:36 UTC (gfz2021ektt) | About 160°, beyond the 103° of core-refracted waves | 60 min from 19:43:35 UTC | 72,000 | About 1.6 s |

Both are the vertical (BHZ) channel of GEOFON station MORC (Moravsky Beroun, Czech Republic; Streckeisen STS-2/N, 20 samples per second), fetched once from GEOFON's FDSN dataselect service.

## Published and reproduced parameters

"Reproduced" values are read from the committed render records in [manifests.json](manifests.json).

| Parameter | Published (section) | Reproduced | Match |
|---|---|---|---|
| Method | Time compression by resampling, no frequency doubling (§2) | The `audify` mark: the series resampled to the output rate, not pitch-shifted (SDR-31, ADR-0030) | Yes |
| Speed-up | 2,200, chosen by test runs (§2) | `audify.k` = 2200, the seismic preset `SEISMIC_K` | Yes |
| Component | The vertical z-component (§2) | BHZ, the vertical channel | Yes |
| Dynamic range | 32-bit records reduced to 16 bit, linearly (normalized) for single events, with automatic gain control for day records (§2) | Linear: the series is normalized to its peak (the `audify` encoder), then rendered at 24 bit at the SDR-23 level; no gain control | Yes for single events; day records are not reproduced |
| DC removal | "When needed", as preprocessing (§2) | The window's mean, an integer offset of counts, removed in `convert.py` (−14 counts for Petrinja, −299 for Kermadec), recorded in each file's metadata | Yes |
| Near and far | A quake at 3° lasts minutes, one at 143° more than half an hour; at 2,200 × that is half a second to a few seconds; near ones are "sharp clicks", far ones "deep rumbles" (§3) | 10 minutes at about 4.4° and 60 minutes at about 160° play for about 0.27 s and 1.6 s | Timing yes; the sound is not compared by listening (below) |
| Data | Not stated: records from seismological stations, among them Kobe 1995 at Matsushiro and at Nana, Peru (§3) | Two earthquakes at one GEOFON station | Different data (below) |

## Deviations

- **Data.** The paper does not name its archive, and its 1995 examples were not available as open data with a stated licence. This group uses two recent earthquakes at one GEOFON station, chosen to span "near" and "far" (TODO(verify): whether Dombois's own records are available under an open licence).
- **Length.** At k = 2200 the near quake plays for only about 0.27 s, short for careful listening. The k is Dombois's own, written in each spec (`audify.k`, §2: "a factor of 2.200 suited best"), not SDR-44's default. Without `k`, SDR-44 (INFER) would put the top of the band at 4 kHz: k = 2 × 4000 / 20 = 400, so Petrinja would play for 1.5 s and Kermadec for 9 s, at frequencies 5.5 times lower. The gallery keeps the published value; whether a slower default suits listeners better is for the HP-21 listening check, not this item.
- **Band.** At 20 samples per second and k = 2200 the series sounds from about 3.7 Hz (Petrinja) or 0.6 Hz (Kermadec) up to 22 kHz, so part of each band is outside hearing, which the compile reports (`SNR-ENG-461`). Most of a seismogram's energy, below about 1 Hz, lands below 2.2 kHz.
- **Timestamps.** Sample times are the first sample's time plus multiples of 50 ms, written to the millisecond, so the Kermadec record's 49.9 ms start offset is truncated to 49 ms.
- **Listening.** No listening check has been made of these renders; the owner's HP-21 covers the audification presets.

## Reproducing the data

`convert.py` (Python with pyarrow) decodes the miniSEED, checks each record's Steim integration constants, refuses gaps and overlaps, keeps the window, removes the mean and writes Parquet (brotli, delta-encoded, 24 KB and 85 KB):

```bash
curl -o petrinja.mseed "https://geofon.gfz.de/fdsnws/dataselect/1/query?network=GE&station=MORC&location=--&channel=BHZ&starttime=2020-12-29T11:19:25&endtime=2020-12-29T11:29:25"
python -I convert.py petrinja.mseed 2020-12-29T11:19:25 2020-12-29T11:29:25 data/petrinja-2020-morc-bhz.parquet
```

The Kermadec file is made the same way with the window 2021-03-04T19:43:35 to 2021-03-04T20:43:35. The downloads were 35,840 bytes (SHA-256 `31f53e406f76e43431eafdd954357b447f75756dce4efa30b504cea1386a714f`) and 120,320 bytes (`118513fc3d051a1764fbde253af1f12a378066b056da5b97d6f78fd727659b64`) on 2026-11-01; each file's metadata keeps its download's SHA-256. The station metadata of the request marks the network and the station `restrictedStatus="open"` and names the DOI above, whose DataCite record gives the licence.

## Licences

The two data files are the GEOFON Data Centre's data under CC BY 4.0, modified as above (REUSE.toml credits them). The specs and records are CC BY 4.0 like the rest of the gallery, and `convert.py` is MIT OR Apache-2.0.
