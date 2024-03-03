<script lang="ts">
	import type { PlayerInfo } from '$lib/types';

	export let players: { [key: string]: PlayerInfo } = {};
	export let name = '';
	export let stage = '';
	export let activePlayer = '';
	let sortedPlayersList: string[] = [];

	$: {
		sortedPlayersList = Object.keys(players).sort((a, b) => {
			return players[b].points - players[a].points;
		});
	}
</script>

<div class="flex w-80/10 justify-center">
	<div class="card light p-4">
		<h2>Points</h2>
		<div>
			{#each sortedPlayersList as player, i}
				<div class="flex space-between w-52">
					<div class="flex-auto">
						{i + 1}.
						<span class={`${player === name ? 'font-bold' : ''}`}>{player}</span>
						{#if !players[player].connected}
							<span class="text-error-500">(afk)</span>
						{/if}

						{#if stage === 'Joining' || ((stage === 'PlayersChoose' || stage === 'Voting') && player !== activePlayer)}
							{#if players[player].ready}
								<span>🟢</span>
							{:else}
								<span>🔴</span>
							{/if}
						{/if}
					</div>
					<div class="font-right">{players[player].points}</div>
				</div>
			{/each}
		</div>
	</div>
</div>
