# talespin
Talespin (inspired by [Dixit](https://en.wikipedia.org/wiki/Dixit_(board_game))) is a simple-to-learn card game that emphasizes creativity and abstract thinking.


<p align="center">
<a href="https://talespin.live">
<img src="static/gameplay.png" width="800"><br>
<strong>talespin.live</strong>
</a>
</p>

The server is written in Rust using the Axum web framework and a number of awesome tools (anyhow, dashmap, serde_json, tokio). The client-side is written in Svelte with Skeleton UI. Communicaton between the client-side and server-side is primarily handled through websockets with a central server that stores state in-memory, meaning that any latency is introduced by the network.

## Development

Install frontend dependencies:

```bash
npm install
```

Run the frontend locally:

```bash
npm run dev
```

Run the backend:

```bash
cd dixit-server && cargo run
```
