# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Converts a GEOFON miniSEED clip to the Parquet file of the Dombois (2001) gallery item.

Usage (Python 3.10+, pyarrow):

    python -I convert.py CLIP.mseed START END OUT.parquet

It reads miniSEED 2 records with Steim-1 or Steim-2 data (SEED manual 2.4, chapter 8 and
appendix B), joins the records of one channel in time order, refuses gaps and overlaps,
keeps the samples from START (inclusive) to END (exclusive, ISO 8601 UTC), removes the
window's mean as an integer offset (DC removal, as Dombois 2001 §2 did before
audification), and writes `time` (timestamp, ms, UTC) and `z` (int32 counts). The source,
licence and offset go into the file's metadata.

Written for SONARITHM from the SEED documentation; it is not a general miniSEED reader.
"""

from __future__ import annotations

import datetime as dt
import hashlib
import struct
import sys

import pyarrow as pa
import pyarrow.parquet as pq


def btime(raw: bytes) -> dt.datetime:
    year, day, hour, minute, second, _, tenth_ms = struct.unpack(">HHBBBBH", raw)
    start = dt.datetime(year, 1, 1, tzinfo=dt.timezone.utc)
    return start + dt.timedelta(
        days=day - 1, hours=hour, minutes=minute, seconds=second, microseconds=tenth_ms * 100
    )


def signed(value: int, bits: int) -> int:
    return value - (1 << bits) if value & (1 << (bits - 1)) else value


def steim(data: bytes, count: int, version: int) -> list[int]:
    """Decodes `count` samples of Steim-1 or Steim-2 frames."""
    diffs: list[int] = []
    first = last = None
    for frame in range(len(data) // 64):
        words = struct.unpack(">16I", data[frame * 64 : frame * 64 + 64])
        nibbles = words[0]
        for i in range(1, 16):
            code = (nibbles >> (30 - 2 * i)) & 3
            word = words[i]
            if frame == 0 and i == 1:
                first = signed(word, 32)
                continue
            if frame == 0 and i == 2:
                last = signed(word, 32)
                continue
            if code == 0:
                continue
            if code == 1:
                diffs += [signed((word >> s) & 0xFF, 8) for s in (24, 16, 8, 0)]
            elif version == 1 and code == 2:
                diffs += [signed((word >> s) & 0xFFFF, 16) for s in (16, 0)]
            elif version == 1 and code == 3:
                diffs.append(signed(word, 32))
            elif version == 2 and code == 2:
                dnib = word >> 30
                if dnib == 1:
                    diffs.append(signed(word & 0x3FFFFFFF, 30))
                elif dnib == 2:
                    diffs += [signed((word >> s) & 0x7FFF, 15) for s in (15, 0)]
                elif dnib == 3:
                    diffs += [signed((word >> s) & 0x3FF, 10) for s in (20, 10, 0)]
                else:
                    raise ValueError("bad Steim-2 dnib")
            elif version == 2 and code == 3:
                dnib = word >> 30
                if dnib == 0:
                    diffs += [signed((word >> s) & 0x3F, 6) for s in (24, 18, 12, 6, 0)]
                elif dnib == 1:
                    diffs += [signed((word >> s) & 0x1F, 5) for s in (25, 20, 15, 10, 5, 0)]
                elif dnib == 2:
                    diffs += [signed((word >> s) & 0xF, 4) for s in (24, 20, 16, 12, 8, 4, 0)]
                else:
                    raise ValueError("bad Steim-2 dnib")
            else:
                raise ValueError("bad Steim nibble")
    if first is None or len(diffs) < count:
        raise ValueError("short Steim data")
    samples = [first]
    for d in diffs[1:count]:
        samples.append(samples[-1] + d)
    if samples[-1] != last:
        raise ValueError("Steim reverse integration constant does not match")
    return samples


def records(blob: bytes):
    offset = 0
    while offset < len(blob):
        head = blob[offset : offset + 48]
        net = head[18:20].decode().strip()
        sta = head[8:13].decode().strip()
        loc = head[13:15].decode().strip()
        cha = head[15:18].decode().strip()
        start = btime(head[20:30])
        count, factor, multiplier = struct.unpack(">Hhh", head[30:36])
        # A time correction (0.0001 s units) that the activity flags say is not applied yet
        # would move every sample time; refuse it rather than guess (SEED 2.4, fixed header).
        (correction,) = struct.unpack(">i", head[40:44])
        if correction and not head[36] & 0x02:
            raise ValueError(f"record at {start} has an unapplied time correction of {correction / 1e4} s")
        data_offset, blockette = struct.unpack(">HH", head[44:48])
        encoding = length = None
        while blockette:
            kind, following = struct.unpack(">HH", blob[offset + blockette : offset + blockette + 4])
            if kind == 1000:
                encoding, order, exponent = struct.unpack(
                    ">BBB", blob[offset + blockette + 4 : offset + blockette + 7]
                )
                if order != 1:
                    raise ValueError("little-endian records are not supported")
                length = 1 << exponent
            blockette = following
        if length is None or encoding not in (10, 11):
            raise ValueError(f"unsupported record (encoding {encoding})")
        if factor > 0 and multiplier > 0:
            rate = factor * multiplier
        elif factor > 0 and multiplier < 0:
            rate = -factor / multiplier
        elif factor < 0 and multiplier > 0:
            rate = -multiplier / factor
        else:
            rate = 1 / (factor * multiplier)
        data = blob[offset + data_offset : offset + length]
        yield (net, sta, loc, cha), start, rate, steim(data, count, 1 if encoding == 10 else 2)
        offset += length


def utc(text: str) -> dt.datetime:
    """An ISO 8601 time as UTC: a trailing Z (which Python 3.10 cannot parse) or any
    offset is honoured, and a time without one is taken as UTC."""
    if text.endswith(("Z", "z")):
        text = text[:-1] + "+00:00"
    time = dt.datetime.fromisoformat(text)
    if time.tzinfo is None:
        return time.replace(tzinfo=dt.timezone.utc)
    return time.astimezone(dt.timezone.utc)


def main(path: str, start_text: str, end_text: str, out: str) -> None:
    blob = open(path, "rb").read()
    window = [utc(t) for t in (start_text, end_text)]
    recs = sorted(records(blob), key=lambda r: r[1])
    ids = {r[0] for r in recs}
    rates = {r[2] for r in recs}
    if len(ids) != 1 or len(rates) != 1:
        raise ValueError(f"expected one channel at one rate, got {ids} {rates}")
    rate = rates.pop()
    step = dt.timedelta(seconds=1 / rate)
    times: list[dt.datetime] = []
    values: list[int] = []
    for _, begin, _, samples in recs:
        if times:
            gap = (begin - (times[-1] + step)).total_seconds()
            if abs(gap) > 0.5 / rate:
                raise ValueError(f"gap or overlap of {gap} s at {begin}")
        for i, v in enumerate(samples):
            times.append(begin + i * step)
            values.append(v)
    keep = [(t, v) for t, v in zip(times, values) if window[0] <= t < window[1]]
    if not keep:
        raise ValueError("no samples in the window")
    offset = round(sum(v for _, v in keep) / len(keep))
    t0 = keep[0][0]
    # Sample times from the first kept sample and the nominal rate, to the millisecond.
    stamps = [t0 + i * step for i in range(len(keep))]
    net, sta, loc, cha = recs[0][0]
    metadata = {
        "snr.source": f"GEOFON Data Centre (1993): GEOFON Seismic Network. GFZ. doi:10.14470/TR560404; {net}.{sta}.{loc}.{cha}",
        "snr.licence": "CC-BY-4.0",
        "snr.request": f"fdsnws/dataselect {net}.{sta}.{loc or '--'}.{cha} {start_text} to {end_text}",
        "snr.raw_sha256": hashlib.sha256(blob).hexdigest(),
        "snr.dc_offset_counts": str(offset),
        "snr.sample_rate_hz": str(rate),
    }
    table = pa.table(
        {
            "time": pa.array(stamps, type=pa.timestamp("ms", tz="UTC")),
            "z": pa.array([v - offset for _, v in keep], type=pa.int32()),
        }
    ).replace_schema_metadata(metadata)
    pq.write_table(
        table,
        out,
        # Brotli, not zstd: the data layer reads it without its `zstd` feature.
        compression="brotli",
        compression_level=11,
        use_dictionary=False,
        column_encoding={"time": "DELTA_BINARY_PACKED", "z": "DELTA_BINARY_PACKED"},
    )
    print(f"{out}: {len(keep)} samples at {rate} Hz from {t0.isoformat()}, DC offset {offset} counts")


if __name__ == "__main__":
    main(*sys.argv[1:5])
