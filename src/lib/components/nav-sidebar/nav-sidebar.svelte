<script>
  import { Home, SquareActivity } from 'lucide-svelte';
  import { Button } from '../ui/button';
</script>

<aside
  class="bg-background w-16 flex flex-col items-center space-y-2 py-4 border-r"
>
  <Button class="w-10 h-10" variant="ghost" size="icon"><Home /></Button>
  <Button class="w-10 h-10" variant="ghost" size="icon"
    ><SquareActivity /></Button
  >
</aside>
