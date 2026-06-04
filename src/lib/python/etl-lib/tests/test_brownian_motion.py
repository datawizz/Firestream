"""Unit tests for the standalone Brownian Motion helpers.

These tests bypass ``BrownianMotion_DataSource.make`` (which is currently
NotImplementedError pending ``DataModel.make_one``) and exercise the
working ``generate_gbm`` and ``generate_timestamps`` methods directly.
"""

from datetime import timedelta

import numpy as np
import pandas as pd

from etl_lib.source import BrownianMotion_DataSource


def _make_source(seconds: int = 60, interval_s: int = 1) -> BrownianMotion_DataSource:
    start = pd.Timestamp("2024-01-01T00:00:00", tz="UTC")
    end = start + pd.Timedelta(seconds=seconds)
    source = BrownianMotion_DataSource.__new__(BrownianMotion_DataSource)
    source.start = start
    source.end = end
    source.intervals = [timedelta(seconds=interval_s)]
    source.seed = 42
    np.random.seed(source.seed)
    interval = min(source.intervals)
    ms_period = interval.total_seconds() * 10**9
    source.length = int((source.end.value - source.start.value) / ms_period)
    return source


def test_generate_gbm_returns_series_of_expected_length():
    source = _make_source(seconds=10, interval_s=1)
    series = source.generate_gbm(mu=0.1, sigma=0.1, p0=100)
    assert isinstance(series, pd.Series)
    assert len(series) == source.length
    assert series.iloc[0] == 100
    assert not series.isnull().any()


def test_generate_gbm_is_deterministic_with_seed():
    a = _make_source(seconds=20, interval_s=1).generate_gbm(p0=50)
    b = _make_source(seconds=20, interval_s=1).generate_gbm(p0=50)
    pd.testing.assert_series_equal(a, b)


def test_generate_timestamps_monotonic_within_interval():
    source = _make_source(seconds=30, interval_s=1)
    timestamps = source.generate_timestamps()
    assert isinstance(timestamps, pd.Series)
    assert len(timestamps) == source.length
    diffs = timestamps.diff().dropna()
    assert (diffs > 0).all()
