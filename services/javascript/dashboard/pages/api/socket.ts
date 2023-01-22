import { useWebSocket } from 'react-use-websocket';

export function createConnection(url: string) {
    const [sendMessage, lastMessage, readyState] = useWebSocket(url);

    return {
        send: sendMessage,
        onopen: (event: Event) => {
            console.log('WebSocket connection opened');
        },
        onclose: (event: CloseEvent) => {
            console.log('WebSocket connection closed');
        },
        onerror: (event: Event) => {
            console.error(`WebSocket error: ${event}`);
        },
        onmessage: (event: MessageEvent) => {
            console.log(event.data);
        },
        readyState: readyState
    }
}
