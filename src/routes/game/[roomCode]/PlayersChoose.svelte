<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import Images from './Images.svelte';
	import { getToastStore } from '@skeletonlabs/skeleton';

	export let displayImages: string[] = [];
	export let name = '';
	export let activePlayer = '';
	export let gameServer: GameServer;
	export let description = '';

	let toastStore = getToastStore();
	let selectedImage = '';

	if (name === activePlayer) {
		toastStore.trigger({
			message: '👉 Your turn!',
			autohide: true,
			timeout: 5000
		});
	}

	function choose() {
		gameServer.playersChoose(selectedImage);
		toastStore.trigger({
			message: '👌 Locked in!',
			autohide: true,
			timeout: 2500
		});
	}
</script>

<div class="flex justify-center">
	<div>
		<h1 class="text-2xl">{name}, your hand:</h1>
		<Images {displayImages} bind:selectedImage selectable={activePlayer !== name} />

		<div class="pt-5">
			{#if activePlayer === name}
				<h1 class="text-xl">Sit tight!</h1>
				<p>Players are choosing cards that match "{description}"</p>
			{:else}
				<h1 class="text-xl">Your turn!</h1>
				<p>
					Choose a card that <strong>{activePlayer}</strong> would put for "{description}"
				</p>
				<div class="flex justify-center mt-5">
					<button class="btn variant-filled" disabled={selectedImage === ''} on:click={choose}
						>Choose</button
					>
				</div>
			{/if}
		</div>
	</div>
</div>
