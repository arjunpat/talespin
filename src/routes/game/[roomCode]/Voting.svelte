<script lang="ts">
	import GameServer from '$lib/gameServer';
	import Images from './Images.svelte';

	export let displayImages: string[] = [];
	export let name = '';
	export let activePlayer = '';
	export let gameServer: GameServer;
	export let description = '';

	let selectedImage = '';
</script>

<div class="flex justify-center">
	<div>
		<h1 class="text-2xl">Here's what everyone chose:</h1>
		<Images {displayImages} bind:selectedImage selectable={activePlayer !== name} />
		<div class="mt-5">
			{#if activePlayer === name}
				<h1 class="text-xl">Tallying the votes!</h1>
				<br />
				<strong>If all players find your card:</strong>
				<ul class="ml-8">
					<li>+0 for you</li>
					<li>+2 for everyone else</li>
				</ul>

				<strong>If nobody finds your card:</strong>
				<ul class="ml-8">
					<li>+0 for you</li>
					<li>+2 for everyone else</li>
					<li>+1 bonus point for each vote a player receives</li>
				</ul>

				<strong>Otherwise</strong>
				<ul class="ml-8">
					<li>+3 for you</li>
					<li>+3 for those you find your card</li>
					<li>+1 bonus point for each vote a player receives</li>
				</ul>
			{:else}
				<strong>If all players find <strong>{activePlayer}'s</strong> card:</strong>
				<ul class="ml-8">
					<li>+0 for <strong>{activePlayer}</strong></li>
					<li>+2 for everyone else</li>
				</ul>

				<strong>If nobody finds <strong>{activePlayer}'s</strong> card:</strong>
				<ul class="ml-8">
					<li>+0 for <strong>{activePlayer}</strong></li>
					<li>+2 for everyone else</li>
					<li>+1 bonus point for each vote your card receives</li>
				</ul>

				<strong>Otherwise</strong>
				<ul class="ml-8">
					<li>+3 for <strong>{activePlayer}</strong></li>
					<li>+3 for if you find <strong>{activePlayer}'s</strong> card</li>
					<li>+1 bonus point for each vote your card receives</li>
				</ul>

				<h2>Which card did <strong>{activePlayer}</strong> choose for "{description}"?</h2>
				<button
					class="btn variant-filled"
					disabled={selectedImage === ''}
					on:click={() => gameServer.vote(selectedImage)}>Vote</button
				>
			{/if}
		</div>
	</div>
</div>
