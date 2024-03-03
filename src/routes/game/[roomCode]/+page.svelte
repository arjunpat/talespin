<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { getToastStore } from '@skeletonlabs/skeleton';

	import { nameStore } from '$lib/store';
	import type { PlayerInfo } from '$lib/types';
	import GameServer from '$lib/gameServer';

	import Joining from './Joining.svelte';
	import Leaderboard from './Leaderboard.svelte';
	import ActiveChooses from './ActiveChooses.svelte';
	import PlayersChoose from './PlayersChoose.svelte';
	import Voting from './Voting.svelte';

	onDestroy(() => {
		if (gameServer) {
			rejoin = false;
			gameServer.close();
		}
	});

	let name = '';
	let roomCode = '';
	let gameServer: GameServer;
	let players: { [key: string]: PlayerInfo } = {};
	let stage: string = 'Joining';
	let activePlayer = '';
	let displayImages: string[] = [];
	let playerOrder: string[] = [];
	let selectedImage = '';
	let description = '';
	let progressBarPercentage: number | null = null;
	let rejoin = false;
	let toastStore = getToastStore();

	// results
	let card_to_player: { [key: string]: string } = {};
	let card_to_voters: { [key: string]: string[] } = {};
	let activeCard = '';

	nameStore.subscribe((value) => {
		name = value;
	});

	onMount(() => {
		roomCode = $page.params.roomCode;

		if (name === '') {
			goto('/');
		}

		gameServer = new GameServer();
		gameServer.joinRoom(roomCode, name);
		gameServer.onclose(() => {
			if (rejoin) {
				gameServer.joinRoom(roomCode, name);
			}
		});
		gameServer.addMsgHandler((data: any) => {
			console.log(data);

			if (data.RoomState) {
				players = data.RoomState.players;
				stage = data.RoomState.stage;
				activePlayer = data.RoomState.active_player || '';
				if (!rejoin) {
					toastStore.trigger({
						message: '👋 Connected to room!',
						autohide: true,
						timeout: 2500
					});
					rejoin = true;
				}
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
				description = data.PlayersChoose.description;
			} else if (data.BeginVoting) {
				stage = 'Voting';
				displayImages = data.BeginVoting.center_cards;
				description = data.BeginVoting.description;
				selectedImage = '';
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
			} else if (data.ErrorMsg) {
				toastStore.trigger({
					message: '😭 ' + data.ErrorMsg,
					autohide: true,
					timeout: 2500
				});
			} else if (data.InvalidRoomId) {
				rejoin = false;
				toastStore.trigger({
					message: '💔 Invalid Room Code!',
					autohide: true,
					timeout: 2500
				});
				console.log('hello');
				goto('/');
			}
		});
	});

	function playersChoose() {
		gameServer.playersChoose(selectedImage);
	}

	function vote() {
		gameServer.vote(selectedImage);
	}
</script>

<div class="p-5 w-full">
	{#if stage === 'Joining'}
		<Joining {name} {gameServer} {players} {roomCode} />
	{:else if stage === 'ActiveChooses'}
		<ActiveChooses {displayImages} {activePlayer} {name} {gameServer} />
	{:else if stage === 'PlayersChoose'}
		<PlayersChoose {displayImages} {name} {activePlayer} {gameServer} {description} />
	{:else if stage === 'Voting'}
		<Voting {displayImages} {activePlayer} {name} {gameServer} {description} />
	{:else}
		<div>
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

				<button class="btn variant-filled mt-5" on:click={() => gameServer.ready()}>Ready</button>
			{/if}
		</div>
	{/if}
	{#if stage !== 'Joining'}
		<div class="mt-28"></div>
		<Leaderboard {players} {name} {stage} {activePlayer} />
	{/if}
</div>

<style>
</style>
