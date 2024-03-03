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
	import Results from './Results.svelte';

	// connection information
	let name = '';
	let roomCode = '';
	let gameServer: GameServer;
	let rejoin = false;

	// game state
	let players: { [key: string]: PlayerInfo } = {};
	let stage: string = 'Joining';
	let activePlayer = '';
	let description = '';
	let roundNum = 0;

	// UI state
	let displayImages: string[] = [];
	let selectedImage = '';

	// results
	let playerToCurrentCard: { [key: string]: string } = {};
	let playerToVote: { [key: string]: string } = {};
	let activeCard = '';
	let pointChange: { [key: string]: number } = {};

	// store
	let toastStore = getToastStore();
	nameStore.subscribe((value) => (name = value));

	onDestroy(() => {
		if (gameServer) {
			rejoin = false;
			gameServer.close();
		}
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
				roundNum = data.RoomState.round;
				if (!rejoin) {
					toastStore.trigger({
						message: '👋 Connected to room!',
						autohide: true,
						timeout: 2500
					});
					rejoin = true;
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
				playerToCurrentCard = data.Results.player_to_current_card;
				playerToVote = data.Results.player_to_vote;
				activeCard = data.Results.active_card;
				pointChange = data.Results.point_change;
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
</script>

<div class="pt-10 w-full">
	<div class="flex flex-row justify-center">
		<div>
			{#if stage !== 'Joining'}
				<div class="p-5">
					<Leaderboard {players} {stage} {pointChange} {activePlayer} {roundNum} />
				</div>
			{/if}
		</div>
		<div>
			{#if stage === 'Joining'}
				<Joining {name} {gameServer} {players} {roomCode} />
			{:else if stage === 'ActiveChooses'}
				<ActiveChooses {displayImages} {activePlayer} {name} {gameServer} />
			{:else if stage === 'PlayersChoose'}
				<PlayersChoose {displayImages} {name} {activePlayer} {gameServer} {description} />
			{:else if stage === 'Voting'}
				<Voting {displayImages} {activePlayer} {name} {gameServer} {description} />
			{:else if stage === 'Results'}
				<Results {displayImages} {gameServer} {playerToCurrentCard} {playerToVote} {activeCard} />
			{/if}
		</div>
		<!-- {#if stage !== 'Joining'}
		<div class="mt-28"></div>
		<Leaderboard {players} {name} {stage} {activePlayer} />
		{/if} -->
	</div>
</div>

<style>
</style>
