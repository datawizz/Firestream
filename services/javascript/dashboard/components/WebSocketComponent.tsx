import React, { useEffect } from 'react';

const WebSocketComponent: React.FC = () => {
    useEffect(() => {
        const ws = new WebSocket('ws://localhost:8080');
        ws.onopen = () => {
            console.log('WebSocket connection opened');
        };
        ws.onmessage = (event) => {
            console.log('Received data:', event.data);
        };
        ws.onclose = () => {
            console.log('WebSocket connection closed');
        };
        ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };
        return () => {
            ws.close();
        }
    }, []);
    return <div>WebSocket connected and data logged to console</div>;
}

export default WebSocketComponent;
