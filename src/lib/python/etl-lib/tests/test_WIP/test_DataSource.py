"""
Load a local file from the data directory and send to a S3 bucket
Use the boto 3 library to stay away from a dependency on spark?
"""

from typing import List
from datetime import timedelta, datetime

from dataclasses import dataclass, field


from etl_lib import DataModel, DataContext, DataFactory
from etl_lib.source import BrownianMotion_DataSource


def test():
    @dataclass
    class _Point(DataModel):
        time: datetime = field(metadata={"stochastic": True})
        price: float = field(metadata={"stochastic": True})

    @dataclass
    class _Trade(DataModel):
        ticker: str
        point: _Point

    @dataclass
    class _Trades(DataModel):
        ticker: str
        trades: List[_Trade]

    model = _Trade
    start = "2020-01-01 00"
    end = "2020-01-02 00"
    intervals = [timedelta(seconds=100)]

    with DataFactory(start=start, end=end, intervals=intervals) as factory:
        factory.make(model=_Trade, source=BrownianMotion_DataSource)
        factory.make(model=_Point, source=BrownianMotion_DataSource)
        factory.make(model=_Trades, source=_Trade)


if __name__ == "__main__":
    test()
