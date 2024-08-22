<script lang="ts">
	import '../app.css'
	import LeftSidebar from '$lib/LeftSidebar.svelte'
</script>

<div class="grid h-screen w-screen grid-rows-[auto_1fr_auto]">
	<!-- Header -->
	<!-- <header class="bg-red-500 p-4">(header)</header> -->
	<!-- Grid Columns -->
	<div class="grid grid-cols-[auto_1fr]">
		<!-- Left Sidebar. -->
		<!-- <aside class="bg-yellow-500 p-4">(sidebar)</aside> -->
		<LeftSidebar />
		<!-- Main Content -->
		<slot />
	</div>
	<!-- Footer -->
	<!-- <footer class="bg-blue-500 p-4">(footer)</footer> -->
</div>
