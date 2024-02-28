<script lang="ts">
	import { onMount } from 'svelte';
	const host = 'localhost:8080';

	onMount(() => {
		console.log('Hello from +page.svelte');

		let ws = new WebSocket(`ws://${host}/ws`);
		ws.onopen = () => {
			console.log('Connected to server');
			ws.send(
				JSON.stringify({
					CreateRoom: {
						name: 'arjun'
					}
				})
			);
		};

		ws.onmessage = (event) => {
			let data = JSON.parse(event.data);
			console.log('Message from server', data);
		};

		ws.onclose = () => {
			console.log('Disconnected from server');
		};
	});
</script>

<!-- YOU CAN DELETE EVERYTHING IN THIS PAGE -->

<div class="container h-full mx-auto flex justify-center items-center">
	<div class="space-y-5">
		<h1 class="h1">Let's get cracking bones!</h1>
		<p>Start by exploring:</p>
		<ul>
			<li><code class="code">/src/routes/+layout.svelte</code> - barebones layout</li>
			<li><code class="code">/src/app.postcss</code> - app wide css</li>
			<li>
				<code class="code">/src/routes/+page.svelte</code> - this page, you can replace the contents
			</li>
		</ul>
	</div>
</div>
