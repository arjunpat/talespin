<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { nameStore } from '$lib/store';
	import GameServer from '../../../gameServer';
	import { goto } from '$app/navigation';
	import { ProgressBar } from '@skeletonlabs/skeleton';

	interface PlayerInfo {
		active: boolean;
		points: number;
		ready: boolean;
	}
	interface Deadline {
		issued: number;
		deadline: number;
		active: boolean;
	}

	let name = '';
	let room_id = '';
	let gameServer: GameServer;
	let players: { [key: string]: PlayerInfo } = {};
	let stage: string = 'Joining';
	let activePlayer = '';
	let displayImages: string[] = [];
	let playerOrder: string[] = [];
	let selectedImage = '';
	let descriptionBox = '';
	let progressBarPercentage: number | null = null;

	// results
	let card_to_player: { [key: string]: string } = {};
	let card_to_voters: { [key: string]: string[] } = {};
	let activeCard = '';

	nameStore.subscribe((value) => {
		name = value;
	});

	onMount(() => {
		room_id = $page.params.room_id;

		if (name === '') {
			goto('/');
		}

		gameServer = new GameServer();
		gameServer.joinRoom(room_id, name);
		gameServer.addMsgHandler((data: any) => {
			console.log(data);

			if (data.RoomState) {
				players = data.RoomState.players;
				stage = data.RoomState.stage;
				activePlayer = data.RoomState.active_player || '';
				if (data.RoomState.player_order.length == 0) {
					playerOrder = Object.keys(players);
				} else {
					playerOrder = data.RoomState.player_order;
				}
			} else if (data.StartRound) {
				stage = 'ActiveChooses';
				displayImages = data.StartRound.hand;
			} else if (data.PlayersChoose) {
				stage = 'PlayersChoose';
				displayImages = data.PlayersChoose.hand;
				descriptionBox = data.PlayersChoose.description;
				// let answerDeadline = data.PlayersChoose.deadline;
				// let issued = Date.now();

				// function updateProgressBar() {
				// 	progressBarPercentage = (Date.now() - issued) / (answerDeadline - issued);

				// 	if (
				// 		typeof progressBarPercentage === 'number' &&
				// 		progressBarPercentage <= 1 &&
				// 		progressBarPercentage >= 0
				// 	) {
				// 		setTimeout(updateProgressBar, 50);
				// 	} else {
				// 		progressBarPercentage = null;
				// 	}
				// }

				// updateProgressBar();
			} else if (data.BeginVoting) {
				stage = 'Voting';
				displayImages = data.BeginVoting.center_cards;
				descriptionBox = data.BeginVoting.description;

				// let answerDeadline = data.BeginVoting.deadline;
				// let issued = Date.now();
				selectedImage = '';

				// function updateProgressBar() {
				// 	progressBarPercentage = (Date.now() - issued) / (answerDeadline - issued);

				// 	if (
				// 		typeof progressBarPercentage === 'number' &&
				// 		progressBarPercentage <= 1 &&
				// 		progressBarPercentage >= 0
				// 	) {
				// 		setTimeout(updateProgressBar, 50);
				// 	} else {
				// 		progressBarPercentage = null;
				// 	}
				// }

				// updateProgressBar();
			} else if (data.Results) {
				stage = 'Results';
				displayImages = Object.values(data.Results.player_to_current_card);

				card_to_player = {};
				Object.entries(<{ [key: string]: string }>data.Results.player_to_current_card).forEach(
					([player, card]) => {
						card_to_player[card] = player;
					}
				);

				card_to_voters = {};
				Object.entries(<{ [key: string]: string }>data.Results.player_to_vote).forEach(
					([player, card]) => {
						if (!card_to_voters[card]) {
							card_to_voters[card] = [];
						}
						card_to_voters[card].push(player);
					}
				);

				activeCard = data.Results.active_card;
			}
		});
	});

	function activePlayerChoose() {
		let card = selectedImage;
		let description = descriptionBox;
		gameServer.activePlayerChoose(card, description);
	}

	function playersChoose() {
		gameServer.playersChoose(selectedImage);
	}

	function vote() {
		gameServer.vote(selectedImage);
	}
</script>

