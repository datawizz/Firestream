import os
import logging
from time import sleep, time
import pytest

def setup_logging():
    path = "./logs"
    if not os.path.exists(path):
        os.mkdir(path)
    logging.basicConfig(
        filename=os.path.join(path, "test_logging.log"),
        filemode="a",
        level=logging.DEBUG,
    )

def log_time(loop):
    s = f"The time is now {time()} on loop {loop} and {logging.getLoggerClass().root.handlers[0]}"
    logging.debug(s)
    print(s)

@pytest.mark.parametrize('loop', range(10))
def test_log(loop):
    setup_logging()
    log_time(loop)
    sleep(0.1)
