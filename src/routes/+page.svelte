<script lang="ts">
	import { fade } from 'svelte/transition';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { nameStore } from '$lib/store';
	import { get } from 'svelte/store';
	import GameServer from '../gameServer';

	let name = get(nameStore) || '';
	let roomCode = '';
	let joinGameClicked = false;
	let gameServer: GameServer;

	$: nameStore.set(name);

	onMount(() => {
		gameServer = new GameServer();

		gameServer.addMsgHandler((data: any) => {
			if (data.RoomState) {
				gameServer.close();
				goto(`/game/${data.RoomState.room_id}`);
			}
			console.log(data);
		});
	});

	function createGame() {
		console.log('Creating game');

		gameServer.createRoom(name);
	}

	function joinGame() {
		if (joinGameClicked) {
			goto(`/game/${roomCode}`);
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
