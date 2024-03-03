<script lang="ts">
	import { Accordion, AccordionItem, ProgressBar } from '@skeletonlabs/skeleton';
	import { InfoIcon } from 'svelte-feather-icons';

	import GameServer from '$lib/gameServer';
	import Images from './Images.svelte';
	import ActiveChooses from './ActiveChooses.svelte';

	export let displayImages: string[] = [];
	export let name = '';
	export let activePlayer = '';
	export let gameServer: GameServer;
	export let description = '';

	let selectedImage = '';
</script>

<div class="flex justify-center">
	<div>
		<div class="mt-5">
			{#if activePlayer === name}
				<h1 class="text-2xl mb-3">Tallying the votes!</h1>
				<h1 class="text-xl mt-7">Here's what everyone chose:</h1>
				<Images {displayImages} bind:selectedImage selectable={activePlayer !== name} />

				<h1 class="text-xl my-5">How points work</h1>
				<div class="card light mb-20">
					<Accordion>
						<AccordionItem>
							<svelte:fragment slot="summary">
								<strong>If all players find your card:</strong>
							</svelte:fragment>
							<svelte:fragment slot="content">
								<ul class="ml-8">
									<li>+0 for you</li>
									<li>+2 for everyone else</li>
								</ul>
							</svelte:fragment>
						</AccordionItem>
						<AccordionItem>
							<svelte:fragment slot="summary">
								<strong>If nobody finds your card:</strong>
							</svelte:fragment>
							<svelte:fragment slot="content">
								<ul class="ml-8">
									<li>+0 for you</li>
									<li>+2 for everyone else</li>
									<li>+1 bonus point for each vote a player receives</li>
								</ul>
							</svelte:fragment>
						</AccordionItem>
						<AccordionItem>
							<svelte:fragment slot="summary">
								<strong>Otherwise</strong>
							</svelte:fragment>
							<svelte:fragment slot="content">
								<ul class="ml-8">
									<li>+3 for you</li>
									<li>+3 for those you find your card</li>
									<li>+1 bonus point for each vote a player receives</li>
								</ul>
							</svelte:fragment>
						</AccordionItem>
					</Accordion>
				</div>
			{:else}
				<h1 class="text-2xl">
					Which card did <span class="font-bold">{activePlayer}</span> choose for "{description}"?
				</h1>
				<p class="text-xl mt-5">Don't fall for the fakes!</p>
				<Images {displayImages} bind:selectedImage selectable={activePlayer !== name} />

				<div class="flex justify-center">
					<button
						class="btn variant-filled mt-5"
						disabled={selectedImage === ''}
						on:click={() => gameServer.vote(selectedImage)}>Vote</button
					>
				</div>

				<h1 class="text-xl my-5 mt-20">How points work</h1>
				<div class="card light mb-20">
					<Accordion>
						<AccordionItem>
							<svelte:fragment slot="summary">
								<strong
									>If all players find <span class="boujee-text">{activePlayer}'s</span> card:</strong
								>
							</svelte:fragment>
							<svelte:fragment slot="content">
								<ul class="ml-8">
									<li>+0 for <span class="font-bold">{activePlayer}</span></li>
									<li>+2 for everyone else</li>
								</ul>
							</svelte:fragment>
						</AccordionItem>
						<AccordionItem>
							<svelte:fragment slot="summary">
								<strong
									>If nobody finds <span class="boujee-text">{activePlayer}'s</span> card:</strong
								>
							</svelte:fragment>
							<svelte:fragment slot="content">
								<ul class="ml-8">
									<li>+0 for <span class="font-bold">{activePlayer}</span></li>
									<li>+2 for everyone else</li>
									<li>+1 bonus point for each vote your card receives</li>
								</ul>
							</svelte:fragment>
						</AccordionItem>
						<AccordionItem>
							<svelte:fragment slot="summary">
								<strong>Otherwise</strong>
							</svelte:fragment>
							<svelte:fragment slot="content">
								<ul class="ml-8">
									<li>+3 for <span class="font-bold">{activePlayer}</span></li>
									<li>+3 for you if you find <span>{activePlayer}'s</span> card</li>
									<li>+1 bonus point for each vote your card receives</li>
								</ul>
							</svelte:fragment>
						</AccordionItem>
					</Accordion>
				</div>
			{/if}
		</div>
	</div>
</div>
