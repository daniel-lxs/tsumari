<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import ProgressCircle from "$lib/components/ui/progress-circle/progress-circle.svelte";
  import SimpleCard from "$lib/components/ui/simple-card/simple-card.svelte";
  import * as Table from "$lib/components/ui/table";

  type ProcessSort = "cpu" | "ram";

  interface SystemInfo {
    cpu_usage: number;
    memory: {
      used: number;
      total: number;
      percentage: number;
    };
    processes: {
      name: string;
      cpu: number;
      memory: number;
    }[];
  }

  interface StorageInfo {
    used: number;
    total: number;
    percentage: number;
  }

  let isConnected = $state(false);
  let connectionConfig = $state({
    host: "100.98.93.96",
    port: 22,
    username: "opc",
  });

  let systemInfo = $state<SystemInfo>({
    cpu_usage: 0,
    memory: {
      used: 0,
      total: 0,
      percentage: 0,
    },
    processes: [],
  });

  let storageInfo = $state<StorageInfo>({
    used: 0,
    total: 0,
    percentage: 0,
  });

  let sortBy = $state<ProcessSort>("cpu");

  async function connectSSH() {
    try {
      await invoke("connect_ssh", {
        ...connectionConfig,
        port: Number(connectionConfig.port),
      });
      console.log("SSH connection established.");

      isConnected = true;
      getSystemInfo();
      getStorageInfo();
    } catch (error) {
      console.error("SSH connection failed:", error);
      isConnected = false;
    }
  }

  async function disconnectSSH() {
    try {
      const response = await invoke("disconnect_ssh");
      isConnected = false;
      systemInfo = {
        cpu_usage: 0,
        memory: {
          used: 0,
          total: 0,
          percentage: 0,
        },
        processes: [],
      };
      storageInfo = {
        used: 0,
        total: 0,
        percentage: 0,
      };
      console.log(response);
    } catch (error) {
      console.error("SSH disconnection failed:", error);
    }
  }

  async function getSystemInfo() {
    let retryCount = 0;
    const maxRetries = 5;
    const timeout = 2000; // 2 seconds

    while (isConnected) {
      try {
        systemInfo = await invoke<SystemInfo>("get_system_info", { sortBy });
        retryCount = 0;
      } catch (error) {
        retryCount++;
        console.error(
          `Failed to get system info, retrying: ${retryCount} of ${maxRetries}: `,
          error,
        );
        if (retryCount >= maxRetries) {
          isConnected = false;
          console.error("Max retries reached, exiting loop.");
          break;
        }
      }

      await new Promise((r) => setTimeout(r, timeout));
    }
  }

  async function getStorageInfo() {
    let retryCount = 0;
    const maxRetries = 5;
    const timeout = 1000 * 60 * 5; // 5 minutes

    while (isConnected) {
      try {
        storageInfo = await invoke<StorageInfo>("get_disk_usage");
        retryCount = 0;
      } catch (error) {
        retryCount++;
        console.error(
          `Failed to get storage info, retrying: ${retryCount} of ${maxRetries}: `,
          error,
        );
        if (retryCount >= maxRetries) {
          isConnected = false;
          console.error("Max retries reached, exiting loop.");
          break;
        }
      }

      await new Promise((r) => setTimeout(r, timeout));
    }
  }

  function setSortBy(newSort: ProcessSort) {
    sortBy = newSort;
  }
</script>

<main class="flex flex-col h-full gap-4 p-4">
  <Card.Root>
    <Card.Header>
      <Card.Title
        >SSH Connection {isConnected ? "Connected" : "Disconnected"}</Card.Title
      >
    </Card.Header>
    <Card.Content>
      <div class="space-y-2">
        <div class="flex gap-4">
          <SimpleCard title="CPU">
            <ProgressCircle value={systemInfo.cpu_usage} max={100} />
          </SimpleCard>
          <SimpleCard title="Memory">
            <ProgressCircle value={systemInfo.memory.percentage} max={100}>
              <div class="flex flex-col items-center">
                <span class="text-xs text-muted-foreground">Used</span>
                <span class="text-sm border-b"
                  >{systemInfo.memory.used.toFixed(1)} GB</span
                >
                <span class="text-sm border-t"
                  >{systemInfo.memory.total.toFixed(1)} GB</span
                >
                <span class="text-xs text-muted-foreground">Total</span>
              </div>
            </ProgressCircle>
          </SimpleCard>
          <SimpleCard title="Storage">
            <ProgressCircle value={storageInfo.percentage} max={100}>
              <div class="flex flex-col items-center">
                <span
                  class="before:content-['Used'] md:before:content-['Used_Space'] text-xs text-muted-foreground"
                ></span>
                <span class="text-sm border-b"
                  >{storageInfo.used.toFixed(1)} GB</span
                >
                <span class="text-sm border-t"
                  >{storageInfo.total.toFixed(1)} GB</span
                >
                <span class="text-xs text-muted-foreground">Total</span>
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

  <Card.Root class="flex flex-col flex-1">
    <Card.Header class="flex flex-row items-center justify-between">
      <Card.Title>Top Processes</Card.Title>
      <div class="flex space-x-2">
        <Button
          variant={sortBy === "cpu" ? "default" : "outline"}
          on:click={() => setSortBy("cpu")}
        >
          Sort by CPU
        </Button>
        <Button
          variant={sortBy === "ram" ? "default" : "outline"}
          on:click={() => setSortBy("ram")}
        >
          Sort by RAM
        </Button>
      </div>
    </Card.Header>
    <Card.Content class="flex-1">
      <div class="relative rounded-xl border overflow-hidden h-full">
        <div class="overflow-hidden h-full flex flex-col">
          <div class="relative flex-1 min-h-28">
            <Table.Root>
              <Table.Header class="sticky top-0 bg-background z-10">
                <Table.Row>
                  <Table.Head class="w-[200px]">Process</Table.Head>
                  <Table.Head class="w-[100px] text-center"
                    >CPU Usage</Table.Head
                  >
                  <Table.Head class="w-[100px] text-center"
                    >Memory Usage</Table.Head
                  >
                </Table.Row>
              </Table.Header>
            </Table.Root>
            <div class="overflow-auto absolute inset-0 top-[37px]">
              <Table.Root>
                <Table.Body>
                  {#each systemInfo.processes as process}
                    <Table.Row>
                      <Table.Cell class="w-[200px] truncate"
                        >{process.name}</Table.Cell
                      >
                      <Table.Cell class="w-[100px] text-center"
                        >{process.cpu.toFixed(1)}%</Table.Cell
                      >
                      <Table.Cell class="w-[100px] text-center"
                        >{process.memory.toFixed(1)}%</Table.Cell
                      >
                    </Table.Row>
                  {/each}
                </Table.Body>
              </Table.Root>
            </div>
          </div>
        </div>
      </div>
    </Card.Content>
  </Card.Root>
</main>
