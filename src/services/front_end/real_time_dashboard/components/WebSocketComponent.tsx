import React, { useEffect, useState } from 'react';

const debug = false

// const _address = 'ws://localhost:8069'
// const _address = 'ws://websocket-middleware.default.svc.cluster.local:8080'
const _address = 'ws://localhost:8002'


const MessageList: React.FC<{ messages: string[] }> = ({ messages }) => {
    return (
        <div style={{ width: '80%', maxWidth: '600px', margin: '0 auto' }}>
            {messages.map((message, index) => (
                <div
                    key={index}
                    style={{ wordBreak: 'break-all', marginBottom: '10px' }}
                >
                    {message}
                </div>
            ))}
        </div>
    );
};



const WebSocketComponent: React.FC = () => {
    const [messages, setMessages] = useState<string[]>([]);

    useEffect(() => {
        const ws = new WebSocket(_address);

        ws.onopen = () => {
            console.log('WebSocket connection opened');
            // Send a ping message every 30 seconds to keep the connection alive
            // setInterval(() => {
            //     console.log('Sending pong to server...');
            //     ws.send('pong');
            // }, 5000);
        };

        ws.onmessage = (event) => {
            if (debug) {
                console.log('Received data:', event.data);
            }
            setMessages((prevMessages) => [...prevMessages, event.data]);
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

    return <MessageList messages={messages.slice(-3)} />;
};

export default WebSocketComponent;
