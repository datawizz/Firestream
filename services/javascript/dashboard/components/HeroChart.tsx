import { useState, useEffect } from 'react';
import dynamic from "next/dynamic";
const Plot = dynamic(() => import("react-plotly.js"), { ssr: false, })
import { createConnection } from '../pages/api/socket';

interface Data {
    device_id: string;
    magnitude: number;
}

interface Props {
    data: Data[];
}

// const ChartPage: React.FC<Props> = ({ data }) => {
//     const [chartData, setChartData] = useState<Data[]>(data);

//     return (
//         <div>
//             <Plot
//                 data={[
//                     {
//                         x: chartData.map(d => d.device_id),
//                         y: chartData.map(d => d.magnitude),
//                         type: 'scatter',
//                         mode: 'lines+points',
//                         marker: { color: 'red' },
//                     },
//                 ]}
//                 layout={{
//                     title: 'Line Chart',
//                     xaxis: { title: 'Device ID' },
//                     yaxis: { title: 'Magnitude' },
//                 }}
//             />
//         </div>
//     );
// };

// ChartPage.getInitialProps = async () => {
//     const connection = createConnection('127.0.0.1:8080');
//     return {
//         data: await connection.getData()
//     }
// }

// Static Plot
// const ChartPage: React.FC = () => {
//     return (
//         <div>
//             <Plot
//                 data={[{
//                     x: ['device1', 'device2', 'device3'],
//                     y: [1, 2, 3],
//                     type: 'scatter',
//                     mode: 'lines+points',
//                     marker: { color: 'red' },
//                 },
//                 ]}
//                 layout={{
//                     title: 'Line Chart',
//                     xaxis: { title: 'Device ID' },
//                     yaxis: { title: 'Magnitude' },
//                 }}
//             />
//         </div>
//     );
// };


// const ChartPage = () => {
//     const data = Array.from({ length: 5 }, (_, index) => {
//         const x = Array.from({ length: 10 }, (_, i) => i + 1);
//         const y = x.map(() => Math.random());
//         return {
//             x,
//             y,
//             type: 'scatter',
//             mode: 'lines+points',
//             name: `Series ${index + 1}`,
//             marker: { color: `rgb(${Math.floor(Math.random() * 255)}, ${Math.floor(Math.random() * 255)}, ${Math.floor(Math.random() * 255)})` },
//         }
//     });

//     return (
//         <div>
//             <Plot
//                 data={data}
//                 layout={{
//                     title: 'Random Line Chart',
//                     xaxis: { title: 'X-axis' },
//                     yaxis: { title: 'Y-axis' },
//                     legend: { x: 1, y: 1 },
//                 }}
//             />
//         </div>
//     );
// };


//Working!
const numSeries = 5;
const numPoints = 60;
const updateInterval = 100;

const ChartPage = () => {
    const [data, setData] = useState(generateInitialData());

    useEffect(() => {
        const intervalId = setInterval(() => {
            setData(prevData => {
                const newData = prevData.map(series => {
                    const newSeries = { x: [...series.x], y: [...series.y], type: 'scatter', mode: 'lines+points' };
                    newSeries.x.shift();
                    newSeries.y.shift();
                    newSeries.x.push(newSeries.x[newSeries.x.length - 1] + 1);
                    newSeries.y.push(Math.random() * 100);
                    return newSeries;
                });
                return newData;
            });
        }, updateInterval);
        return () => clearInterval(intervalId);
    }, []);

    return (
        <div className="chart-container">
            <Plot
                useResizeHandler={true}
                style={{ width: "100%", height: "100%" }}
                data={data}
                layout={{
                    title: 'Line Chart',
                    xaxis: { title: 'X' },
                    yaxis: { title: 'Y' },
                    showlegend: true,
                    legend: {
                        x: 1,
                        y: 1
                    },
                    width: "100%",
                    height: "100%",
                    autosize: true
                }}
            />
        </div>
    );
}

function generateInitialData() {
    const data = [];
    for (let i = 0; i < numSeries; i++) {
        const series = { x: [], y: [], type: 'scatter', mode: 'lines+points', name: `Series ${i + 1}` };
        for (let j = 0; j < numPoints; j++) {
            series.x.push(j);
            series.y.push(Math.random() * 100);
        }
        data.push(series);
    }
    return data;
}


export default ChartPage;
