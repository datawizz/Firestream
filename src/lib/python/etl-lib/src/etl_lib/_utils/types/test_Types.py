from typing import List
from datetime import timedelta, datetime

from dataclasses import dataclass, field


from etl_lib import DataModel, DataContext, DataFactory
from etl_lib.source import BrownianMotion_DataSource

from etl_lib.types import StochasticFloat_Type, EventTime_Type, String_Type, Array_Type


def test():
    @dataclass
    class _Point(DataModel):
        time: EventTime_Type
        price: StochasticFloat_Type

    @dataclass
    class _Trade(DataModel):
        ticker: String_Type
        point: _Point

    @dataclass
    class _Trades(DataModel):
        ticker: String_Type
        trades: Array_Type[_Trade]

    model = _Trade
    start = "2020-01-01 00"
    end = "2020-01-02 00"
    intervals = [timedelta(seconds=100)]

    with DataFactory(start=start, end=end, intervals=intervals) as factory:
        factory.make(model=_Trade, source=BrownianMotion_DataSource)
        factory.make(model=_Point, source=BrownianMotion_DataSource)
        factory.make(model=_Trades, source=_Trade)
        factory.run()


if __name__ == "__main__":
    test()
