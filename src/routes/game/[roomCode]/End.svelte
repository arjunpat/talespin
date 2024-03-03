<script lang="ts">
	import type { PlayerInfo } from '$lib/types';
	export let players: { [key: string]: PlayerInfo } = {};
	let sortedPlayersList: string[] = [];

	$: {
		sortedPlayersList = Object.keys(players).sort((a, b) => {
			return players[b].points - players[a].points;
		});
	}
</script>

<div class="flex w-80/10 justify-center">
	<div>
		<h1 class="text-4xl mb-10 text-center">Game Over!</h1>

		<div class="light p-4">
			<div>
				{#each sortedPlayersList as player, i}
					<div class="flex space-between w-44 text-xl">
						<div class="flex-auto">
							{i + 1}.
							<span class={`${i === 0 ? 'boujee-text' : ''} `}>{player}</span>
						</div>
						<div class="font-right">
							{players[player].points}
						</div>
					</div>
				{/each}
			</div>
		</div>
	</div>
</div>
