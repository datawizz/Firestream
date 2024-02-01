from time import sleep, time
import logging
import os


def test_log():
    path = "./logs"
    if not os.path.exists(path):
        os.mkdir(path)
    logging.basicConfig(
        filename=os.path.join(path, "test_logging.log"),
        filemode="a",
        level=logging.DEBUG,
    )

    loop = 0
    while True:
        s = f"The time is now {time()} on loop {loop} and {logging.getLoggerClass().root.handlers[0]}"
        logging.debug(s)
        print(s)
        loop += 1
        sleep(0.1)
        if loop > 5:
            break


if __name__ == "__main__":
    test_log()
