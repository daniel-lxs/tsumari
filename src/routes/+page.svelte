<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import ProgressCircle from '$lib/components/ui/progress-circle/progress-circle.svelte';
  import SimpleCard from '$lib/components/ui/simple-card/simple-card.svelte';

  let isConnected = $state(false);
  let connectionConfig = $state({
    host: '100.98.93.96',
    port: 22,
    username: 'opc',
  });

  let cpuUsage = $state(0);
  let memoryUsage = $state(0);
  let diskUsage = $state<string[]>(['0', '0', '0']);

  async function connectSSH() {
    try {
      await invoke('connect_ssh', {
        ...connectionConfig,
        port: Number(connectionConfig.port),
      });
      console.log('SSH connection established.');

      isConnected = true;

      getSysInfo();
      getDiskUsage();
    } catch (error) {
      console.error('SSH connection failed:', error);
      isConnected = false;
    }
  }

  async function disconnectSSH() {
    try {
      const response = await invoke('disconnect_ssh');
      isConnected = false;
      console.log(response);
    } catch (error) {
      console.error('SSH disconnection failed:', error);
    }
  }

  async function getSysInfo() {
    let retryCount = 0;
    const maxRetries = 5;
    const timeout = 1000; // 1 second

    while (isConnected) {
      // Only run loop if connected
      try {
        memoryUsage = Number(await invoke<string>('get_memory_usage'));
        cpuUsage = Number(await invoke<string>('get_cpu_usage'));

        retryCount = 0; // Reset retry count on success
      } catch (cpuError) {
        retryCount++;
        console.error(
          `Failed to get system usage, retrying: ${retryCount} of ${maxRetries}: `,
          cpuError
        );
        if (retryCount >= maxRetries) {
          isConnected = false;
          console.error('Max retries reached, exiting loop.');
          break;
        }
      }

      await new Promise((r) => setTimeout(r, timeout));
    }
  }

  async function getDiskUsage() {
    let retryCount = 0;
    const maxRetries = 5;
    const timeout = 5 * 60 * 1000; // 5 minutes

    while (isConnected) {
      // Only run loop if connected
      try {
        diskUsage = await invoke<string[]>('get_disk_usage');

        retryCount = 0; // Reset retry count on success
      } catch (cpuError) {
        retryCount++;
        console.error(
          `Failed to get disk usage, retrying: ${retryCount} of ${maxRetries}: `,
          cpuError
        );
        if (retryCount >= maxRetries) {
          isConnected = false;
          console.error('Max retries reached, exiting loop.');
          break;
        }
      }

      await new Promise((r) => setTimeout(r, timeout));
    }
  }
</script>

<main class="p-4">
  <Card.Root>
    <Card.Header>
      <Card.Title
        >SSH Connection {isConnected ? 'Connected' : 'Disconnected'}</Card.Title
      >
    </Card.Header>
    <Card.Content>
      <div class="space-y-2">
        <div class="flex gap-4">
          <SimpleCard title="CPU"
            ><ProgressCircle value={cpuUsage} max={100} /></SimpleCard
          >
          <SimpleCard title="Memory"
            ><ProgressCircle value={memoryUsage} max={100} /></SimpleCard
          >
          <SimpleCard title="Storage"
            ><ProgressCircle value={Number(diskUsage[2])} max={100}>
              <div class="flex flex-col gap-2">
                <span class="text-xs">Used: {diskUsage[0]}GiB</span>
                <hr />
                <span class="text-xs">Total: {diskUsage[1]}GiB</span>
              </div>
            </ProgressCircle>
          </SimpleCard>
        </div>
        <div class="flex space-x-2">
          <Button on:click={connectSSH} variant="outline">Connect</Button>
          <Button on:click={disconnectSSH} variant="outline">Disconnect</Button>
        </div>
      </div>
    </Card.Content>
  </Card.Root>
</main>

<style>
</style>
