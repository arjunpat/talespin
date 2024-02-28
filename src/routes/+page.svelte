<script lang="ts">
	import { fade } from 'svelte/transition';
	let name = '';
	let roomCode = '';
	let joinGameClicked = false;

	const host = 'localhost:8080';

	function setupSocket(ws: WebSocket) {
		ws.onopen = () => {
			console.log('Connected to server');
		};

		ws.onmessage = (event) => {
			let data = JSON.parse(event.data);
			console.log('ServerMsg:', data);
		};

		ws.onclose = () => {
			console.log('Disconnected from server');
		};
	}

	function createGame() {
		console.log('Creating game');
		let ws = new WebSocket(`ws://${host}/ws`);

		ws.onopen = () => {
			ws.send(
				JSON.stringify({
					CreateRoom: {
						name: name
					}
				})
			);
		};

		ws.onmessage = (event) => {
			let data = JSON.parse(event.data);
			console.log('ServerMsg:', data);
			if (data.RoomState) {
				console.log('Created room');
			}
		};
	}

	function joinGame() {
		if (joinGameClicked) {
			console.log('Joining game');
			let ws = new WebSocket(`ws://${host}/ws`);
			ws.onopen = () => {
				ws.send(
					JSON.stringify({
						JoinRoom: {
							name: name,
							room_id: roomCode
						}
					})
				);
			};

			ws.onmessage = (event) => {
				let data = JSON.parse(event.data);
				console.log('ServerMsg:', data);
				if (data.RoomState) {
					console.log('Joined room');
				}
			};
		} else {
			joinGameClicked = true;
		}
	}
</script>

<div class="max-w-md mx-auto p-4 mt-5">
	<h1 class="h1">
		<span
			class="bg-gradient-to-br from-red-500 to-yellow-500 bg-clip-text text-transparent box-decoration-clone"
			>Play Dixit!</span
		>
	</h1>

	<div class="card p-4 mt-8">
		<div class="mb-4">
			<label for="name">Name:</label>
			<input
				type="text"
				id="name"
				bind:value={name}
				class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
			/>
		</div>

		{#if joinGameClicked}
			<div transition:fade>
				<label for="roomCode">Room Code:</label>
				<input
					type="text"
					id="roomCode"
					bind:value={roomCode}
					class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 mb-4 leading-tight focus:outline-none focus:shadow-outline"
				/>
			</div>
		{/if}

		<div class="flex justify-between mb-4">
			<button on:click={() => createGame()} class="btn variant-filled">Create Game</button>
			<button on:click={() => joinGame()} class="btn variant-filled">Join Game</button>
		</div>
	</div>
</div>
