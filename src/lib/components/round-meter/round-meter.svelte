<script lang="ts">
  import ProgressCircle from '../ui/progress-circle/progress-circle.svelte';

  export let label = '';
  export let value = 0;
  export let max = 100;
</script>

<div class="flex flex-col items-center gap-2 bg-primary-foreground p-4 rounded">
  <span>{label}</span>
  <ProgressCircle value={Number(value)} {max} />
</div>
