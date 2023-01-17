import pandas as pd
import numpy as np
import plotly.express as px
import plotly.graph_objects as go
import dash
from dash import dcc
from dash import html
from dash.dependencies import Input, Output
import os

import os
from dash import Dash, html, dcc
import plotly.express as px
import pandas as pd

from collections import deque 
      
# Declaring deque 
queue = deque([0])

BOOTSTRAP_SERVERS = "kafka.default.svc.cluster.local:9092"
SOURCE_TOPIC = "python_markov_chain"

# debug = False if os.environ["DASH_DEBUG_MODE"] == "False" else True

app = Dash(__name__)

server = app.server


def collect_data(topic: str):
    """
    Read data from the provided topic
    """

    c = Consumer(
        {
            "bootstrap.servers": BOOTSTRAP_SERVERS,
            "group.id": "mygroup",
            "auto.offset.reset": "earliest",
        }
    )

    c.subscribe([topic])

    while True:
        msg = c.poll(1.0)

        if msg is None:
            continue
        if msg.error():
            print("Consumer error: {}".format(msg.error()))
            continue

        print("Received message: {}".format(msg.value().decode("utf-8")))

    c.close()





# code and plot setup
# settings
pd.options.plotting.backend = "plotly"
countdown = 20
# global df

# sample dataframe of a wide format
np.random.seed(4)
cols = list("abc")
X = np.random.randn(60, len(cols))
df = pd.DataFrame(X, columns=cols)
df.iloc[0] = 0




# plotly figure
fig = df.plot(template="plotly_dark")

app = dash.Dash("Device Measurements", update_title=None)
app.layout = html.Div(
    [
        html.H1("Streaming data from Kafka"),
        dcc.Interval(
            id="interval-component",
            interval=1 * 1000,
            n_intervals=0,  # in milliseconds, updates every second
        ),
        dcc.Graph(id="graph"),
    ]
)

# Define callback to update graph
@app.callback(Output("graph", "figure"), [Input("interval-component", "n_intervals")])
def streamFig(value):

    global df

    Y = np.random.randn(1, len(cols))
    df2 = pd.DataFrame(Y, columns=cols)
    df = df.append(df2, ignore_index=True)  # .reset_index()
    df.tail()
    df3 = df.copy()
    df3 = df3.cumsum()
    df3 = df3.tail(60)  # Keep the last 60 seconds on the chart
    fig = df3.plot(template="plotly_dark")
    # fig.show()
    return fig


if __name__ == "__main__":
    app.run_server(
        port=8050,
        dev_tools_ui=True,  # debug=True,
        dev_tools_hot_reload=True,
        threaded=True,
    )
