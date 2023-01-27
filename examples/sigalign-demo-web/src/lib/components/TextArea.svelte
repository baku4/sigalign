<script lang="ts">
	export let value: string = '';
  export let height_rem: number = 1;

</script>

<div class="container">
	<textarea bind:value style="height: {height_rem}rem;"></textarea>
</div>

<style>
	.container {
		position: relative;
	}

  textarea {
    font-family: inherit;
    box-sizing: border-box;
    top: 0;
    width: 100%;
    border: 1px solid #213547;
    resize: none;
  }
	
	/* pre, textarea {
		font-family: inherit;
		padding: 0.5em;
		box-sizing: border-box;
		border: 1px solid #eee;
		line-height: 1.0;
		overflow: hidden;
	} */
	
	/* textarea {
		position: absolute;
		width: 100%;
		height: 100%;
		top: 0;
		resize: none;
	} */
</style>