const host = 'localhost:8080';
import { nameStore } from "$lib/store";

class GameServer {
    _ws: WebSocket;
    onmessage_handler: ((data: object) => void)[] = [];
    message_queue: string[] = [];
    opened = false;
    joined_room = '';
    name = '';

    constructor() {
        this._ws = new WebSocket(`ws://${host}/ws`);
        this.setupSocket();
        nameStore.subscribe(value => {
            this.name = value;
        })
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
            if (data.RoomStatus) {
                this.joined_room = data.RoomStatus.room_id;
            }
            this.opened = true;
            this.onmessage_handler.forEach(handler => {
                handler(data);
            });
        };
        this._ws.onclose = () => {
            console.log('disconnected');
            this._ws = new WebSocket(`ws://${host}/ws`);
            if (this.joined_room !== '') {
                this.message_queue.unshift(JSON.stringify({
                    JoinRoom: {
                        name: this.name,
                        room_id: this.joined_room
                    }
                }))
            }

            this.setupSocket();
        }
    }

    ping() {
        console.log('sending ping');
        this.send({
            Ping: {}
        });
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

    startRound() {
        this.send({
            StartRound: {}
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
}

export default GameServer;
