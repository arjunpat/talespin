<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import { Avatar } from '@skeletonlabs/skeleton';

	export let displayImages: string[] = [];
	export let activeCard = '';
	// export let activePlayer = '';
	// export let name = '';
	// export let description = '';
	export let gameServer: GameServer;
	export let playerToCurrentCard: { [key: string]: string } = {};
	export let playerToVote: { [key: string]: string } = {};

	let cardToPlayer: { [key: string]: string } = {};
	let cardToVoters: { [key: string]: string[] } = {};

	$: {
		console.log('updated');
		Object.entries(playerToCurrentCard).forEach(([key, value]) => {
			cardToPlayer[value] = key;
		});

		Object.entries(playerToVote).forEach(([key, value]) => {
			if (!cardToVoters[value]) {
				cardToVoters[value] = [];
			}
			cardToVoters[value].push(key);
		});
	}
</script>

<div class="flex justify-center">
	<div>
		<section class="grid grid-cols-2 md:grid-cols-3 gap-4 mt-5 max-w-3xl">
			{#each displayImages as image}
				<div
					class={`${activeCard == image ? 'boujee-border' : ''} rounded-lg overflow-hidden relative`}
				>
					<img
						src="../../assets/cards/{image}"
						alt="You can't play this game without the images!"
						class="relative"
					/>
					{#if cardToVoters[image]}
						<div class="absolute" style="top: 20px; right: 12px;">
							<div class="flex flex-col gap-2">
								{#each cardToVoters[image] as voter}
									<div class="bg-success-500 px-1.5 rounded text-black font-bold">
										🔘 {voter}
									</div>
								{/each}
							</div>
						</div>
					{/if}
					<div
						style="bottom: 0;"
						class="rounded-tr w-full absolute bg-primary-200 p-0.5 px-2 text-black font-bold"
					>
						{cardToPlayer[image]}'s card
					</div>
				</div>
			{/each}
		</section>
	</div>
</div>

<button class="btn variant-filled mt-5" on:click={() => gameServer.ready()}>Next Round</button>

<style>
	@property --bg-angle {
		inherits: false;
		initial-value: 0deg;
		syntax: '<angle>';
	}
	.boujee-border {
		animation: spin 2.5s infinite linear;
		background:
			linear-gradient(to bottom, rgb(var(--color-primary-500)), rgb(var(--color-primary-500)))
				padding-box,
			conic-gradient(from var(--bg-angle) in oklch longer hue, rgb(var(--color-success-500)) 0 0)
				border-box;
		border: 5px solid transparent;
		box-shadow: 0.125rem 0.25rem 0.25rem 0.5rem oklch(0.1 0.37 315 / 0.25);
	}

	@keyframes spin {
		to {
			--bg-angle: 360deg;
		}
	}
</style>