<div class="p-5 w-full flex">
	<div class="flex-none w-40 h-full">
		<!-- <div class="flex-none"> -->
		<h2>Players</h2>
		{#each playerOrder as player}
			<div>
				<p>
					{#if activePlayer === player}
						<span>➤</span><span class="font-bold">{player}:</span>
					{:else}
						<span>{player}:</span>
					{/if}
					{players[player].points} points
				</p>
			</div>
		{/each}
	</div>
	<div>
		{#if stage === 'Joining'}
			<div>
				<button class="btn variant-filled" on:click={() => gameServer.startRound()}
					>Start Game</button
				>
			</div>
		{/if}

		{#if stage !== 'Results'}
			<section class="grid grid-cols-2 md:grid-cols-3 gap-4 mt-5">
				{#each displayImages as image}
					<a
						href=""
						class="group"
						on:click={() => {
							selectedImage = image;
						}}
					>
						<img
							class={`${selectedImage === image ? 'border-4 border-white' : ''} transition-all duration-150 ease-in-out group-hover:scale-110 group-hover:shadow-2xl group-focus:shadow-2xl rounded-lg`}
							src="../../assets/cards/{image}"
							alt="You can't play this game without the images!"
						/>
					</a>
				{/each}
			</section>
		{/if}

		{#if progressBarPercentage !== null}
			<progress value={progressBarPercentage} max={1} />
		{/if}

		{#if stage === 'ActiveChooses'}
			{#if activePlayer === name}
				<input
					type="text"
					placeholder="Description"
					bind:value={descriptionBox}
					class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline mt-5 mb-5"
				/>

				<button
					class="btn variant-filled"
					disabled={selectedImage === '' || descriptionBox === ''}
					on:click={activePlayerChoose}>Choose</button
				>
			{:else}
				<p>Waiting for <strong>{activePlayer}</strong> to choose</p>
			{/if}
		{/if}

		{#if stage === 'PlayersChoose'}
			<div class="mt-5">
				<ul>
					{#each playerOrder as player}
						{#if player !== activePlayer}
							<li>
								<span>
									{players[player].ready ? '✔️ ' : ''}
									{player}
								</span>
							</li>
						{/if}
					{/each}
				</ul>
			</div>

			<div class="mt-5">
				{#if activePlayer === name}
					<p>Waiting for players to choose their cards!</p>
				{:else}
					<p>
						Choose a card that <strong>{activePlayer}</strong> would put for "{descriptionBox}"
					</p>
					<button
						class="btn variant-filled"
						disabled={selectedImage === ''}
						on:click={playersChoose}>Choose</button
					>
				{/if}
			</div>
		{/if}

		{#if stage === 'Voting'}
			<div class="mt-5">
				<ul>
					{#each playerOrder as player}
						{#if player !== activePlayer}
							<li>
								<span>
									{players[player].ready ? '✔️ ' : ''}
									{player}
								</span>
							</li>
						{/if}
					{/each}
				</ul>
			</div>
			<div class="mt-5">
				{#if activePlayer === name}
					<p>Waiting for players to vote!</p>
				{:else}
					<p>Which card would <strong>{activePlayer}</strong> choose for "{descriptionBox}"</p>
					<button class="btn variant-filled" disabled={selectedImage === ''} on:click={vote}
						>Vote</button
					>
				{/if}
			</div>
		{/if}

		{#if stage === 'Results'}
			<!-- <section class="grid grid-cols-2 md:grid-cols-3 gap-10 mt-5"> -->
			<section class="grid grid-cols-2 md:grid-cols-3 gap-4 mt-5">
				{#each displayImages as image}
					<div
						style="position: relative;"
						class={`transition-all duration-150 ease-in-out group-hover:scale-110 group-hover:shadow-2xl group-focus:shadow-2xl rounded-lg ${activeCard == image ? 'border-4 border-white' : ''}`}
					>
						<img
							src="../../assets/cards/{image}"
							alt="You can't play this game without the images!"
						/>
						{#if card_to_voters[image]}
							<!-- {#each card_to_voters[image] as voter} -->
							<div style="position: absolute; bottom: 36px; right: 12px;" class="bg-black">
								{card_to_voters[image].join(', ')}
							</div>
							<!-- {/each} -->
						{/if}
						<div class="bg-black p-1">{card_to_player[image]}'s card</div>
					</div>
				{/each}
			</section>

			<button class="btn variant-filled mt-5" on:click={() => gameServer.startRound()}
				>Next Round</button
			>
		{/if}
	</div>
</div>

<style>
</style>
