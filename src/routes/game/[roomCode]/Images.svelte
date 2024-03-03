<script lang="ts">
	export let displayImages: string[];
	export let selectable = false;
	export let selectedImage = '';
</script>

<section class="grid grid-cols-2 md:grid-cols-3 gap-4 mt-5 max-w-2xl">
	{#if selectable}
		{#each displayImages as image}
			<div class="group" on:click={() => (selectedImage = image)}>
				<img
					class={`${selectedImage === image ? 'border-4 border-white shadow-xlg' : ''} transition-all duration-150 ease-in-out group-hover:scale-110 group-hover:shadow-2xl group-focus:shadow-2xl rounded-lg cursor-pointer`}
					src="../../assets/cards/{image}"
					alt="You can't play this game without the images!"
				/>
			</div>
		{/each}
	{:else}
		{#each displayImages as image}
			<img
				class={`transition-all duration-150 ease-in-out group-hover:scale-110 group-hover:shadow-2xl group-focus:shadow-2xl rounded-lg`}
				src="../../assets/cards/{image}"
				alt="You can't play this game without the images!"
			/>
		{/each}
	{/if}
</section>
