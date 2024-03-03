export const production = window.location.href.includes('talespin.live');

export const host = window.location.href.includes('talespin.live') ? 'api.talespin.live' : '127.0.0.1:8081';
export const http_host = `https://${host}`;
export const ws_host = `wss://${host}`;
export const ws_url = `${ws_host}/ws`;

class GameServer {
    _ws: WebSocket;
    onmessage_handler: ((data: object) => void)[] = [];
    message_queue: string[] = [];
    onclosehandler = () => { };

    constructor() {
        this._ws = new WebSocket(ws_url);
        this.setupSocket();
    }

    setupSocket() {
        this._ws.onopen = () => {
            console.log('connected');
            if (this.message_queue.length > 0) {
                this.message_queue.forEach(data_str => {
                    this._ws.send(data_str);
                });
                this.message_queue = [];
            }
        };
        this._ws.onmessage = (event) => {
            let data = JSON.parse(event.data.toString());
            this.onmessage_handler.forEach(handler => {
                handler(data);
            });
        };
        this._ws.onclose = () => {
            console.log('disconnected');
            this._ws = new WebSocket(ws_url);


            this.setupSocket();
            this.onclosehandler();
        }
    }


    send(data: object) {
        let data_str = JSON.stringify(data);
        if (this._ws.readyState === 1) {
            this._ws.send(data_str);
        } else {
            this.message_queue.push(data_str);
        }
    }

    createRoom(name: string) {
        this.send({
            CreateRoom: {
                name
            }
        })
    }

    joinRoom(room_id: string, name: string) {
        this.send({
            JoinRoom: {
                name,
                room_id
            }
        });
    }

    ready() {
        this.send({
            Ready: {}
        });
    }

    activePlayerChoose(card: string, description: string) {
        this.send({
            ActivePlayerChooseCard: {
                card,
                description
            }
        });
    }

    playersChoose(card: string) {
        this.send({
            PlayerChooseCard: {
                card
            }
        });
    }

    vote(card: string) {
        this.send({
            Vote: {
                card
            }
        });
    }

    addMsgHandler(func: (data: object) => void) {
        this.onmessage_handler.push(func);
    }

    close() {
        this._ws.close();
    }

    onclose(func: () => void) {
        this.onclosehandler = func;
    }
}

export default GameServer;
