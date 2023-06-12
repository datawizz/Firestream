import { useState, useEffect } from 'react';
import dynamic from "next/dynamic";
import { ScatterData, PlotData } from 'plotly.js';
const Plot = dynamic(() => import("react-plotly.js"), { ssr: false });


interface Data {
    device_id: string;
    magnitude: number;
    index: number;
}

const debug = false

// const _address = 'ws://localhost:8069'
//const _address = 'ws://websocket-middleware.default.svc.cluster.local:8080'
const _address = 'ws://localhost:8002'


const ChartPage = () => {
    const [realtimeData, setRealtimeData] = useState<Record<string, Data[]>>({});

    useEffect(() => {
        const ws = new WebSocket(_address);

        ws.onopen = () => {
            console.log('WebSocket connection opened');
        };

        ws.onmessage = (event) => {
            const newData: Data = JSON.parse(event.data).value;
            if (debug) {
                console.log('Received data:', newData);
            }
            setRealtimeData(prevData => {
                const newDataList = prevData[newData.device_id]
                    ? [...prevData[newData.device_id], newData]
                    : [newData];
                const last100DataList = newDataList.slice(-25);
                return { ...prevData, [newData.device_id]: last100DataList };
            });
        };

        ws.onclose = () => {
            console.log('WebSocket connection closed');
        };

        ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };

        return () => {
            ws.close();
        };
    }, []);

    const datapoint: Partial<ScatterData>[] = Object.entries(realtimeData).map(([deviceId, dataList]) => ({
        x: dataList.map(d => d.index),
        y: dataList.map(d => d.magnitude),
        type: 'scatter',
        mode: 'lines',
        name: `Device ${deviceId}`
    }));



    return (
        <div className="chart-container">
            <Plot
                useResizeHandler={true}
                style={{ width: '100%', height: '100%' }}
                data={datapoint as ScatterData[]}
                layout={{
                    title: 'Real-time Line Chart',
                    xaxis: { title: 'Index' },
                    yaxis: { title: 'Magnitude' },
                    showlegend: true,
                    legend: {
                        x: 1,
                        y: 1,
                        orientation: 'h'
                    },
                    // width: '100%',
                    // height: '100%',
                    autosize: true
                }}
            />
        </div>
    );
};

export default ChartPage;
