<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import Images from './Images.svelte';
	import { getToastStore } from '@skeletonlabs/skeleton';

	export let displayImages: string[];
	export let activePlayer: string;
	export let name: string;
	export let gameServer: GameServer;

	let toastStore = getToastStore();
	let descriptionBox = '';
	let selectedImage = '';

	function activePlayerChoose() {
		gameServer.activePlayerChoose(selectedImage, descriptionBox);
	}

	if (name === activePlayer) {
		toastStore.trigger({
			message: '👉 Your turn!',
			autohide: true,
			timeout: 5000
		});
	}
</script>

<div class="flex justify-center">
	<div>
		<h1 class="text-2xl">{name}, your six cards:</h1>
		<Images {displayImages} bind:selectedImage selectable={activePlayer === name} />
		<div class="pt-5">
			{#if activePlayer === name}
				<h1 class="text-xl">Choose a card and write a one-word description</h1>
				<input
					type="text"
					placeholder="Description"
					bind:value={descriptionBox}
					class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline mt-5 mb-5"
				/>

				{#if descriptionBox.includes(' ')}
					<p class="text-red-500">Description must be one word</p>
				{/if}

				<div class="flex justify-center">
					<button
						class="btn variant-filled"
						disabled={selectedImage === '' || descriptionBox === ''}
						on:click={activePlayerChoose}>Choose</button
					>
				</div>
			{:else}
				<h1 class="text-xl">Sit tight!</h1>
				<p>Waiting for {activePlayer} to choose a card and description</p>
			{/if}
		</div>
	</div>
</div>
