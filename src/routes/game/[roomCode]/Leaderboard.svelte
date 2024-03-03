<script lang="ts">
	import type { PlayerInfo } from '$lib/types';

	export let players: { [key: string]: PlayerInfo } = {};
	// export let name = '';
	export let stage = '';
	export let activePlayer = '';
	export let pointChange: { [key: string]: number } = {};
	export let roundNum: number;
	let sortedPlayersList: string[] = [];

	$: {
		sortedPlayersList = Object.keys(players).sort((a, b) => {
			return players[b].points - players[a].points;
		});
	}
</script>

<div class="flex w-80/10 justify-center">
	<div class="card light p-4">
		<h2 class="text-xl">Round {roundNum}</h2>
		<div>
			{#each sortedPlayersList as player, i}
				<div class="flex space-between w-44">
					<div class="flex-auto">
						{i + 1}.
						<span
							class={`${stage == 'ActiveChooses' && player === activePlayer ? 'boujee-text' : ''} `}
							>{player}</span
						>
						{#if !players[player].connected}
							<span class="text-error-500">(afk)</span>
						{/if}

						{#if stage === 'Joining' || ((stage === 'PlayersChoose' || stage === 'Voting') && player !== activePlayer) || stage === 'Results'}
							{#if players[player].ready}
								<span>🟢</span>
							{:else}
								<span>🔴</span>
							{/if}
						{/if}
					</div>
					<div class="font-right">
						{#if stage === 'Results' && typeof pointChange[player] === 'number' && pointChange[player] !== 0}
							<span class="opacity-50">(+{pointChange[player]})</span>
						{/if}
						{players[player].points}
					</div>
				</div>
			{/each}
		</div>
	</div>
</div>

<style>
</style>
